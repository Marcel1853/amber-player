//! Audio aus dem Netz (Internetradio): Ein eigener Thread lädt und dekodiert vorab in einen
//! Puffer, damit Netzaussetzer die Wiedergabe nicht anhalten. Songtitel, die Radiosender per
//! ICY-Metadaten mitschicken, werden mitgelesen.

use std::io::Read;
use std::sync::mpsc::{sync_channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use rodio::source::SeekError;
use rodio::Source;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, ReadOnlySource};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

const USER_AGENT: &str = concat!("AmberPlayer/", env!("CARGO_PKG_VERSION"));
/// Etwa 4–8 Sekunden Puffer, je nach Paketgröße.
const BUFFERED_CHUNKS: usize = 96;

/// Vom Sender gemeldeter Songtitel (ändert sich während der Wiedergabe).
pub type StreamTitle = Arc<Mutex<Option<String>>>;

struct Chunk {
    samples: Vec<f32>,
    channels: u16,
    rate: u32,
}

pub struct NetSource {
    rx: Receiver<Chunk>,
    buf: Vec<f32>,
    pos: usize,
    channels: u16,
    rate: u32,
    ended: bool,
}

/// Liest einen HTTP-Datenstrom und entfernt dabei die eingebetteten ICY-Metadaten.
struct IcyReader<R> {
    inner: R,
    metaint: usize,
    until_meta: usize,
    title: StreamTitle,
}

impl<R: Read> Read for IcyReader<R> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.metaint == 0 {
            return self.inner.read(out);
        }
        if self.until_meta == 0 {
            let mut len = [0u8; 1];
            self.inner.read_exact(&mut len)?;
            let mut meta = vec![0u8; len[0] as usize * 16];
            self.inner.read_exact(&mut meta)?;
            if let Some(t) = parse_stream_title(&meta) {
                *self.title.lock().unwrap_or_else(|e| e.into_inner()) = Some(t);
            }
            self.until_meta = self.metaint;
        }
        let n = out.len().min(self.until_meta);
        let read = self.inner.read(&mut out[..n])?;
        self.until_meta -= read;
        Ok(read)
    }
}

fn parse_stream_title(meta: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(meta);
    let start = text.find("StreamTitle='")? + "StreamTitle='".len();
    let rest = &text[start..];
    let end = rest.find("';").unwrap_or_else(|| rest.trim_end_matches('\0').len());
    let title = rest[..end].trim().to_string();
    (!title.is_empty()).then_some(title)
}

/// Mime-Typ des Senders als Hinweis für die Formaterkennung.
fn hint_for(content_type: &str, url: &str) -> Hint {
    let mut hint = Hint::new();
    let ct = content_type.to_ascii_lowercase();
    let ext = if ct.contains("mpeg") || ct.contains("mp3") {
        Some("mp3")
    } else if ct.contains("aac") || ct.contains("aacp") {
        Some("aac")
    } else if ct.contains("ogg") || ct.contains("opus") {
        Some("ogg")
    } else if ct.contains("flac") {
        Some("flac")
    } else {
        url.rsplit('.').next().filter(|e| e.len() <= 4)
    };
    if let Some(e) = ext {
        hint.with_extension(e);
    }
    if !ct.is_empty() {
        hint.mime_type(&ct);
    }
    hint
}

/// Wiedergabelisten (.m3u/.pls) eines Senders auflösen: erste Stream-Adresse nehmen.
fn resolve_playlist(url: &str) -> Result<String> {
    let lower = url.to_ascii_lowercase();
    let path = lower.split(['?', '#']).next().unwrap_or("");
    if !(path.ends_with(".m3u") || path.ends_with(".m3u8") || path.ends_with(".pls")) {
        return Ok(url.to_string());
    }
    let body = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .timeout(Duration::from_secs(10))
        .call()?
        .into_string()?;
    body.lines()
        .map(str::trim)
        .find_map(|l| {
            let l = l.split_once('=').filter(|(k, _)| k.to_ascii_lowercase().starts_with("file")).map_or(l, |(_, v)| v);
            (l.starts_with("http://") || l.starts_with("https://")).then(|| l.to_string())
        })
        .ok_or_else(|| anyhow!("In der Senderliste steht keine Stream-Adresse"))
}

impl NetSource {
    /// Verbindet sich und wartet, bis die ersten Töne dekodiert sind.
    pub fn open(url: &str, title: StreamTitle) -> Result<Self> {
        let url = resolve_playlist(url)?;
        // Kein Gesamt-Zeitlimit (Radio läuft endlos), aber hängende Sender erkennen.
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(20))
            .build();
        let resp = agent
            .get(&url)
            .set("User-Agent", USER_AGENT)
            .set("Icy-MetaData", "1")
            .call()
            .map_err(|e| anyhow!("Sender nicht erreichbar: {e}"))?;
        let content_type = resp.content_type().to_string();
        if content_type.starts_with("text/html") || content_type.starts_with("video/") {
            bail!("Unter der Adresse liegt kein Audio-Stream ({content_type})");
        }
        let metaint = resp.header("icy-metaint").and_then(|v| v.trim().parse().ok()).unwrap_or(0);
        if let Some(name) = resp.header("icy-name").filter(|n| !n.trim().is_empty()) {
            log::info!("Sender: {name}");
        }
        let reader = IcyReader { inner: resp.into_reader(), metaint, until_meta: metaint, title };
        let mss = MediaSourceStream::new(Box::new(ReadOnlySource::new(reader)), Default::default());
        let hint = hint_for(&content_type, &url);

        let (tx, rx) = sync_channel::<Chunk>(BUFFERED_CHUNKS);
        let (ready_tx, ready_rx) = sync_channel::<Result<(), String>>(1);
        std::thread::Builder::new()
            .name("amber-player-radio".into())
            .spawn(move || {
                let probed = symphonia::default::get_probe().format(
                    &hint,
                    mss,
                    &FormatOptions::default(),
                    &MetadataOptions::default(),
                );
                let mut format = match probed {
                    Ok(p) => p.format,
                    Err(e) => {
                        let _ = ready_tx.send(Err(format!("Format nicht erkannt: {e}")));
                        return;
                    }
                };
                let Some(track) = format.tracks().iter().find(|t| t.codec_params.codec != CODEC_TYPE_NULL) else {
                    let _ = ready_tx.send(Err("Keine Audiospur im Stream".into()));
                    return;
                };
                let track_id = track.id;
                let mut decoder = match symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default()) {
                    Ok(d) => d,
                    Err(e) => {
                        let _ = ready_tx.send(Err(format!("Codec nicht unterstützt: {e}")));
                        return;
                    }
                };
                let mut sent_ready = false;
                let mut bad = 0;
                loop {
                    let packet = match format.next_packet() {
                        Ok(p) => p,
                        Err(SymError::ResetRequired) => {
                            decoder.reset();
                            continue;
                        }
                        Err(_) => break,
                    };
                    if packet.track_id() != track_id {
                        continue;
                    }
                    let audio = match decoder.decode(&packet) {
                        Ok(a) => a,
                        Err(SymError::DecodeError(_)) => {
                            bad += 1;
                            if bad > 500 {
                                break;
                            }
                            continue;
                        }
                        Err(_) => break,
                    };
                    bad = 0;
                    let spec = *audio.spec();
                    let mut sb = SampleBuffer::<f32>::new(audio.capacity() as u64, spec);
                    sb.copy_interleaved_ref(audio);
                    let chunk = Chunk {
                        samples: sb.samples().to_vec(),
                        channels: spec.channels.count() as u16,
                        rate: spec.rate,
                    };
                    if chunk.samples.is_empty() {
                        continue;
                    }
                    // Empfänger weg = Wiedergabe beendet → Thread endet.
                    if tx.send(chunk).is_err() {
                        return;
                    }
                    if !sent_ready {
                        sent_ready = true;
                        let _ = ready_tx.send(Ok(()));
                    }
                }
                if !sent_ready {
                    let _ = ready_tx.send(Err("Der Sender liefert keine Daten".into()));
                }
            })?;

        match ready_rx.recv_timeout(Duration::from_secs(15)) {
            Ok(Ok(())) => {}
            Ok(Err(e)) => bail!(e),
            Err(_) => bail!("Der Sender antwortet nicht"),
        }
        let first = rx.recv().map_err(|_| anyhow!("Der Sender liefert keine Daten"))?;
        Ok(Self {
            channels: first.channels,
            rate: first.rate,
            buf: first.samples,
            pos: 0,
            rx,
            ended: false,
        })
    }
}

impl Iterator for NetSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.buf.len() {
            if self.ended {
                return None;
            }
            match self.rx.try_recv() {
                Ok(c) => {
                    self.buf = c.samples;
                    self.pos = 0;
                    self.channels = c.channels;
                    self.rate = c.rate;
                }
                // Puffer leer: kurz Stille, bis wieder Daten da sind (nicht blockieren).
                Err(TryRecvError::Empty) => return Some(0.0),
                Err(TryRecvError::Disconnected) => {
                    self.ended = true;
                    return None;
                }
            }
        }
        self.pos += 1;
        Some(self.buf[self.pos - 1])
    }
}

impl Source for NetSource {
    fn current_frame_len(&self) -> Option<usize> {
        // Ist der Puffer leer, kommt Stille in Einzelframes – Kanäle/Rate bleiben gleich.
        Some(self.buf.len().saturating_sub(self.pos).max(self.channels as usize))
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }

    fn try_seek(&mut self, _pos: Duration) -> Result<(), SeekError> {
        Err(SeekError::NotSupported { underlying_source: "Internetradio" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icy_title_is_parsed() {
        let meta = b"StreamTitle='Santiano - Weh Mir';StreamUrl='';\0\0\0";
        assert_eq!(parse_stream_title(meta).as_deref(), Some("Santiano - Weh Mir"));
        assert_eq!(parse_stream_title(b"StreamTitle='';"), None);
    }

    #[test]
    fn icy_reader_strips_metadata() {
        // 4 Byte Audio, dann Metadaten (1 Block = 16 Byte), dann wieder 4 Byte Audio
        let mut data = b"ABCD".to_vec();
        data.push(1);
        let mut block = b"StreamTitle='X';".to_vec();
        block.resize(16, 0);
        data.extend(block);
        data.extend(b"EFGH");
        let title: StreamTitle = Arc::new(Mutex::new(None));
        let mut r = IcyReader { inner: &data[..], metaint: 4, until_meta: 4, title: title.clone() };
        let mut out = Vec::new();
        let mut buf = [0u8; 3];
        loop {
            match r.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => out.extend_from_slice(&buf[..n]),
            }
        }
        assert_eq!(out, b"ABCDEFGH");
        assert_eq!(title.lock().unwrap().as_deref(), Some("X"));
    }
}

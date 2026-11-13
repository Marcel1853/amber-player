//! Abspielen von Audio aus dem Netz mit Spulen: eine HTTP-Quelle, die per Range-Anfrage an jeder
//! Stelle weiterlesen kann, und ein Vorpuffer, der in einem eigenen Thread dekodiert, damit
//! Netzaussetzer die Wiedergabe nicht anhalten.

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::sync::mpsc::{channel, sync_channel, Receiver, RecvTimeoutError, Sender, SyncSender, TryRecvError};
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use rodio::source::SeekError;
use rodio::Source;
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions, ReadOnlySource};
use symphonia::core::probe::Hint;

use crate::decoder::AudioSource;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AmberPlayer";

/// Datei im Netz, die sich wie eine lokale Datei lesen und spulen lässt.
pub struct HttpSource {
    agent: ureq::Agent,
    url: String,
    headers: HashMap<String, String>,
    len: u64,
    pos: u64,
    reader: Option<Box<dyn Read + Send + Sync>>,
    content_type: String,
}

impl HttpSource {
    pub fn open(url: &str, headers: HashMap<String, String>) -> Result<Self> {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(20))
            .build();
        let mut src = Self { agent, url: url.to_string(), headers, len: 0, pos: 0, reader: None, content_type: String::new() };
        let resp = src.request(0)?;
        src.content_type = resp.content_type().to_string();
        src.len = resp
            .header("Content-Range")
            .and_then(|r| r.rsplit('/').next())
            .and_then(|n| n.trim().parse().ok())
            .or_else(|| resp.header("Content-Length").and_then(|n| n.parse().ok()))
            .ok_or_else(|| anyhow!("Der Server nennt keine Dateigröße"))?;
        src.reader = Some(resp.into_reader());
        Ok(src)
    }

    fn request(&self, from: u64) -> Result<ureq::Response> {
        range_get(&self.agent, &self.url, &self.headers, from, None)
    }

    /// Dateiendung als Hinweis für die Formaterkennung.
    pub fn hint(&self) -> Hint {
        let mut hint = Hint::new();
        let ct = self.content_type.to_ascii_lowercase();
        let ext = if ct.contains("mp4") || ct.contains("m4a") {
            Some("m4a")
        } else if ct.contains("webm") {
            Some("webm")
        } else if ct.contains("mpeg") {
            Some("mp3")
        } else if ct.contains("ogg") {
            Some("ogg")
        } else {
            None
        };
        if let Some(e) = ext {
            hint.with_extension(e);
        }
        hint
    }
}

impl Read for HttpSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.len {
            return Ok(0);
        }
        // Einmal neu verbinden, falls die Verbindung abreißt.
        for attempt in 0..2 {
            if self.reader.is_none() {
                let r = self.request(self.pos).map_err(|e| std::io::Error::other(e.to_string()))?;
                self.reader = Some(r.into_reader());
            }
            match self.reader.as_mut().unwrap().read(buf) {
                Ok(n) => {
                    self.pos += n as u64;
                    return Ok(n);
                }
                Err(e) if attempt == 0 => {
                    log::debug!("Netz-Lesefehler, verbinde neu: {e}");
                    self.reader = None;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(0)
    }
}

impl Seek for HttpSource {
    fn seek(&mut self, to: SeekFrom) -> std::io::Result<u64> {
        let new = match to {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::End(d) => self.len as i64 + d,
            SeekFrom::Current(d) => self.pos as i64 + d,
        };
        if new < 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "vor dem Anfang"));
        }
        let new = new as u64;
        if new != self.pos {
            self.pos = new;
            self.reader = None;
        }
        Ok(self.pos)
    }
}

impl MediaSource for HttpSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.len)
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(20))
        .build()
}

/// GET mit Range-Kopf ab `from` (optional bis einschließlich `to`).
fn range_get(agent: &ureq::Agent, url: &str, headers: &HashMap<String, String>, from: u64, to: Option<u64>) -> Result<ureq::Response> {
    let range = match to {
        Some(t) => format!("bytes={from}-{t}"),
        None => format!("bytes={from}-"),
    };
    let mut req = agent.get(url).set("User-Agent", USER_AGENT).set("Range", &range);
    for (k, v) in headers {
        req = req.set(k, v);
    }
    match req.call() {
        Ok(r) if r.status() == 206 || (r.status() == 200 && from == 0) => Ok(r),
        Ok(r) => bail!("Server antwortet mit {}", r.status()),
        Err(e) => bail!("Abruf fehlgeschlagen: {e}"),
    }
}

/// Öffnet eine Audio-Adresse aus dem Netz als spulbare, vorgepufferte Quelle.
pub fn open_url(url: &str, headers: HashMap<String, String>) -> Result<Prefetch> {
    let http = HttpSource::open(url, headers.clone())?;
    // Fragmentiertes MP4 (bei manchen Online-Diensten): über das Inhaltsverzeichnis abspielen statt alle Stücke anzulesen.
    if http.content_type.contains("mp4") {
        match read_dash(&http.agent, url, &headers, http.len) {
            Ok(Some(dash)) => {
                log::info!("Fragmentiertes MP4 mit {} Abschnitten", dash.segments.len());
                return Prefetch::new(DashSource::new(url.to_string(), headers, dash)?);
            }
            Ok(None) => {}
            Err(e) => log::debug!("Kein Inhaltsverzeichnis lesbar: {e:#}"),
        }
    }
    let hint = http.hint();
    let mss = MediaSourceStream::new(Box::new(http), MediaSourceStreamOptions { buffer_len: 256 * 1024 });
    let src = AudioSource::from_stream(mss, hint)?;
    Prefetch::new(src)
}

// ---------- Fragmentiertes MP4 (DASH, bei manchen Online-Diensten) ----------
// Aufbau: ftyp, moov (Beschreibung), sidx (Inhaltsverzeichnis: Zeit → Byte-Position), dann viele
// moof/mdat-Abschnitte. Abgespielt wird „Beschreibung + ab Abschnitt n“ am Stück, ohne Spulen
// im Datenstrom; gesprungen wird über das Inhaltsverzeichnis.

#[derive(Debug, Clone, PartialEq)]
struct Segment {
    time: f64,
    offset: u64,
}

#[derive(Debug, Clone)]
pub struct Dash {
    init: Vec<u8>,
    segments: Vec<Segment>,
    total: Duration,
}

fn be(b: &[u8], i: usize, n: usize) -> Option<u64> {
    b.get(i..i + n).map(|s| s.iter().fold(0u64, |a, &x| (a << 8) | x as u64))
}

/// Liest ftyp/moov und das Inhaltsverzeichnis (sidx) aus dem Dateikopf.
/// `None`, wenn die Datei kein fragmentiertes MP4 mit Inhaltsverzeichnis ist.
fn parse_dash(head: &[u8]) -> Option<Dash> {
    let mut pos = 0usize;
    let mut init = Vec::new();
    let mut sidx: Option<(usize, usize)> = None;
    let mut has_moov = false;
    while pos + 8 <= head.len() {
        let size32 = be(head, pos, 4)?;
        let typ = &head[pos + 4..pos + 8];
        let (hdr, size) = match size32 {
            1 => (16usize, be(head, pos + 8, 8)? as usize),
            0 => return None,
            n => (8usize, n as usize),
        };
        if size < hdr {
            return None;
        }
        let end = pos + size;
        match typ {
            b"moof" | b"mdat" => break,
            b"sidx" => {
                if end > head.len() {
                    return None;
                }
                sidx = Some((pos + hdr, end));
            }
            _ => {
                if end > head.len() {
                    return None;
                }
                has_moov |= typ == b"moov";
                init.extend_from_slice(&head[pos..end]);
            }
        }
        pos = end;
    }
    let (s, sidx_end) = sidx?;
    if !has_moov {
        return None;
    }
    let b = &head[s..sidx_end];
    let version = *b.first()?;
    let timescale = be(b, 8, 4)? as f64;
    let (ept, first_offset, mut p) = if version == 0 {
        (be(b, 12, 4)?, be(b, 16, 4)?, 20)
    } else {
        (be(b, 12, 8)?, be(b, 20, 8)?, 28)
    };
    let count = be(b, p + 2, 2)? as usize;
    p += 4;
    if timescale <= 0.0 || count == 0 {
        return None;
    }
    let mut offset = sidx_end as u64 + first_offset;
    let mut t = ept as f64 / timescale;
    let mut segments = Vec::with_capacity(count);
    for _ in 0..count {
        let size = be(b, p, 4)? & 0x7fff_ffff;
        let dur = be(b, p + 4, 4)? as f64 / timescale;
        segments.push(Segment { time: t, offset });
        offset += size;
        t += dur;
        p += 12;
    }
    Some(Dash { init, segments, total: Duration::from_secs_f64(t) })
}

fn read_dash(agent: &ureq::Agent, url: &str, headers: &HashMap<String, String>, len: u64) -> Result<Option<Dash>> {
    // Kopf und Inhaltsverzeichnis sind bei Audio klein; bei sehr langen Sets etwas mehr laden.
    for want in [256 * 1024u64, 2 * 1024 * 1024] {
        let to = want.min(len).saturating_sub(1);
        let mut head = Vec::new();
        range_get(agent, url, headers, 0, Some(to))?.into_reader().take(want).read_to_end(&mut head)?;
        if let Some(d) = parse_dash(&head) {
            return Ok(Some(d));
        }
        if want >= len {
            break;
        }
    }
    Ok(None)
}

/// Liest ab einer Byte-Position am Stück weiter (verbindet bei Abbrüchen neu).
struct RangeReader {
    agent: ureq::Agent,
    url: String,
    headers: HashMap<String, String>,
    pos: u64,
    reader: Option<Box<dyn Read + Send + Sync>>,
}

impl Read for RangeReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for attempt in 0..3 {
            if self.reader.is_none() {
                match range_get(&self.agent, &self.url, &self.headers, self.pos, None) {
                    Ok(r) => self.reader = Some(r.into_reader()),
                    // 416 = hinter dem Ende → fertig
                    Err(e) if e.to_string().contains("416") => return Ok(0),
                    Err(e) => return Err(std::io::Error::other(e.to_string())),
                }
            }
            match self.reader.as_mut().unwrap().read(buf) {
                Ok(n) => {
                    self.pos += n as u64;
                    return Ok(n);
                }
                Err(e) if attempt < 2 => {
                    log::debug!("Netz-Lesefehler, verbinde neu: {e}");
                    self.reader = None;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(0)
    }
}

/// Spielt fragmentiertes MP4 ab einem Abschnitt; Springen öffnet den passenden Abschnitt neu.
pub struct DashSource {
    url: String,
    headers: HashMap<String, String>,
    dash: Dash,
    inner: AudioSource,
}

impl DashSource {
    fn new(url: String, headers: HashMap<String, String>, dash: Dash) -> Result<Self> {
        let inner = Self::open_at(&url, &headers, &dash, 0)?;
        Ok(Self { url, headers, dash, inner })
    }

    fn open_at(url: &str, headers: &HashMap<String, String>, dash: &Dash, seg: usize) -> Result<AudioSource> {
        let rest = RangeReader {
            agent: agent(),
            url: url.to_string(),
            headers: headers.clone(),
            pos: dash.segments[seg].offset,
            reader: None,
        };
        let reader = std::io::Cursor::new(dash.init.clone()).chain(rest);
        let mss = MediaSourceStream::new(Box::new(ReadOnlySource::new(reader)), MediaSourceStreamOptions { buffer_len: 256 * 1024 });
        let mut hint = Hint::new();
        hint.with_extension("m4a");
        AudioSource::from_stream(mss, hint)
    }
}

impl Iterator for DashSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.inner.next()
    }
}

impl Source for DashSource {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.dash.total)
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let t = pos.as_secs_f64();
        let seg = self.dash.segments.iter().rposition(|s| s.time <= t).unwrap_or(0);
        let mut inner = Self::open_at(&self.url, &self.headers, &self.dash, seg)
            .map_err(|e| SeekError::Other(e.into()))?;
        // Innerhalb des Abschnitts bis zur genauen Stelle vorspulen.
        let frames = ((t - self.dash.segments[seg].time).max(0.0) * inner.sample_rate() as f64) as usize;
        inner.skip_samples(frames * inner.channels() as usize);
        self.inner = inner;
        Ok(())
    }
}

// ---------- Vorpuffer ----------

enum Ctl {
    Seek(Duration, u64),
}

enum Msg {
    Chunk { gen: u64, samples: Vec<f32>, channels: u16, rate: u32 },
    End { gen: u64 },
}

/// Dekodiert eine (langsame) Quelle in einem eigenen Thread vor. Springen wird an den Thread
/// weitergereicht; veraltete Daten erkennt der Empfänger an der „Generation“.
pub struct Prefetch {
    rx: Receiver<Msg>,
    ctl: Sender<Ctl>,
    gen: u64,
    buf: Vec<f32>,
    pos: usize,
    channels: u16,
    rate: u32,
    total: Option<Duration>,
    ended: bool,
}

const CHUNK_FRAMES: usize = 4096;
const CHUNKS: usize = 48;

impl Prefetch {
    pub fn new<S: Source<Item = f32> + Send + 'static>(src: S) -> Result<Self> {
        let total = src.total_duration();
        let (tx, rx) = sync_channel::<Msg>(CHUNKS);
        let (ctl_tx, ctl_rx) = channel::<Ctl>();
        std::thread::Builder::new()
            .name("amber-player-vorpuffer".into())
            .spawn(move || prefetch_loop(src, tx, ctl_rx))?;
        let mut p = Self { rx, ctl: ctl_tx, gen: 0, buf: Vec::new(), pos: 0, channels: 2, rate: 44_100, total, ended: false };
        // Auf die ersten Daten warten, damit Kanäle und Abtastrate bekannt sind.
        match p.rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Msg::Chunk { samples, channels, rate, .. }) => {
                p.buf = samples;
                p.channels = channels;
                p.rate = rate;
            }
            Ok(Msg::End { .. }) => bail!("Keine Audiodaten erhalten"),
            Err(RecvTimeoutError::Timeout) => bail!("Die Quelle antwortet nicht"),
            Err(RecvTimeoutError::Disconnected) => bail!("Keine Audiodaten erhalten"),
        }
        Ok(p)
    }
}

fn prefetch_loop<S: Source<Item = f32>>(mut src: S, tx: SyncSender<Msg>, ctl: Receiver<Ctl>) {
    let mut gen = 0;
    let mut ended = false;
    loop {
        // Steuerbefehle (Springen) zuerst; nach dem Ende auf einen Befehl warten.
        let cmd = if ended { ctl.recv().ok() } else { ctl.try_recv().ok() };
        if ended && cmd.is_none() {
            return; // Empfänger weg
        }
        if let Some(Ctl::Seek(pos, g)) = cmd {
            gen = g;
            ended = false;
            if let Err(e) = src.try_seek(pos) {
                log::warn!("Springen fehlgeschlagen: {e}");
            }
            continue;
        }
        let channels = src.channels().max(1);
        let rate = src.sample_rate();
        let samples: Vec<f32> = src.by_ref().take(CHUNK_FRAMES * channels as usize).collect();
        if samples.is_empty() {
            ended = true;
            if tx.send(Msg::End { gen }).is_err() {
                return;
            }
            continue;
        }
        if tx.send(Msg::Chunk { gen, samples, channels, rate }).is_err() {
            return;
        }
    }
}

impl Iterator for Prefetch {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        while self.pos >= self.buf.len() {
            if self.ended {
                return None;
            }
            match self.rx.try_recv() {
                Ok(Msg::Chunk { gen, samples, channels, rate }) if gen == self.gen => {
                    self.buf = samples;
                    self.pos = 0;
                    self.channels = channels;
                    self.rate = rate;
                }
                Ok(Msg::End { gen }) if gen == self.gen => self.ended = true,
                Ok(_) => {} // veraltet (vor dem Springen)
                // Puffer leer: kurz Stille statt die Wiedergabe zu blockieren.
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

impl Source for Prefetch {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.buf.len().saturating_sub(self.pos).max(self.channels as usize))
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.total
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.gen += 1;
        self.buf.clear();
        self.pos = 0;
        self.ended = false;
        self.ctl
            .send(Ctl::Seek(pos, self.gen))
            .map_err(|_| SeekError::NotSupported { underlying_source: "Vorpuffer beendet" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;

    /// Baut ein Mini-MP4: ftyp, moov, sidx (Version 0) mit zwei Abschnitten.
    fn fake_head() -> Vec<u8> {
        let boxed = |typ: &[u8], body: &[u8]| {
            let mut v = ((body.len() + 8) as u32).to_be_bytes().to_vec();
            v.extend_from_slice(typ);
            v.extend_from_slice(body);
            v
        };
        let mut sidx = vec![0, 0, 0, 0]; // Version 0, Flags
        sidx.extend(1u32.to_be_bytes()); // reference_ID
        sidx.extend(1000u32.to_be_bytes()); // timescale
        sidx.extend(0u32.to_be_bytes()); // earliest_presentation_time
        sidx.extend(0u32.to_be_bytes()); // first_offset
        sidx.extend([0, 0, 0, 2]); // reserved, reference_count = 2
        for (size, dur) in [(500u32, 10_000u32), (700, 5_000)] {
            sidx.extend(size.to_be_bytes());
            sidx.extend(dur.to_be_bytes());
            sidx.extend(0u32.to_be_bytes());
        }
        let mut head = boxed(b"ftyp", b"dash0000");
        head.extend(boxed(b"moov", &[1, 2, 3, 4]));
        head.extend(boxed(b"sidx", &sidx));
        head.extend(boxed(b"moof", &[0; 8]));
        head
    }

    #[test]
    fn dash_index_is_parsed() {
        let head = fake_head();
        let d = parse_dash(&head).unwrap();
        assert_eq!(d.init.len(), 16 + 12, "ftyp + moov, ohne sidx");
        let sidx_end = (16 + 12 + 8 + 24 + 2 * 12) as u64;
        assert_eq!(d.segments, vec![Segment { time: 0.0, offset: sidx_end }, Segment { time: 10.0, offset: sidx_end + 500 }]);
        assert_eq!(d.total, Duration::from_secs(15));
        // Ohne Inhaltsverzeichnis: kein DASH
        assert!(parse_dash(&head[..28]).is_none());
    }

    #[test]
    fn prefetch_passes_all_samples_and_seeks() {
        let samples: Vec<f32> = (0..50_000).map(|i| (i % 1000) as f32 / 1000.0 + 0.001).collect();
        let mut p = Prefetch::new(SamplesBuffer::new(1, 10_000, samples.clone())).unwrap();
        // Warten, bis alles im Puffer liegt – dann darf keine Stille dazwischenkommen.
        std::thread::sleep(Duration::from_millis(300));
        let got: Vec<f32> = p.by_ref().take(samples.len()).collect();
        assert_eq!(got, samples);
        assert_eq!(p.next(), None);

        // Springen nach 4 s = Sample 40 000 (10 kHz, mono)
        p.try_seek(Duration::from_secs(4)).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        let rest: Vec<f32> = p.by_ref().take(10_000).collect();
        assert_eq!(rest, samples[40_000..]);
    }
}

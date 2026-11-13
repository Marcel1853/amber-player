//! Eigener Decoder auf Basis von Symphonia. Anders als der Decoder von rodio bricht er bei
//! einzelnen kaputten Frames nicht ab, sondern überspringt sie – das ist bei MP3s aus
//! Online-Konvertern häufig nötig („invalid main_data offset“). Kommt Symphonia mit einer
//! MP3 gar nicht zurecht, springt minimp3 ein.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use rodio::source::SeekError;
use rodio::Source;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

/// So viele Pakete hintereinander dürfen kaputt sein, bevor wir aufgeben.
const MAX_BAD_PACKETS: usize = 200;

/// Öffnet eine Audiodatei; MP3s, an denen Symphonia scheitert, übernimmt minimp3.
pub fn open_audio(path: &Path) -> Result<Box<dyn Source<Item = f32> + Send>> {
    match AudioSource::open(path) {
        Ok(src) => Ok(Box::new(src)),
        Err(e) if is_mp3(path) => {
            log::info!("{}: Symphonia scheitert ({e:#}), versuche minimp3", path.display());
            Ok(Box::new(Mp3Source::open(path).map_err(|e2| e2.context(e))?))
        }
        Err(e) => Err(e),
    }
}

fn is_mp3(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("mp3"))
}

pub struct AudioSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    buf: Option<SampleBuffer<f32>>,
    pos: usize,
    len: usize,
    channels: u16,
    rate: u32,
    duration: Option<Duration>,
    /// Nach dem Springen bis zur genauen Stelle verworfene Samples
    skip: usize,
}

impl AudioSource {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("{} öffnen", path.display()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        Self::from_stream(mss, hint)
    }

    /// Die nächsten `n` Samples verwerfen (für genaues Springen innerhalb eines Abschnitts).
    pub fn skip_samples(&mut self, n: usize) {
        self.skip = n;
    }

    /// Dekodiert aus einer beliebigen Quelle (Datei oder Netz).
    pub fn from_stream(mss: MediaSourceStream, hint: Hint) -> Result<Self> {
        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions {
                    enable_gapless: true,
                    ..Default::default()
                },
                &MetadataOptions::default(),
            )
            .map_err(|e| anyhow!("Format nicht erkannt: {e}"))?;
        let format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow!("Keine Audiospur gefunden"))?;
        let params = &track.codec_params;
        let decoder = symphonia::default::get_codecs()
            .make(params, &DecoderOptions::default())
            .map_err(|e| anyhow!("Codec nicht unterstützt: {e}"))?;
        let duration = match (params.n_frames, params.time_base) {
            (Some(n), Some(tb)) => {
                let t = tb.calc_time(n);
                Some(Duration::from_secs(t.seconds) + Duration::from_secs_f64(t.frac))
            }
            (Some(n), None) => params.sample_rate.map(|r| Duration::from_secs_f64(n as f64 / r as f64)),
            _ => None,
        };
        let mut src = Self {
            track_id: track.id,
            channels: params.channels.map(|c| c.count() as u16).unwrap_or(2),
            rate: params.sample_rate.unwrap_or(44_100),
            format,
            decoder,
            buf: None,
            pos: 0,
            len: 0,
            duration,
            skip: 0,
        };
        if !src.refill() {
            return Err(anyhow!("Datei enthält keine abspielbaren Daten"));
        }
        Ok(src)
    }

    /// Nächstes Paket dekodieren. `false` = Ende der Datei oder nicht mehr lesbar.
    fn refill(&mut self) -> bool {
        let mut bad = 0;
        loop {
            let packet = match self.format.next_packet() {
                Ok(p) => p,
                Err(SymError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(_) => return false,
            };
            if packet.track_id() != self.track_id {
                continue;
            }
            match self.decoder.decode(&packet) {
                Ok(audio) => {
                    let spec = *audio.spec();
                    let frames = audio.capacity() as u64;
                    let fits = self.buf.as_ref().is_some_and(|b| b.capacity() as u64 >= frames * spec.channels.count() as u64);
                    if !fits {
                        self.buf = Some(SampleBuffer::new(frames, spec));
                    }
                    let buf = self.buf.as_mut().unwrap();
                    buf.copy_interleaved_ref(audio);
                    self.len = buf.samples().len();
                    self.pos = 0;
                    self.channels = spec.channels.count() as u16;
                    self.rate = spec.rate;
                    if self.len > 0 {
                        return true;
                    }
                }
                // Kaputter Frame: überspringen statt abbrechen.
                Err(SymError::DecodeError(e)) => {
                    log::debug!("Frame übersprungen: {e}");
                    bad += 1;
                    if bad > MAX_BAD_PACKETS {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }
    }
}

impl Iterator for AudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        loop {
            if self.pos >= self.len && !self.refill() {
                return None;
            }
            let s = self.buf.as_ref()?.samples()[self.pos];
            self.pos += 1;
            if self.skip > 0 {
                self.skip -= 1;
                continue;
            }
            return Some(s);
        }
    }
}

impl Source for AudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.len.saturating_sub(self.pos).max(1))
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let seeked = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: Time::from(pos.as_secs_f64()),
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| SeekError::Other(Box::new(e)))?;
        self.decoder.reset();
        self.pos = 0;
        self.len = 0;
        // Bis zur genauen Stelle vorspulen (Zeitbasis der Spur = Samples).
        let frames = seeked.required_ts.saturating_sub(seeked.actual_ts) as usize;
        self.skip = frames.min(self.rate as usize * 10) * self.channels as usize;
        Ok(())
    }
}

/// Rückfall-Decoder für MP3s (minimp3). Springen = neu öffnen und bis zur Stelle dekodieren.
pub struct Mp3Source {
    path: PathBuf,
    dec: minimp3_fixed::Decoder<BufReader<File>>,
    buf: Vec<f32>,
    pos: usize,
    channels: u16,
    rate: u32,
}

impl Mp3Source {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("{} öffnen", path.display()))?;
        let mut src = Self {
            path: path.to_path_buf(),
            dec: minimp3_fixed::Decoder::new(BufReader::new(file)),
            buf: Vec::new(),
            pos: 0,
            channels: 2,
            rate: 44_100,
        };
        if !src.refill() {
            return Err(anyhow!("Keine MP3-Daten gefunden"));
        }
        Ok(src)
    }

    fn refill(&mut self) -> bool {
        loop {
            match self.dec.next_frame() {
                Ok(frame) => {
                    if frame.data.is_empty() {
                        continue;
                    }
                    self.channels = frame.channels.max(1) as u16;
                    self.rate = frame.sample_rate.max(1) as u32;
                    self.buf.clear();
                    self.buf.extend(frame.data.iter().map(|&s| s as f32 / 32768.0));
                    self.pos = 0;
                    return true;
                }
                Err(minimp3_fixed::Error::SkippedData) | Err(minimp3_fixed::Error::InsufficientData) => continue,
                Err(_) => return false,
            }
        }
    }
}

impl Iterator for Mp3Source {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.buf.len() && !self.refill() {
            return None;
        }
        self.pos += 1;
        Some(self.buf[self.pos - 1])
    }
}

impl Source for Mp3Source {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.buf.len().saturating_sub(self.pos).max(1))
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

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let fresh = Mp3Source::open(&self.path).map_err(|e| SeekError::Other(e.into()))?;
        *self = fresh;
        let mut skip = (pos.as_secs_f64() * self.rate as f64) as usize * self.channels as usize;
        while skip > 0 {
            let left = self.buf.len() - self.pos;
            if skip < left {
                self.pos += skip;
                break;
            }
            skip -= left;
            if !self.refill() {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_and_seeks_wav() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.wav");
        let rate = 8000u32;
        let samples: Vec<i16> = (0..rate * 2).map(|i| ((i % 100) as i16 - 50) * 300).collect();
        let mut data = Vec::new();
        data.extend_from_slice(b"RIFF");
        data.extend_from_slice(&(36 + samples.len() as u32 * 2).to_le_bytes());
        data.extend_from_slice(b"WAVEfmt ");
        data.extend_from_slice(&16u32.to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&rate.to_le_bytes());
        data.extend_from_slice(&(rate * 2).to_le_bytes());
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(&16u16.to_le_bytes());
        data.extend_from_slice(b"data");
        data.extend_from_slice(&(samples.len() as u32 * 2).to_le_bytes());
        for s in &samples {
            data.extend_from_slice(&s.to_le_bytes());
        }
        std::fs::write(&path, data).unwrap();

        let mut src = AudioSource::open(&path).unwrap();
        assert_eq!(src.sample_rate(), rate);
        assert_eq!(src.channels(), 1);
        assert_eq!(src.total_duration(), Some(Duration::from_secs(2)));
        src.try_seek(Duration::from_secs(1)).unwrap();
        let rest = src.count();
        assert!((rest as i64 - rate as i64).abs() < 1200, "rest {rest}");
    }

    #[test]
    fn garbage_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.mp3");
        std::fs::write(&path, vec![0u8; 4096]).unwrap();
        assert!(AudioSource::open(&path).is_err());
    }
}

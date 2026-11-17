//! Steuerung für einen laufenden Titel: weich ein- und ausblenden (Überblendung) und einen schon
//! angehängten Titel wieder verwerfen (lückenlose Wiedergabe, wenn sich die Warteschlange ändert).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rodio::source::SeekError;
use rodio::Source;

/// Von der Engine aus bedienbar, während der Titel im Audio-Thread läuft.
pub struct TrackCtl {
    /// Ziel-Lautstärke 0…1 (als f32-Bits).
    target: AtomicU32,
    /// Sekunden, in denen der Weg von 0 nach 1 zurückgelegt wird.
    fade_secs: AtomicU32,
    cancelled: AtomicBool,
}

impl TrackCtl {
    pub fn new(start_gain: f32) -> Arc<Self> {
        Arc::new(Self {
            target: AtomicU32::new(start_gain.to_bits()),
            fade_secs: AtomicU32::new(0f32.to_bits()),
            cancelled: AtomicBool::new(false),
        })
    }

    /// Lautstärke in `secs` Sekunden auf `target` bringen (0 = sofort).
    pub fn fade_to(&self, target: f32, secs: f32) {
        self.fade_secs.store(secs.max(0.0).to_bits(), Ordering::Relaxed);
        self.target.store(target.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    /// Titel endet sofort (z. B. vorgeladener Titel, der nicht mehr der nächste ist).
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

/// Wendet [`TrackCtl`] auf eine Quelle an.
pub struct Controlled<S> {
    inner: S,
    ctl: Arc<TrackCtl>,
    gain: f32,
}

impl<S: Source<Item = f32>> Controlled<S> {
    pub fn new(inner: S, ctl: Arc<TrackCtl>) -> Self {
        let gain = f32::from_bits(ctl.target.load(Ordering::Relaxed));
        Self { inner, ctl, gain }
    }
}

impl<S: Source<Item = f32>> Iterator for Controlled<S> {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        if self.ctl.cancelled.load(Ordering::Relaxed) {
            return None;
        }
        let s = self.inner.next()?;
        let target = f32::from_bits(self.ctl.target.load(Ordering::Relaxed));
        if self.gain != target {
            let secs = f32::from_bits(self.ctl.fade_secs.load(Ordering::Relaxed));
            let per_sec = (self.inner.sample_rate() as f32 * self.inner.channels() as f32).max(1.0);
            let step = if secs > 0.0 { 1.0 / (secs * per_sec) } else { 1.0 };
            self.gain = if self.gain < target { (self.gain + step).min(target) } else { (self.gain - step).max(target) };
        }
        // Gleichmäßig empfundene Überblendung: Lautstärke folgt einer Viertelkreis-Kurve.
        let g = (self.gain * std::f32::consts::FRAC_PI_2).sin();
        Some(s * g)
    }
}

impl<S: Source<Item = f32>> Source for Controlled<S> {
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
        self.inner.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.inner.try_seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;

    #[test]
    fn fades_and_cancels() {
        let src = SamplesBuffer::new(1, 1000, vec![1.0f32; 3000]);
        let ctl = TrackCtl::new(0.0);
        let faded = Controlled::new(src, ctl.clone());
        // erst danach einblenden – die Quelle beginnt mit der Lautstärke beim Erzeugen
        ctl.fade_to(1.0, 1.0);
        let out: Vec<f32> = faded.collect();
        assert!(out[0] < 0.01, "beginnt leise");
        assert!(out[500] > 0.5 && out[500] < 0.8, "halb eingeblendet: {}", out[500]);
        assert!((out[1500] - 1.0).abs() < 1e-3, "danach voll");

        let ctl = TrackCtl::new(1.0);
        let mut c = Controlled::new(SamplesBuffer::new(1, 1000, vec![1.0f32; 3000]), ctl.clone());
        assert_eq!(c.next(), Some(1.0));
        ctl.cancel();
        assert_eq!(c.next(), None, "verworfen");
    }
}

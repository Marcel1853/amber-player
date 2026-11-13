//! Klanganalyse für die Visualisierung: Ringpuffer der zuletzt ausgegebenen Samples,
//! daraus Frequenzbänder (FFT) und Wellenform.

use std::sync::Mutex;

use serde::Serialize;

const RING: usize = 4096;
const FFT: usize = 2048;

pub struct Analyzer {
    ring: Mutex<Ring>,
}

struct Ring {
    buf: Vec<f32>,
    pos: usize,
    rate: u32,
}

/// Ein Bild der Visualisierung: Bänder 0–1 (tief → hoch), Wellenform −1…1, Pegel 0–1.
#[derive(Debug, Clone, Serialize, Default)]
pub struct VisualFrame {
    pub bands: Vec<f32>,
    pub wave: Vec<f32>,
    pub level: f32,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self {
            ring: Mutex::new(Ring {
                buf: vec![0.0; RING],
                pos: 0,
                rate: 44_100,
            }),
        }
    }
}

impl Analyzer {
    /// Mono-Samples aus der Wiedergabe anhängen.
    pub(crate) fn push(&self, samples: &[f32], rate: u32) {
        let mut r = self.ring.lock().unwrap_or_else(|e| e.into_inner());
        r.rate = rate;
        for &s in samples {
            let p = r.pos;
            r.buf[p] = s;
            r.pos = (p + 1) % RING;
        }
    }

    /// Die letzten `n` Samples in zeitlicher Reihenfolge.
    fn latest(&self, n: usize) -> (Vec<f32>, u32) {
        let r = self.ring.lock().unwrap_or_else(|e| e.into_inner());
        let n = n.min(RING);
        let start = (r.pos + RING - n) % RING;
        let out = (0..n).map(|i| r.buf[(start + i) % RING]).collect();
        (out, r.rate)
    }

    pub fn frame(&self, bands: usize, wave_len: usize) -> VisualFrame {
        let bands = bands.clamp(8, 256);
        let wave_len = wave_len.clamp(32, 1024);
        let (samples, rate) = self.latest(FFT);

        let level = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
        let tail = &samples[FFT - wave_len.min(FFT)..];
        let wave = tail.to_vec();

        // Hann-Fenster und FFT
        let mut re: Vec<f32> = samples
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let w = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (FFT - 1) as f32).cos();
                s * w
            })
            .collect();
        let mut im = vec![0.0f32; FFT];
        fft(&mut re, &mut im);
        let mags: Vec<f32> = (0..FFT / 2).map(|k| (re[k] * re[k] + im[k] * im[k]).sqrt()).collect();

        // Logarithmisch verteilte Bänder von 30 Hz bis 16 kHz
        let bin_hz = rate as f32 / FFT as f32;
        let (lo, hi) = (30.0f32, 16_000f32.min(rate as f32 / 2.0));
        let mut out = Vec::with_capacity(bands);
        for b in 0..bands {
            let f0 = lo * (hi / lo).powf(b as f32 / bands as f32);
            let f1 = lo * (hi / lo).powf((b + 1) as f32 / bands as f32);
            let k0 = ((f0 / bin_hz) as usize).min(mags.len() - 1);
            let k1 = ((f1 / bin_hz).ceil() as usize).clamp(k0 + 1, mags.len());
            let peak = mags[k0..k1].iter().fold(0f32, |m, &v| m.max(v));
            // Höhere Bänder etwas anheben, sonst wirken sie immer leer.
            let tilt = 1.0 + b as f32 / bands as f32 * 1.5;
            let db = 20.0 * (peak * tilt / (FFT as f32 / 4.0) + 1e-9).log10();
            // Etwa 48 dB Dynamik: leise Bänder bleiben unten, laute schlagen aus.
            out.push(((db + 48.0) / 44.0).clamp(0.0, 1.0));
        }
        VisualFrame {
            bands: out,
            wave,
            level: (level * 2.5).min(1.0),
        }
    }
}

/// Iterative Radix-2-FFT (Länge muss eine Zweierpotenz sein).
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * std::f32::consts::PI / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        for start in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0f32, 0.0f32);
            for k in 0..len / 2 {
                let (a, b) = (start + k, start + k + len / 2);
                let tr = re[b] * cr - im[b] * ci;
                let ti = re[b] * ci + im[b] * cr;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let nr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = nr;
            }
        }
        len <<= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_lands_in_matching_band() {
        let a = Analyzer::default();
        let rate = 44_100;
        let tone: Vec<f32> = (0..RING)
            .map(|i| 0.8 * (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / rate as f32).sin())
            .collect();
        a.push(&tone, rate);
        let f = a.frame(32, 256);
        let loudest = f
            .bands
            .iter()
            .enumerate()
            .max_by(|x, y| x.1.partial_cmp(y.1).unwrap())
            .unwrap()
            .0;
        // 1 kHz liegt bei 32 Bändern (30 Hz–16 kHz, logarithmisch) im Band 17.
        assert!((16..=18).contains(&loudest), "Band {loudest}");
        assert!(f.level > 0.5);
        assert_eq!(f.wave.len(), 256);
    }

    #[test]
    fn silence_is_flat() {
        let f = Analyzer::default().frame(16, 64);
        assert!(f.bands.iter().all(|&b| b == 0.0));
        assert_eq!(f.level, 0.0);
    }
}

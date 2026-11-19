//! Amber-Effekte für andere Apps (Android): Android lässt an fremde Apps nur seine eingebauten
//! Bausteine hängen (Dynamics Processing, Hall, Lautstärke-Verstärker). Darum misst Amber hier
//! seine eingeschaltete Effektkette mit einem leisen Testimpuls und gibt das Ergebnis als
//! Klangkurve (31 Terzbänder) und Balance weiter; der Hall geht mit seinen Reglern getrennt
//! raus. So wirken Equalizer, Ton und alle Filter aus Plugins automatisch mit – auch künftige.
//! Was kein Filter ist (Echo, Verzerrung, Stereobreite), fällt für andere Apps weg.

use serde::Serialize;

use super::PluginHost;

const RATE: f32 = 48_000.0;
const FRAMES: usize = 16_384;
/// Leise genug, dass Kompressoren und Verzerrer nicht eingreifen
const AMP: f32 = 0.01;
/// Messpunkte je Band
const POINTS: usize = 8;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OutsideReverb {
    pub size: f32,
    pub damp: f32,
    pub decay: f32,
    pub predelay: f32,
    pub mix: f32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OutsideProfile {
    /// Mittenfrequenzen der Bänder (Hz)
    pub centers: Vec<f32>,
    /// Obere Bandgrenzen (Hz) – so erwartet es Android
    pub cutoffs: Vec<f32>,
    /// Verstärkung je Band (dB)
    pub gains: Vec<f32>,
    /// Pegel links/rechts relativ zur Mitte (dB)
    pub balance: [f32; 2],
    /// Lautstärke-Anhebung (dB)
    pub boost: f32,
    pub reverb: Option<OutsideReverb>,
    /// Obergrenze des „Vorverstärker & Limiter“ (dB), wenn eingeschaltet
    pub ceiling: Option<f32>,
    /// Auto-Pan (z. B. 8D-Rundflug): je Eintrag [Tempo Hz, Tiefe 0…1]
    pub autopan: Vec<[f32; 2]>,
}

/// Terzbänder von 20 Hz bis 20 kHz.
pub fn band_centers() -> Vec<f32> {
    (-17..=13).map(|k| 1000.0 * 2f32.powf(k as f32 / 3.0)).collect()
}

impl PluginHost {
    /// Klangkurve, Balance, Hall und Anhebung der eingeschalteten Effekte.
    pub fn outside_profile(&self) -> OutsideProfile {
        let centers = band_centers();
        let cutoffs: Vec<f32> = centers.iter().map(|c| (c * 2f32.powf(1.0 / 6.0)).min(22_000.0)).collect();

        // Impuls durch alle eingeschalteten Effekte außer dem Hall
        let mut left = vec![0f32; FRAMES];
        let mut right = vec![0f32; FRAMES];
        left[0] = AMP;
        right[0] = AMP;
        let mut reverb = None;
        let mut ceiling = None;
        let mut autopan = Vec::new();
        for (i, values) in self.active() {
            let slot = &self.slots[i];
            autopan.extend(slot.plugin.outside_autopan(&values).into_iter().map(|(r, d)| [r, d]));
            if slot.plugin.id() == "limiter" {
                ceiling = values.get(1).copied();
            }
            if slot.plugin.id() == "reverb" {
                let v = |n: usize, d: f32| values.get(n).copied().unwrap_or(d);
                reverb = Some(OutsideReverb {
                    size: v(0, 45.0),
                    damp: v(1, 60.0),
                    decay: v(3, 1.6),
                    predelay: v(4, 20.0),
                    mix: v(5, 25.0),
                });
                continue;
            }
            let mut proc = slot.plugin.create(RATE as u32, 2);
            proc.set_params(&values);
            let mut frame = [0f32; 2];
            for n in 0..FRAMES {
                frame[0] = left[n];
                frame[1] = right[n];
                proc.process(&mut frame);
                left[n] = frame[0];
                right[n] = frame[1];
            }
        }

        // Leistung je Band (Mittel über Messpunkte und beide Kanäle)
        let mut gains = Vec::with_capacity(centers.len());
        let (mut sum_l, mut sum_r) = (0f64, 0f64);
        for &c in &centers {
            let lo = c / 2f32.powf(1.0 / 6.0);
            let hi = (c * 2f32.powf(1.0 / 6.0)).min(RATE / 2.0 - 100.0);
            let mut power = 0f64;
            for p in 0..POINTS {
                let f = lo * (hi / lo).powf((p as f32 + 0.5) / POINTS as f32);
                let pl = power_at(&left, f);
                let pr = power_at(&right, f);
                sum_l += pl;
                sum_r += pr;
                power += (pl + pr) / 2.0;
            }
            let rel = power / POINTS as f64 / (AMP as f64 * AMP as f64);
            gains.push(((10.0 * rel.max(1e-12).log10()) as f32).clamp(-40.0, 24.0));
        }
        let mean = ((sum_l + sum_r) / 2.0).max(1e-30);
        let balance = [
            ((10.0 * (sum_l.max(1e-30) / mean).log10()) as f32).clamp(-40.0, 6.0),
            ((10.0 * (sum_r.max(1e-30) / mean).log10()) as f32).clamp(-40.0, 6.0),
        ];
        OutsideProfile {
            centers,
            cutoffs,
            gains,
            balance,
            boost: self.boost_db(),
            reverb,
            ceiling,
            autopan,
        }
    }

    /// Ändert sich bei jeder Einstellung an Effekten oder Anhebung.
    pub fn settings_version(&self) -> u64 {
        self.version()
    }
}

/// |X(f)|² eines Signals (Goertzel).
fn power_at(x: &[f32], f: f32) -> f64 {
    let w = 2.0 * std::f64::consts::PI * f as f64 / RATE as f64;
    let coeff = 2.0 * w.cos();
    let (mut s1, mut s2) = (0f64, 0f64);
    for &v in x {
        let s0 = v as f64 + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    s1 * s1 + s2 * s2 - coeff * s1 * s2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_chain_is_flat_and_eq_shows_up() {
        let host = PluginHost::new(super::super::builtin_plugins(), None);
        let p = host.outside_profile();
        assert_eq!(p.gains.len(), 31);
        assert!(p.gains.iter().all(|g| g.abs() < 0.3), "ohne Effekte flach: {:?}", p.gains);
        assert!(p.reverb.is_none());

        // Bass +12 → tiefe Bänder hoch, hohe durch die Absenkung leiser
        let mut vals = vec![0.0; 10];
        vals[0] = 12.0;
        vals[1] = 12.0;
        host.set("equalizer", Some(true), Some(&vals)).unwrap();
        let p = host.outside_profile();
        let at = |hz: f32| p.gains[p.centers.iter().position(|&c| c >= hz).unwrap()];
        assert!(at(40.0) > -2.0, "Bass bleibt etwa gleich: {}", at(40.0));
        assert!(at(4000.0) < -10.0, "Höhen um ~12 dB leiser: {}", at(4000.0));

        // Balance ganz links → rechts deutlich leiser
        host.set("equalizer", Some(false), None).unwrap();
        host.set("stereo_width", Some(true), Some(&[100.0, -100.0])).unwrap();
        let p = host.outside_profile();
        assert!(p.balance[1] < p.balance[0] - 6.0, "Balance: {:?}", p.balance);
    }

    #[test]
    fn rundflug_is_partial_and_reports_autopan() {
        let host = PluginHost::new(super::super::builtin_plugins(), None);
        assert_eq!(host.get("rundflug").unwrap().outside, super::super::Outside::Partial);
        assert_eq!(host.get("equalizer").unwrap().outside, super::super::Outside::Full);
        assert!(host.outside_profile().autopan.is_empty());
        host.set("rundflug", Some(true), None).unwrap();
        let p = host.outside_profile();
        assert_eq!(p.autopan.len(), 1);
        assert!((p.autopan[0][0] - 0.15).abs() < 1e-3 && (p.autopan[0][1] - 0.8).abs() < 1e-3, "{:?}", p.autopan);
    }
}

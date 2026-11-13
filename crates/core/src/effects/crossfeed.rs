use super::{EffectPlugin, ParamInfo, Processor};

pub struct Crossfeed;

impl EffectPlugin for Crossfeed {
    fn id(&self) -> &'static str {
        "crossfeed"
    }

    fn name(&self) -> &'static str {
        "Kopfhörer-Crossfeed"
    }

    fn description(&self) -> &'static str {
        "Mischt tiefe Frequenzen leicht auf die andere Seite – entspannteres Hören mit Kopfhörern"
    }

    fn params(&self) -> Vec<ParamInfo> {
        vec![
            ParamInfo {
                id: "amount",
                name: "Stärke",
                min: 0.0,
                max: 100.0,
                default: 50.0,
                step: 5.0,
                unit: "%",
            },
            ParamInfo {
                id: "cutoff",
                name: "Grenzfrequenz",
                min: 300.0,
                max: 1500.0,
                default: 700.0,
                step: 50.0,
                unit: "Hz",
            },
        ]
    }

    fn create(&self, sample_rate: u32, _channels: u16) -> Box<dyn Processor> {
        Box::new(CrossfeedProcessor {
            rate: sample_rate as f32,
            gain: 0.0,
            coef: 0.0,
            lp: [0.0; 2],
        })
    }
}

struct CrossfeedProcessor {
    rate: f32,
    gain: f32,
    /// Koeffizient des einpoligen Tiefpasses
    coef: f32,
    lp: [f32; 2],
}

impl Processor for CrossfeedProcessor {
    fn set_params(&mut self, values: &[f32]) {
        let amount = values.first().copied().unwrap_or(50.0) / 100.0;
        let cutoff = values.get(1).copied().unwrap_or(700.0);
        // 100 % entspricht etwa −6 dB Übersprechen.
        self.gain = amount * 0.5;
        self.coef = 1.0 - (-2.0 * std::f32::consts::PI * cutoff / self.rate).exp();
    }

    fn process(&mut self, frame: &mut [f32]) {
        if let [l, r] = frame {
            self.lp[0] += self.coef * (*l - self.lp[0]);
            self.lp[1] += self.coef * (*r - self.lp[1]);
            let norm = 1.0 / (1.0 + self.gain);
            let (nl, nr) = (
                (*l + self.gain * self.lp[1]) * norm,
                (*r + self.gain * self.lp[0]) * norm,
            );
            *l = nl;
            *r = nr;
        }
    }

    fn reset(&mut self) {
        self.lp = [0.0; 2];
    }
}

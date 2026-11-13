use super::{EffectPlugin, ParamInfo, Processor};

pub struct Limiter;

impl EffectPlugin for Limiter {
    fn id(&self) -> &'static str {
        "limiter"
    }

    fn name(&self) -> &'static str {
        "Vorverstärker & Limiter"
    }

    fn description(&self) -> &'static str {
        "Lauter oder leiser machen und Spitzen abfangen, damit nichts übersteuert"
    }

    fn params(&self) -> Vec<ParamInfo> {
        vec![
            ParamInfo {
                id: "preamp",
                name: "Vorverstärkung",
                min: -12.0,
                max: 12.0,
                default: 0.0,
                step: 0.5,
                unit: "dB",
            },
            ParamInfo {
                id: "ceiling",
                name: "Obergrenze",
                min: -6.0,
                max: 0.0,
                default: -0.5,
                step: 0.1,
                unit: "dB",
            },
        ]
    }

    fn create(&self, sample_rate: u32, _channels: u16) -> Box<dyn Processor> {
        // Rückkehr zur vollen Lautstärke innerhalb von etwa 150 ms
        let release = 1.0 - (-1.0 / (0.15 * sample_rate as f32)).exp();
        Box::new(LimiterProcessor {
            preamp: 1.0,
            ceiling: 1.0,
            gain: 1.0,
            release,
        })
    }
}

struct LimiterProcessor {
    preamp: f32,
    ceiling: f32,
    gain: f32,
    release: f32,
}

fn db_to_lin(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

impl Processor for LimiterProcessor {
    fn set_params(&mut self, values: &[f32]) {
        self.preamp = db_to_lin(values.first().copied().unwrap_or(0.0));
        self.ceiling = db_to_lin(values.get(1).copied().unwrap_or(-0.5));
    }

    fn process(&mut self, frame: &mut [f32]) {
        let peak = frame.iter().fold(0f32, |m, s| m.max(s.abs())) * self.preamp;
        let target = if peak > self.ceiling {
            self.ceiling / peak
        } else {
            1.0
        };
        if target < self.gain {
            self.gain = target;
        } else {
            self.gain += (target - self.gain) * self.release;
        }
        let g = self.preamp * self.gain;
        for s in frame {
            *s *= g;
        }
    }

    fn reset(&mut self) {
        self.gain = 1.0;
    }
}

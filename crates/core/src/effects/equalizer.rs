use super::{Biquad, BiquadState, EffectPlugin, ParamInfo, Preset, Processor};

const BANDS: [(&str, &str, f32); 10] = [
    ("31", "31 Hz", 31.0),
    ("62", "62 Hz", 62.0),
    ("125", "125 Hz", 125.0),
    ("250", "250 Hz", 250.0),
    ("500", "500 Hz", 500.0),
    ("1k", "1 kHz", 1000.0),
    ("2k", "2 kHz", 2000.0),
    ("4k", "4 kHz", 4000.0),
    ("8k", "8 kHz", 8000.0),
    ("16k", "16 kHz", 16000.0),
];
const Q: f32 = 1.41;
const MAX_GAIN_DB: f32 = 12.0;

pub struct Equalizer;

impl EffectPlugin for Equalizer {
    fn id(&self) -> &'static str {
        "equalizer"
    }

    fn name(&self) -> &'static str {
        "Equalizer"
    }

    fn description(&self) -> &'static str {
        "10-Band-Equalizer von 31 Hz bis 16 kHz"
    }

    fn params(&self) -> Vec<ParamInfo> {
        BANDS
            .iter()
            .map(|&(id, name, _)| ParamInfo {
                id,
                name,
                min: -MAX_GAIN_DB,
                max: MAX_GAIN_DB,
                default: 0.0,
                step: 0.5,
                unit: "dB",
            })
            .collect()
    }

    fn presets(&self) -> Vec<Preset> {
        let p = |name, values: [f32; 10]| Preset {
            name,
            values: values.to_vec(),
        };
        vec![
            p("Neutral", [0.0; 10]),
            p("Bass", [6.0, 5.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            p("Höhen", [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 4.0, 5.0, 6.0]),
            p("Pop", [-1.0, 1.0, 3.0, 4.0, 3.0, 0.0, -1.0, -1.0, 1.0, 2.0]),
            p("Rock", [4.0, 3.0, 2.0, 0.0, -1.0, -1.0, 1.0, 3.0, 4.0, 4.0]),
            p("Elektronisch", [5.0, 4.0, 1.0, 0.0, -2.0, 1.0, 0.0, 1.0, 4.0, 5.0]),
            p("Klassik", [3.0, 2.0, 1.0, 0.0, 0.0, 0.0, -1.0, -1.0, 1.0, 2.0]),
            p("Stimme", [-3.0, -2.0, -1.0, 1.0, 3.0, 4.0, 4.0, 3.0, 1.0, -1.0]),
        ]
    }

    fn create(&self, sample_rate: u32, channels: u16) -> Box<dyn Processor> {
        Box::new(EqProcessor {
            rate: sample_rate as f32,
            filters: Vec::new(),
            states: vec![[BiquadState::default(); 10]; channels.max(1) as usize],
            preamp: 1.0,
        })
    }
}

struct EqProcessor {
    rate: f32,
    /// Nur Bänder mit Verstärkung ≠ 0 (Index ins Band, Filter).
    filters: Vec<(usize, Biquad)>,
    states: Vec<[BiquadState; 10]>,
    /// Automatische Absenkung um die stärkste Anhebung, damit der EQ nicht übersteuert.
    preamp: f32,
}

impl Processor for EqProcessor {
    fn set_params(&mut self, values: &[f32]) {
        self.filters = BANDS
            .iter()
            .zip(values)
            .enumerate()
            .filter(|(_, (_, g))| g.abs() >= 0.01)
            .map(|(i, (&(_, _, f), &g))| (i, Biquad::peaking(f, g, Q, self.rate)))
            .collect();
        let max_boost = values.iter().take(BANDS.len()).fold(0f32, |m, &g| m.max(g));
        self.preamp = 10f32.powf(-max_boost / 20.0);
    }

    fn process(&mut self, frame: &mut [f32]) {
        for (s, states) in frame.iter_mut().zip(&mut self.states) {
            *s *= self.preamp;
            for (band, filter) in &self.filters {
                *s = filter.run(&mut states[*band], *s);
            }
        }
    }

    fn reset(&mut self) {
        for s in &mut self.states {
            *s = [BiquadState::default(); 10];
        }
    }
}

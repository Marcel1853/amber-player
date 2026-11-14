//! Ton: Bass und Höhen als einfache Klangregler (Kuhschwanzfilter), wie an einer Stereoanlage.

use super::{Biquad, BiquadState, EffectPlugin, ParamInfo, Preset, Processor};

const BASS_HZ: f32 = 100.0;
const TREBLE_HZ: f32 = 8000.0;
const MAX_DB: f32 = 12.0;

pub struct Tone;

impl EffectPlugin for Tone {
    fn id(&self) -> &'static str {
        "tone"
    }

    fn name(&self) -> &'static str {
        "Ton"
    }

    fn description(&self) -> &'static str {
        "Bass und Höhen anheben oder absenken"
    }

    fn params(&self) -> Vec<ParamInfo> {
        let p = |id, name| ParamInfo { id, name, min: -MAX_DB, max: MAX_DB, default: 0.0, step: 0.5, unit: "dB" };
        vec![p("bass", "Bass"), p("treble", "Höhen")]
    }

    fn presets(&self) -> Vec<Preset> {
        let p = |name, values: [f32; 2]| Preset { name, values: values.to_vec() };
        vec![
            p("Neutral", [0.0, 0.0]),
            p("Weicher Bass", [3.0, 0.0]),
            p("Kräftiger Bass", [6.0, 1.0]),
            p("Klare Höhen", [0.0, 4.0]),
            p("Warm", [3.0, -2.0]),
            p("Lebendig", [4.0, 4.0]),
        ]
    }

    fn create(&self, sample_rate: u32, channels: u16) -> Box<dyn Processor> {
        Box::new(ToneProcessor {
            rate: sample_rate.max(8000) as f32,
            bass: Biquad::identity(),
            treble: Biquad::identity(),
            states: vec![[BiquadState::default(); 2]; channels.max(1) as usize],
            preamp: 1.0,
            active: false,
        })
    }
}

struct ToneProcessor {
    rate: f32,
    bass: Biquad,
    treble: Biquad,
    states: Vec<[BiquadState; 2]>,
    /// Absenkung um die stärkste Anhebung, damit nichts übersteuert.
    preamp: f32,
    active: bool,
}

impl Processor for ToneProcessor {
    fn set_params(&mut self, values: &[f32]) {
        let bass = values.first().copied().unwrap_or(0.0).clamp(-MAX_DB, MAX_DB);
        let treble = values.get(1).copied().unwrap_or(0.0).clamp(-MAX_DB, MAX_DB);
        self.bass = Biquad::shelf(false, BASS_HZ, bass, 0.707, self.rate);
        self.treble = Biquad::shelf(true, TREBLE_HZ.min(self.rate * 0.4), treble, 0.707, self.rate);
        self.preamp = 10f32.powf(-bass.max(treble).max(0.0) / 20.0);
        self.active = bass.abs() >= 0.01 || treble.abs() >= 0.01;
    }

    fn process(&mut self, frame: &mut [f32]) {
        if !self.active {
            return;
        }
        for (s, st) in frame.iter_mut().zip(&mut self.states) {
            let x = *s * self.preamp;
            let x = self.bass.run(&mut st[0], x);
            *s = self.treble.run(&mut st[1], x);
        }
    }

    fn reset(&mut self) {
        for s in &mut self.states {
            *s = [BiquadState::default(); 2];
        }
    }
}

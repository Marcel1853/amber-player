//! Hall nach dem bewährten Freeverb-Aufbau: acht gedämpfte Kammfilter und vier Allpässe je
//! Kanal, davor eine Vor-Verzögerung und ein Filter, der die Tiefen aus dem Hall nimmt.
//! „Mix“ regelt am Ende, wie viel Hall zum Originalklang kommt – so lässt sich ein großer,
//! langer Raum trotzdem dezent einsetzen.

use super::{Biquad, BiquadState, EffectPlugin, ParamInfo, Preset, Processor};

/// Längen bei 44,1 kHz (Freeverb).
const COMBS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASSES: [usize; 4] = [556, 441, 341, 225];
/// Versatz des rechten Kanals – macht den Hall breit.
const SPREAD: usize = 23;
const MAX_SIZE: f32 = 1.6;
const MAX_PREDELAY_MS: f32 = 200.0;

pub struct Reverb;

impl EffectPlugin for Reverb {
    fn id(&self) -> &'static str {
        "reverb"
    }

    fn name(&self) -> &'static str {
        "Hall"
    }

    fn description(&self) -> &'static str {
        "Raumklang vom kleinen Zimmer bis zur Kirche; „Mix“ regelt, wie stark der Hall zu hören ist"
    }

    fn params(&self) -> Vec<ParamInfo> {
        let p = |id, name, min, max, default, step, unit| ParamInfo { id, name, min, max, default, step, unit };
        vec![
            p("size", "Größe", 0.0, 100.0, 45.0, 1.0, "%"),
            p("damp", "Dämpfung", 0.0, 100.0, 60.0, 1.0, "%"),
            p("filter", "Filter", 20.0, 1000.0, 180.0, 10.0, "Hz"),
            p("decay", "Nachhallzeit", 0.2, 8.0, 1.6, 0.1, "s"),
            p("predelay", "Vor-Verzögerung", 0.0, MAX_PREDELAY_MS, 20.0, 1.0, "ms"),
            p("mix", "Mix", 0.0, 100.0, 25.0, 1.0, "%"),
        ]
    }

    fn presets(&self) -> Vec<Preset> {
        let p = |name, values: [f32; 6]| Preset { name, values: values.to_vec() };
        vec![
            p("Leichter Hall", [45.0, 60.0, 180.0, 1.6, 20.0, 25.0]),
            p("Zimmer", [20.0, 70.0, 200.0, 0.6, 5.0, 20.0]),
            p("Studio", [35.0, 55.0, 150.0, 1.0, 12.0, 18.0]),
            p("Saal", [70.0, 45.0, 150.0, 2.8, 30.0, 28.0]),
            p("Kirche", [95.0, 35.0, 120.0, 5.5, 45.0, 30.0]),
            p("Stadion", [100.0, 50.0, 250.0, 3.5, 90.0, 35.0]),
        ]
    }

    fn create(&self, sample_rate: u32, channels: u16) -> Box<dyn Processor> {
        let rate = sample_rate.max(8000) as f32;
        let scale = rate / 44100.0;
        let side = |offset: usize| Side {
            combs: COMBS.iter().map(|&n| Comb::new(((n + offset) as f32 * scale * MAX_SIZE) as usize + 1)).collect(),
            allpasses: ALLPASSES
                .iter()
                .map(|&n| Allpass::new(((n + offset) as f32 * scale) as usize + 1))
                .collect(),
            predelay: vec![0.0; (rate * MAX_PREDELAY_MS / 1000.0) as usize + 1],
            pre_pos: 0,
            hp: BiquadState::default(),
        };
        let sides = if channels >= 2 { vec![side(0), side(SPREAD)] } else { vec![side(0)] };
        let mut proc = ReverbProcessor {
            rate,
            scale,
            sides,
            pre_len: 0,
            highpass: Biquad::identity(),
            wet: 0.0,
            dry: 1.0,
        };
        proc.set_params(&self.params().iter().map(|p| p.default).collect::<Vec<_>>());
        Box::new(proc)
    }
}

struct Comb {
    buf: Vec<f32>,
    len: usize,
    pos: usize,
    feedback: f32,
    damp: f32,
    store: f32,
}

impl Comb {
    fn new(max: usize) -> Self {
        Self { buf: vec![0.0; max.max(1)], len: max.max(1), pos: 0, feedback: 0.0, damp: 0.0, store: 0.0 }
    }

    #[inline]
    fn run(&mut self, x: f32) -> f32 {
        let out = self.buf[self.pos];
        // Dämpfung: Tiefpass in der Rückkopplung – hohe Töne klingen schneller ab, der Hall wird weicher.
        self.store = out * (1.0 - self.damp) + self.store * self.damp;
        self.buf[self.pos] = x + self.store * self.feedback;
        self.pos += 1;
        if self.pos >= self.len {
            self.pos = 0;
        }
        out
    }
}

struct Allpass {
    buf: Vec<f32>,
    pos: usize,
}

impl Allpass {
    fn new(len: usize) -> Self {
        Self { buf: vec![0.0; len.max(1)], pos: 0 }
    }

    #[inline]
    fn run(&mut self, x: f32) -> f32 {
        let b = self.buf[self.pos];
        self.buf[self.pos] = x + b * 0.5;
        self.pos = (self.pos + 1) % self.buf.len();
        b - x
    }
}

struct Side {
    combs: Vec<Comb>,
    allpasses: Vec<Allpass>,
    predelay: Vec<f32>,
    pre_pos: usize,
    hp: BiquadState,
}

struct ReverbProcessor {
    rate: f32,
    scale: f32,
    sides: Vec<Side>,
    pre_len: usize,
    highpass: Biquad,
    wet: f32,
    dry: f32,
}

impl Processor for ReverbProcessor {
    fn set_params(&mut self, v: &[f32]) {
        let get = |i: usize, d: f32| v.get(i).copied().filter(|x| x.is_finite()).unwrap_or(d);
        let size = 0.4 + get(0, 45.0).clamp(0.0, 100.0) / 100.0 * (MAX_SIZE - 0.4);
        let damp = get(1, 60.0).clamp(0.0, 100.0) / 100.0 * 0.85;
        let filter = get(2, 180.0).clamp(20.0, 1000.0);
        let decay = get(3, 1.6).clamp(0.2, 8.0);
        let predelay = get(4, 20.0).clamp(0.0, MAX_PREDELAY_MS);
        let mix = get(5, 25.0).clamp(0.0, 100.0) / 100.0;

        for (s, side) in self.sides.iter_mut().enumerate() {
            let offset = if s == 0 { 0 } else { SPREAD };
            for (comb, &n) in side.combs.iter_mut().zip(&COMBS) {
                let len = (((n + offset) as f32 * self.scale * size) as usize).clamp(1, comb.buf.len());
                if len != comb.len {
                    comb.len = len;
                    comb.pos %= len;
                }
                // Rückkopplung aus der Nachhallzeit: nach `decay` Sekunden ist der Hall um 60 dB leiser.
                comb.feedback = 10f32.powf(-3.0 * len as f32 / (decay * self.rate)).min(0.98);
                comb.damp = damp;
            }
        }
        self.pre_len = ((predelay / 1000.0 * self.rate) as usize).min(self.sides[0].predelay.len() - 1);
        self.highpass = Biquad::pass(true, filter, 0.707, self.rate);
        // Viel Hall macht lauter – den Originalklang dabei etwas zurücknehmen.
        self.wet = mix * 5.0;
        self.dry = 1.0 - mix * 0.35;
    }

    fn process(&mut self, frame: &mut [f32]) {
        if self.wet <= 0.0 {
            return;
        }
        let input = frame.iter().take(2).sum::<f32>() / frame.len().min(2) as f32;
        let mut out = [0.0f32; 2];
        for (s, side) in self.sides.iter_mut().enumerate() {
            // Vor-Verzögerung: der Hall setzt kurz nach dem Originalklang ein.
            let x = if self.pre_len == 0 {
                input
            } else {
                let len = side.predelay.len();
                let read = (side.pre_pos + len - self.pre_len) % len;
                side.predelay[side.pre_pos] = input;
                side.pre_pos = (side.pre_pos + 1) % len;
                side.predelay[read]
            };
            let x = self.highpass.run(&mut side.hp, x) * 0.015;
            let mut acc = 0.0;
            for c in &mut side.combs {
                acc += c.run(x);
            }
            for a in &mut side.allpasses {
                acc = a.run(acc);
            }
            out[s] = acc;
        }
        match frame {
            [l, r, ..] => {
                *l = *l * self.dry + out[0] * self.wet;
                *r = *r * self.dry + out[1] * self.wet;
            }
            [m] => *m = *m * self.dry + out[0] * self.wet,
            [] => {}
        }
    }

    fn reset(&mut self) {
        for side in &mut self.sides {
            for c in &mut side.combs {
                c.buf.fill(0.0);
                c.store = 0.0;
            }
            for a in &mut side.allpasses {
                a.buf.fill(0.0);
            }
            side.predelay.fill(0.0);
            side.hp = BiquadState::default();
        }
    }
}

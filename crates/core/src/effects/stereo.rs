use super::{EffectPlugin, ParamInfo, Preset, Processor};

pub struct StereoWidth;

impl EffectPlugin for StereoWidth {
    fn id(&self) -> &'static str {
        "stereo_width"
    }

    fn name(&self) -> &'static str {
        "Stereo & Balance"
    }

    fn description(&self) -> &'static str {
        "Stereobild schmaler (0 % = Mono) oder breiter (bis 200 %), dazu die Balance zwischen links und rechts"
    }

    fn params(&self) -> Vec<ParamInfo> {
        vec![
            ParamInfo { id: "width", name: "Breite", min: 0.0, max: 200.0, default: 100.0, step: 5.0, unit: "%" },
            ParamInfo { id: "balance", name: "Balance", min: -100.0, max: 100.0, default: 0.0, step: 1.0, unit: "%" },
        ]
    }

    fn presets(&self) -> Vec<Preset> {
        let p = |name, values: [f32; 2]| Preset { name, values: values.to_vec() };
        vec![p("Normal", [100.0, 0.0]), p("Mono", [0.0, 0.0]), p("Breit", [150.0, 0.0])]
    }

    fn create(&self, _sample_rate: u32, _channels: u16) -> Box<dyn Processor> {
        Box::new(WidthProcessor { width: 1.0, left: 1.0, right: 1.0 })
    }
}

struct WidthProcessor {
    width: f32,
    left: f32,
    right: f32,
}

impl Processor for WidthProcessor {
    fn set_params(&mut self, values: &[f32]) {
        self.width = values.first().copied().unwrap_or(100.0) / 100.0;
        // Balance: die Gegenseite wird leiser, die Mitte bleibt bei voller Lautstärke.
        let b = values.get(1).copied().unwrap_or(0.0).clamp(-100.0, 100.0) / 100.0;
        self.left = (1.0 - b).min(1.0);
        self.right = (1.0 + b).min(1.0);
    }

    fn process(&mut self, frame: &mut [f32]) {
        if let [l, r] = frame {
            let mid = (*l + *r) * 0.5;
            let side = (*l - *r) * 0.5 * self.width;
            *l = (mid + side) * self.left;
            *r = (mid - side) * self.right;
        }
    }

    fn reset(&mut self) {}
}

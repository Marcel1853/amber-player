use super::{EffectPlugin, ParamInfo, Processor};

pub struct StereoWidth;

impl EffectPlugin for StereoWidth {
    fn id(&self) -> &'static str {
        "stereo_width"
    }

    fn name(&self) -> &'static str {
        "Stereobreite"
    }

    fn description(&self) -> &'static str {
        "Macht das Stereobild schmaler (0 % = Mono) oder breiter (bis 200 %)"
    }

    fn params(&self) -> Vec<ParamInfo> {
        vec![ParamInfo {
            id: "width",
            name: "Breite",
            min: 0.0,
            max: 200.0,
            default: 100.0,
            step: 5.0,
            unit: "%",
        }]
    }

    fn create(&self, _sample_rate: u32, _channels: u16) -> Box<dyn Processor> {
        Box::new(WidthProcessor { width: 1.0 })
    }
}

struct WidthProcessor {
    width: f32,
}

impl Processor for WidthProcessor {
    fn set_params(&mut self, values: &[f32]) {
        self.width = values.first().copied().unwrap_or(100.0) / 100.0;
    }

    fn process(&mut self, frame: &mut [f32]) {
        if let [l, r] = frame {
            let mid = (*l + *r) * 0.5;
            let side = (*l - *r) * 0.5 * self.width;
            *l = mid + side;
            *r = mid - side;
        }
    }

    fn reset(&mut self) {}
}

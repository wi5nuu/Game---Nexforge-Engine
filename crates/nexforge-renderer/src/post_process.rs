#![deny(clippy::all)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToneMapping {
    None,
    ACES,
    Reinhard,
    Unreal,
}

impl ToneMapping {
    pub fn apply(&self, color: [f32; 3]) -> [f32; 3] {
        match self {
            ToneMapping::None => color,
            ToneMapping::ACES => {
                let a = 2.51;
                let b = 0.03;
                let c = 2.43;
                let d = 0.59;
                let e = 0.14;
                [
                    (color[0] * (a * color[0] + b)) / (color[0] * (c * color[0] + d) + e),
                    (color[1] * (a * color[1] + b)) / (color[1] * (c * color[1] + d) + e),
                    (color[2] * (a * color[2] + b)) / (color[2] * (c * color[2] + d) + e),
                ]
            }
            ToneMapping::Reinhard => [
                color[0] / (1.0 + color[0]),
                color[1] / (1.0 + color[1]),
                color[2] / (1.0 + color[2]),
            ],
            ToneMapping::Unreal => [
                color[0] / (color[0] * 0.15 + 0.015 + 0.05),
                color[1] / (color[1] * 0.15 + 0.015 + 0.05),
                color[2] / (color[2] * 0.15 + 0.015 + 0.05),
            ],
        }
    }
}

pub struct BloomPass {
    pub intensity: f32,
    pub threshold: f32,
    pub radius: f32,
    pub mip_count: u32,
}

impl BloomPass {
    pub fn new() -> Self {
        Self {
            intensity: 0.5,
            threshold: 1.0,
            radius: 0.005,
            mip_count: 5,
        }
    }
    pub fn apply(&self, color: [f32; 3]) -> [f32; 3] {
        let luminance = 0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2];
        if luminance > self.threshold {
            let bloom = (luminance - self.threshold) / luminance.max(0.001);
            [
                color[0] + color[0] * bloom * self.intensity,
                color[1] + color[1] * bloom * self.intensity,
                color[2] + color[2] * bloom * self.intensity,
            ]
        } else {
            color
        }
    }
}

impl Default for BloomPass {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SSAOPass {
    pub enabled: bool,
    pub radius: f32,
    pub bias: f32,
    pub power: f32,
    pub sample_count: u32,
}

impl SSAOPass {
    pub fn new() -> Self {
        Self {
            enabled: true,
            radius: 0.5,
            bias: 0.025,
            power: 1.0,
            sample_count: 16,
        }
    }
}

impl Default for SSAOPass {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MotionBlurPass {
    pub enabled: bool,
    pub samples: u32,
    pub strength: f32,
}

impl MotionBlurPass {
    pub fn new() -> Self {
        Self {
            enabled: false,
            samples: 8,
            strength: 0.5,
        }
    }
}

impl Default for MotionBlurPass {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PostProcessStack {
    pub bloom: BloomPass,
    pub ssao: SSAOPass,
    pub tone_mapping: ToneMapping,
    pub motion_blur: MotionBlurPass,
}

impl PostProcessStack {
    pub fn new() -> Self {
        Self {
            bloom: BloomPass::new(),
            ssao: SSAOPass::new(),
            tone_mapping: ToneMapping::ACES,
            motion_blur: MotionBlurPass::new(),
        }
    }

    pub fn set_tonemapping(&mut self, tm: ToneMapping) {
        self.tone_mapping = tm;
    }

    pub fn set_bloom_intensity(&mut self, intensity: f32) {
        self.bloom.intensity = intensity;
    }

    pub fn apply(&self, color: [f32; 3]) -> [f32; 3] {
        let mut c = color;
        c = self.bloom.apply(c);
        c = self.tone_mapping.apply(c);
        c
    }
}

impl Default for PostProcessStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_post_process() {
        let pp = PostProcessStack::default();
        assert!(pp.ssao.enabled);
        assert!(!pp.motion_blur.enabled);
        assert_eq!(pp.tone_mapping, ToneMapping::ACES);
    }

    #[test]
    fn test_aces_tone_mapping() {
        let result = ToneMapping::ACES.apply([1.0, 0.5, 0.2]);
        assert!(result[0] > 0.0);
        assert!(result[0] <= 1.0);
    }

    #[test]
    fn test_bloom_apply() {
        let bloom = BloomPass::new();
        let result = bloom.apply([3.0, 3.0, 3.0]);
        assert!(result[0] > 3.0); // Bright pixel gets bloom
    }

    #[test]
    fn test_set_tonemapping() {
        let mut pp = PostProcessStack::new();
        pp.set_tonemapping(ToneMapping::Reinhard);
        assert_eq!(pp.tone_mapping, ToneMapping::Reinhard);
    }

    #[test]
    fn test_set_bloom_intensity() {
        let mut pp = PostProcessStack::new();
        pp.set_bloom_intensity(0.8);
        assert!((pp.bloom.intensity - 0.8).abs() < f32::EPSILON);
    }
}

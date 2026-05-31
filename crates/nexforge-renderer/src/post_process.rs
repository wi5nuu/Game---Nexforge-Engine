#![deny(clippy::all)]

pub struct PostProcessStack {
    pub bloom_intensity: f32,
    pub ssao_enabled: bool,
    pub ssao_radius: f32,
    pub tone_mapping: ToneMapping,
    pub motion_blur_enabled: bool,
    pub motion_blur_samples: u32,
}

pub enum ToneMapping {
    None,
    ACES,
    Reinhard,
    Unreal,
}

impl PostProcessStack {
    pub fn new() -> Self {
        Self {
            bloom_intensity: 0.5,
            ssao_enabled: true,
            ssao_radius: 0.5,
            tone_mapping: ToneMapping::ACES,
            motion_blur_enabled: false,
            motion_blur_samples: 8,
        }
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
        assert!(pp.ssao_enabled);
        assert!(!pp.motion_blur_enabled);
    }
}

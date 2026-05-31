#![deny(clippy::all)]

pub struct ShadowMap {
    pub resolution: u32,
    pub cascade_count: u32,
}

impl ShadowMap {
    pub fn new(resolution: u32, cascade_count: u32) -> Self {
        Self {
            resolution,
            cascade_count,
        }
    }
}

impl Default for ShadowMap {
    fn default() -> Self {
        Self {
            resolution: 2048,
            cascade_count: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shadow_map() {
        let shadow = ShadowMap::default();
        assert_eq!(shadow.resolution, 2048);
        assert_eq!(shadow.cascade_count, 4);
    }
}

#![deny(clippy::all)]

pub struct Cascade {
    pub split_depth: f32,
    pub view_proj: [[f32; 4]; 4],
    pub texture_size: u32,
}

pub struct ShadowMap {
    pub resolution: u32,
    pub cascade_count: u32,
    pub split_lambda: f32,
    pub cascades: Vec<Cascade>,
    pub bias: f32,
    pub normal_bias: f32,
}

impl ShadowMap {
    pub fn new(resolution: u32, cascade_count: u32) -> Self {
        let mut cascades = Vec::with_capacity(cascade_count as usize);
        for i in 0..cascade_count {
            cascades.push(Cascade {
                split_depth: (i + 1) as f32 / cascade_count as f32,
                view_proj: [[0.0; 4]; 4],
                texture_size: resolution,
            });
        }
        Self { resolution, cascade_count, split_lambda: 0.5, cascades, bias: 0.005, normal_bias: 0.02 }
    }

    pub fn set_resolution(&mut self, res: u32) { self.resolution = res; }

    pub fn set_split_lambda(&mut self, lambda: f32) { self.split_lambda = lambda.clamp(0.0, 1.0); }

    pub fn compute_splits(&mut self, near: f32, far: f32) {
        for (i, cascade) in self.cascades.iter_mut().enumerate() {
            let i = i as f32;
            let log_split = near * (far / near).powf((i + 1.0) / self.cascade_count as f32);
            let uniform_split = near + (far - near) * (i + 1.0) / self.cascade_count as f32;
            cascade.split_depth = self.split_lambda * log_split + (1.0 - self.split_lambda) * uniform_split;
        }
    }
}

impl Default for ShadowMap {
    fn default() -> Self { Self::new(2048, 4) }
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

    #[test]
    fn test_split_computation() {
        let mut shadow = ShadowMap::new(1024, 3);
        shadow.compute_splits(0.1, 100.0);
        assert_eq!(shadow.cascades.len(), 3);
    }

    #[test]
    fn test_shadow_setters() {
        let mut shadow = ShadowMap::new(1024, 3);
        shadow.set_resolution(4096);
        assert_eq!(shadow.resolution, 4096);
        shadow.set_split_lambda(0.7);
        assert!((shadow.split_lambda - 0.7).abs() < f32::EPSILON);
    }
}

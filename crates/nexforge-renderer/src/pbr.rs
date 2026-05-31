#![deny(clippy::all)]

pub struct PbrMaterial {
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
    pub normal_scale: f32,
}

impl PbrMaterial {
    pub fn new(albedo: [f32; 4], metallic: f32, roughness: f32) -> Self {
        Self {
            albedo,
            metallic,
            roughness,
            ao: 1.0,
            normal_scale: 1.0,
        }
    }
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            albedo: [0.5, 0.5, 0.5, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            ao: 1.0,
            normal_scale: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_material() {
        let mat = PbrMaterial::default();
        assert_eq!(mat.metallic, 0.0);
        assert_eq!(mat.roughness, 0.5);
    }

    #[test]
    fn test_custom_material() {
        let mat = PbrMaterial::new([1.0, 0.0, 0.0, 1.0], 0.8, 0.2);
        assert!((mat.metallic - 0.8).abs() < f32::EPSILON);
    }
}

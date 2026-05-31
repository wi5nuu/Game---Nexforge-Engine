#![deny(clippy::all)]

pub struct PbrMaterial {
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
    pub emissive: [f32; 3],
    pub emissive_strength: f32,
    pub normal_scale: f32,
}

impl PbrMaterial {
    pub fn new(albedo: [f32; 4], metallic: f32, roughness: f32) -> Self {
        Self { albedo, metallic, roughness, ao: 1.0, emissive: [0.0; 3], emissive_strength: 0.0, normal_scale: 1.0 }
    }

    pub fn to_gpu_bytes(&self) -> [f32; 16] {
        [
            self.albedo[0], self.albedo[1], self.albedo[2], self.albedo[3],
            self.metallic, self.roughness, self.ao, self.normal_scale,
            self.emissive[0], self.emissive[1], self.emissive[2], self.emissive_strength,
            0.0, 0.0, 0.0, 0.0,
        ]
    }
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self { albedo: [0.5, 0.5, 0.5, 1.0], metallic: 0.0, roughness: 0.5, ao: 1.0, emissive: [0.0; 3], emissive_strength: 0.0, normal_scale: 1.0 }
    }
}

pub struct PbrLight {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
}

impl PbrLight {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        Self { position, color, intensity, radius: 50.0 }
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

    #[test]
    fn test_gpu_bytes() {
        let mat = PbrMaterial::default();
        let bytes = mat.to_gpu_bytes();
        assert_eq!(bytes.len(), 16);
        assert!((bytes[0] - 0.5).abs() < f32::EPSILON);
    }
}

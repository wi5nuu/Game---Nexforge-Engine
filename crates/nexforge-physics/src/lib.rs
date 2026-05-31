#![deny(clippy::all)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PhysicsError {
    #[error("Rapier3d integration error")]
    RapierError,
}

pub struct PhysicsEngine {
    pub gravity: [f32; 3],
    initialized: bool,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            gravity: [0.0, -9.81, 0.0],
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), PhysicsError> {
        self.initialized = true;
        Ok(())
    }

    pub fn step(&mut self, _dt: f32) {
        // Placeholder
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_init() {
        let mut engine = PhysicsEngine::new();
        assert!(!engine.is_initialized());
        assert!(engine.initialize().is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_default_gravity() {
        let engine = PhysicsEngine::new();
        assert_eq!(engine.gravity[1], -9.81);
    }
}

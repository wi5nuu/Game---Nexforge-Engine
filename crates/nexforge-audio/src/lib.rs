#![deny(clippy::all)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("cpal device error: {0}")]
    DeviceError(String),
    #[error("Audio stream error")]
    StreamError,
}

pub struct AudioEngine {
    pub master_volume: f32,
    initialized: bool,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            master_volume: 1.0,
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), AudioError> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_init() {
        let mut engine = AudioEngine::new();
        assert!(!engine.is_initialized());
        assert!(engine.initialize().is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_default_volume() {
        let engine = AudioEngine::new();
        assert!((engine.master_volume - 1.0).abs() < f32::EPSILON);
    }
}

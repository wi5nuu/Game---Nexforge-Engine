#![deny(clippy::all)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Renderer initialization failed: {0}")]
    RendererError(String),
    #[error("Physics initialization failed: {0}")]
    PhysicsError(String),
    #[error("Audio initialization failed: {0}")]
    AudioError(String),
    #[error("AI initialization failed: {0}")]
    AiError(String),
    #[error("Network initialization failed: {0}")]
    NetError(String),
    #[error("Window creation failed")]
    WindowError,
}

pub struct Engine {
    pub running: bool,
    pub frame_count: u64,
    pub fixed_timestep: f64,
    initialized: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            running: false,
            frame_count: 0,
            fixed_timestep: 1.0 / 60.0,
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), EngineError> {
        log::info!("Nexforge Engine v{}", env!("CARGO_PKG_VERSION"));
        self.initialized = true;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), EngineError> {
        self.running = true;
        log::info!("Engine entering main loop");
        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.running = false;
        log::info!("Engine shutdown complete");
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn frame_time(&self) -> f64 {
        self.fixed_timestep
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new();
        assert!(!engine.is_initialized());
        assert_eq!(engine.frame_count, 0);
    }

    #[test]
    fn test_engine_initialize() {
        let mut engine = Engine::new();
        assert!(engine.initialize().is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_engine_defaults() {
        let engine = Engine::new();
        assert!((engine.fixed_timestep - 1.0 / 60.0).abs() < f64::EPSILON);
    }
}

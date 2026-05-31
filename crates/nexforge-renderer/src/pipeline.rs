#![deny(clippy::all)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to create WGPU device")]
    DeviceCreationFailed,
    #[error("Shader compilation error: {0}")]
    ShaderCompilation(String),
    #[error("Surface error: {0}")]
    SurfaceError(String),
}

pub struct RenderPipeline {
    initialized: bool,
}

impl RenderPipeline {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), RenderError> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_initialization() {
        let mut pipeline = RenderPipeline::new();
        assert!(!pipeline.is_initialized());
        assert!(pipeline.initialize().is_ok());
        assert!(pipeline.is_initialized());
    }
}

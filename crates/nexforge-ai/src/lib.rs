#![deny(clippy::all)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("Path not found")]
    PathNotFound,
    #[error("Invalid behavior tree node")]
    InvalidNode,
}

pub struct AiEngine {
    initialized: bool,
}

impl AiEngine {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), AiError> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for AiEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_init() {
        let mut engine = AiEngine::new();
        assert!(!engine.is_initialized());
        assert!(engine.initialize().is_ok());
        assert!(engine.is_initialized());
    }
}

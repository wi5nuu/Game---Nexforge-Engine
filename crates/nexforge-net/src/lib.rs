#![deny(clippy::all)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Rollback error: frame mismatch")]
    RollbackMismatch,
}

pub struct NetEngine {
    initialized: bool,
}

impl NetEngine {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), NetError> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for NetEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_init() {
        let mut engine = NetEngine::new();
        assert!(!engine.is_initialized());
        assert!(engine.initialize().is_ok());
        assert!(engine.is_initialized());
    }
}

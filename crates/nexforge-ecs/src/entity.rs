#![deny(clippy::all)]

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub index: u32,
    pub generation: u32,
}

static NEXT_ENTITY_ID: AtomicU64 = AtomicU64::new(1);

impl Entity {
    pub fn new() -> Self {
        let id = NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            index: (id & 0xFFFF_FFFF) as u32,
            generation: ((id >> 32) & 0xFFFF_FFFF) as u32,
        }
    }

    pub fn is_null(&self) -> bool {
        self.index == 0 && self.generation == 0
    }

    pub fn null() -> Self {
        Self {
            index: 0,
            generation: 0,
        }
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let e1 = Entity::new();
        let e2 = Entity::new();
        assert_ne!(e1, e2);
    }

    #[test]
    fn test_entity_unique_indices() {
        let mut ids = std::collections::HashSet::new();
        for _ in 0..1000 {
            ids.insert(Entity::new());
        }
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn test_null_entity() {
        let null = Entity::null();
        assert!(null.is_null());
    }
}

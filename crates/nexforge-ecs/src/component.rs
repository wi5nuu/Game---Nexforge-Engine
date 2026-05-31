#![deny(clippy::all)]

use std::any::TypeId;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub usize);

pub struct ComponentRegistry {
    types: HashMap<TypeId, ComponentId>,
    next_id: usize,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn register<T: 'static>(&mut self) -> ComponentId {
        let type_id = TypeId::of::<T>();
        *self.types.entry(type_id).or_insert_with(|| {
            let id = ComponentId(self.next_id);
            self.next_id += 1;
            id
        })
    }

    pub fn id_of<T: 'static>(&self) -> Option<ComponentId> {
        self.types.get(&TypeId::of::<T>()).copied()
    }

    pub fn count(&self) -> usize {
        self.next_id
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Pos {
        x: f32,
        y: f32,
    }
    struct Vel {
        x: f32,
        y: f32,
    }

    #[test]
    fn test_component_registration() {
        let mut reg = ComponentRegistry::new();
        let pos_id = reg.register::<Pos>();
        let vel_id = reg.register::<Vel>();
        assert_ne!(pos_id, vel_id);
        assert_eq!(reg.id_of::<Pos>(), Some(pos_id));
        assert_eq!(reg.id_of::<Vel>(), Some(vel_id));
    }

    #[test]
    fn test_component_count() {
        let mut reg = ComponentRegistry::new();
        assert_eq!(reg.count(), 0);
        reg.register::<Pos>();
        assert_eq!(reg.count(), 1);
        reg.register::<Vel>();
        assert_eq!(reg.count(), 2);
    }
}

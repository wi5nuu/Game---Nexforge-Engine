#![deny(clippy::all)]

use crate::component::ComponentRegistry;
use crate::entity::Entity;
use std::collections::HashMap;

pub struct World {
    entities: Vec<Entity>,
    component_registry: ComponentRegistry,
    archetypes: HashMap<usize, Archetype>,
}

struct Archetype {
    id: usize,
    columns: Vec<Vec<u8>>,
    entities: Vec<Entity>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            component_registry: ComponentRegistry::new(),
            archetypes: HashMap::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new();
        self.entities.push(entity);
        entity
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.entities.retain(|e| *e != entity);
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn registry(&self) -> &ComponentRegistry {
        &self.component_registry
    }

    pub fn registry_mut(&mut self) -> &mut ComponentRegistry {
        &mut self.component_registry
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_entity() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(!e.is_null());
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_despawn_entity() {
        let mut world = World::new();
        let e = world.spawn();
        assert_eq!(world.entity_count(), 1);
        world.despawn(e);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_multiple_entities() {
        let mut world = World::new();
        let _e1 = world.spawn();
        let _e2 = world.spawn();
        let _e3 = world.spawn();
        assert_eq!(world.entity_count(), 3);
    }
}

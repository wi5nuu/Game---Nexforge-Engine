#![deny(clippy::all)]

use crate::component::ComponentId;
use crate::entity::Entity;

pub struct Query {
    entities: Vec<Entity>,
}

impl Query {
    pub fn new(entities: Vec<Entity>) -> Self {
        Self { entities }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    struct Pos { x: f32, y: f32 }

    #[test]
    fn test_empty_query() {
        let query = Query::new(vec![]);
        assert!(query.is_empty());
    }

    #[test]
    fn test_query_with_entities() {
        let e1 = Entity::new();
        let e2 = Entity::new();
        let query = Query::new(vec![e1, e2]);
        assert_eq!(query.len(), 2);
    }

    #[test]
    fn test_query_from_world() {
        let mut world = World::new();
        let e1 = world.spawn(Pos { x: 1.0, y: 2.0 });
        let e2 = world.spawn(Pos { x: 3.0, y: 4.0 });
        let cid = world.registry().resolve::<Pos>();
        let entities = world.query_entities(&[cid]);
        let query = Query::new(entities);
        assert_eq!(query.len(), 2);
    }
}

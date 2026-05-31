#![deny(clippy::all)]

use crate::entity::Entity;

pub struct Query<'a, T> {
    _marker: std::marker::PhantomData<&'a T>,
    entities: Vec<Entity>,
}

impl<'a, T> Query<'a, T> {
    pub fn new(entities: Vec<Entity>) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            entities,
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Entity;

    #[test]
    fn test_empty_query() {
        let query: Query<()> = Query::new(vec![]);
        assert!(query.is_empty());
    }

    #[test]
    fn test_query_with_entities() {
        let e1 = Entity::new();
        let e2 = Entity::new();
        let query: Query<()> = Query::new(vec![e1, e2]);
        assert_eq!(query.len(), 2);
    }
}

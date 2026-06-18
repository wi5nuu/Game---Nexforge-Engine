#![deny(clippy::all)]

pub mod world;
pub mod entity;
pub mod component;
pub mod query;
pub mod scheduler;

pub use world::World;
pub use world::CommandBuffer;
pub use world::Archetype;
pub use world::Column;
pub use entity::Entity;
pub use component::Component;
pub use component::ComponentId;
pub use component::ComponentRegistry;
pub use component::ComponentInfo;
pub use query::Query;
pub use scheduler::SystemScheduler;
pub use scheduler::System;

#[cfg(test)]
mod tests {
    use super::*;

    struct Pos { x: f32, y: f32 }

    #[test]
    fn test_world_creation_with_entities() {
        let mut world = World::new();
        let e1 = world.spawn(Pos { x: 1.0, y: 2.0 });
        let e2 = world.spawn(Pos { x: 3.0, y: 4.0 });
        assert_eq!(world.entity_count(), 2);
        assert!(world.has_component::<Pos>(e1));
        assert!(world.has_component::<Pos>(e2));
    }
}

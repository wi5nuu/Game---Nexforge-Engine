#![deny(clippy::all)]

pub mod component;
pub mod entity;
pub mod query;
pub mod scheduler;
pub mod world;

pub use component::Component;
pub use component::ComponentId;
pub use component::ComponentInfo;
pub use component::ComponentRegistry;
pub use entity::Entity;
pub use query::Query;
pub use scheduler::System;
pub use scheduler::SystemScheduler;
pub use world::Archetype;
pub use world::Column;
pub use world::CommandBuffer;
pub use world::World;

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    struct Pos {
        x: f32,
        y: f32,
    }

    #[allow(dead_code)]
    struct Vel {
        x: f32,
        y: f32,
    }

    #[test]
    fn test_world_creation_with_entities() {
        let mut world = World::new();
        let e1 = world.spawn(Pos { x: 1.0, y: 2.0 });
        let e2 = world.spawn(Pos { x: 3.0, y: 4.0 });
        assert_eq!(world.entity_count(), 2);
        assert!(world.has_component::<Pos>(e1));
        assert!(world.has_component::<Pos>(e2));
    }

    #[test]
    fn test_component_add_get() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 1.0, y: 2.0 });
        world.add_component(e, Vel { x: 3.0, y: 4.0 });
        let vel = world.get_component::<Vel>(e).unwrap();
        assert_eq!(vel.x, 3.0);
        assert_eq!(vel.y, 4.0);
    }

    #[test]
    fn test_component_removal() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 1.0, y: 2.0 });
        world.add_component(e, Vel { x: 3.0, y: 4.0 });
        assert!(world.has_component::<Vel>(e));
        let cid = world.registry().resolve::<Vel>();
        world.command_buffer().remove_component(e, cid);
        world.flush();
        assert!(!world.has_component::<Vel>(e));
    }

    #[test]
    fn test_entity_destruction() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 1.0, y: 2.0 });
        assert_eq!(world.entity_count(), 1);
        world.despawn(e);
        assert_eq!(world.entity_count(), 0);
        assert!(!world.has_component::<Pos>(e));
    }

    #[test]
    fn test_system_execution() {
        let mut world = World::new();
        let mut scheduler = SystemScheduler::new();
        scheduler.add_system(|w: &mut World| {
            w.spawn(Pos { x: 42.0, y: 0.0 });
        });
        scheduler.run(&mut world);
        assert_eq!(world.entity_count(), 1);
        let pos = world.query_entities(&[world.registry().resolve::<Pos>()]);
        assert_eq!(pos.len(), 1);
    }
}

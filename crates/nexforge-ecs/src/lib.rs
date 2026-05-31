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

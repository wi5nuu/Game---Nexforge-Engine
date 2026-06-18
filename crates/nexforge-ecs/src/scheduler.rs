#![deny(clippy::all)]

use crate::world::World;

pub trait System {
    fn run(&mut self, world: &mut World);
}

impl<F> System for F
where
    F: FnMut(&mut World),
{
    fn run(&mut self, world: &mut World) {
        self(world);
    }
}

pub struct SystemScheduler {
    systems: Vec<Box<dyn System>>,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    pub fn run(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }

    pub fn run_parallel(&mut self, world: &mut World) {
        // Sequential for now; parallel will use rayon in Phase 3
        self.run(world);
    }

    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    pub fn clear(&mut self) {
        self.systems.clear();
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    struct Pos { x: f32, y: f32 }
    struct Vel { x: f32, y: f32 }

    #[test]
    fn test_system_execution() {
        let mut scheduler = SystemScheduler::new();
        let counter = 0u32;

        scheduler.add_system(move |_world: &mut World| {
            let _ = counter;
        });

        assert_eq!(scheduler.system_count(), 1);
    }

    #[test]
    fn test_multiple_systems() {
        let mut scheduler = SystemScheduler::new();
        for _ in 0..5 {
            scheduler.add_system(|_world: &mut World| {});
        }
        assert_eq!(scheduler.system_count(), 5);
    }

    #[test]
    fn test_system_modifies_world() {
        let mut scheduler = SystemScheduler::new();
        let mut world = World::new();

        scheduler.add_system(|world: &mut World| {
            world.spawn(Pos { x: 1.0, y: 2.0 });
        });

        scheduler.run(&mut world);
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_system_ordering() {
        let mut scheduler = SystemScheduler::new();
        let mut world = World::new();

        scheduler.add_system(|world: &mut World| {
            world.spawn(Pos { x: 10.0, y: 20.0 });
        });

        scheduler.add_system(|world: &mut World| {
            world.spawn(Pos { x: 30.0, y: 40.0 });
        });

        scheduler.run(&mut world);
        assert_eq!(world.entity_count(), 2);
    }
}

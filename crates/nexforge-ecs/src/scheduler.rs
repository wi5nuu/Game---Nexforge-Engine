#![deny(clippy::all)]

pub struct SystemScheduler {
    systems: Vec<Box<dyn FnMut()>>,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self { systems: Vec::new() }
    }

    pub fn add_system<F>(&mut self, system: F)
    where
        F: 'static + FnMut(),
    {
        self.systems.push(Box::new(system));
    }

    pub fn run(&mut self) {
        for system in &mut self.systems {
            (system)();
        }
    }

    pub fn system_count(&self) -> usize {
        self.systems.len()
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_system_execution() {
        let mut scheduler = SystemScheduler::new();
        let counter = AtomicUsize::new(0);

        scheduler.add_system(|| {
            counter.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(scheduler.system_count(), 1);
        scheduler.run();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_systems() {
        let mut scheduler = SystemScheduler::new();
        let counter = AtomicUsize::new(0);

        for _ in 0..5 {
            let c = &counter;
            scheduler.add_system(move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }

        assert_eq!(scheduler.system_count(), 5);
        scheduler.run();
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }
}

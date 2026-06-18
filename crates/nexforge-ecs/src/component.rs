#![deny(clippy::all)]

use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub usize);

pub trait Component: 'static + Send + Sync {}

impl<T: 'static + Send + Sync> Component for T {}

pub struct ComponentInfo {
    pub id: ComponentId,
    pub type_name: &'static str,
    pub size: usize,
    pub align: usize,
}

pub struct ComponentRegistry {
    types: HashMap<TypeId, ComponentId>,
    info: Vec<ComponentInfo>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            info: Vec::new(),
        }
    }

    pub fn register<T: 'static>(&mut self) -> ComponentId {
        let type_id = TypeId::of::<T>();
        if let Some(&id) = self.types.get(&type_id) {
            return id;
        }
        let id = ComponentId(self.info.len());
        self.types.insert(type_id, id);
        self.info.push(ComponentInfo {
            id,
            type_name: std::any::type_name::<T>(),
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
        });
        id
    }

    pub fn id_of<T: 'static>(&self) -> Option<ComponentId> {
        self.types.get(&TypeId::of::<T>()).copied()
    }

    pub fn resolve<T: 'static>(&self) -> ComponentId {
        self.id_of::<T>()
            .unwrap_or_else(|| panic!("Component {} not registered", std::any::type_name::<T>()))
    }

    pub fn info(&self, id: ComponentId) -> &ComponentInfo {
        &self.info[id.0]
    }

    pub fn count(&self) -> usize {
        self.info.len()
    }

    thread_local! {
        static GLOBAL: RefCell<ComponentRegistry> = RefCell::new(ComponentRegistry::new());
    }

    pub fn with_global<F, R>(f: F) -> R
    where
        F: FnOnce(&ComponentRegistry) -> R,
    {
        Self::GLOBAL.with(|r| f(&r.borrow()))
    }

    pub fn with_global_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut ComponentRegistry) -> R,
    {
        Self::GLOBAL.with(|r| f(&mut r.borrow_mut()))
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

    #[test]
    fn test_duplicate_registration() {
        let mut reg = ComponentRegistry::new();
        let id1 = reg.register::<Pos>();
        let id2 = reg.register::<Pos>();
        assert_eq!(id1, id2);
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn test_component_info() {
        let mut reg = ComponentRegistry::new();
        let id = reg.register::<Pos>();
        let info = reg.info(id);
        assert_eq!(info.size, std::mem::size_of::<Pos>());
    }
}

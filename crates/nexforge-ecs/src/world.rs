#![deny(clippy::all)]

use crate::component::{Component, ComponentId, ComponentRegistry};
use crate::entity::Entity;
use std::any::Any;
use std::collections::HashMap;

type ArchetypeKey = Vec<ComponentId>;

#[derive(Clone)]
pub struct Column {
    pub component_id: ComponentId,
    pub data: Vec<u8>,
    pub stride: usize,
}

impl Column {
    pub fn new(component_id: ComponentId, stride: usize) -> Self {
        Self {
            component_id,
            data: Vec::new(),
            stride,
        }
    }

    pub fn push<T: Component>(&mut self, value: &T) {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                (value as *const T) as *const u8,
                std::mem::size_of::<T>(),
            )
        };
        self.data.extend_from_slice(bytes);
    }

    pub fn get<T: Component>(&self, index: usize) -> &T {
        let offset = index * self.stride;
        unsafe { &*(self.data.as_ptr().add(offset) as *const T) }
    }

    pub fn get_mut<T: Component>(&mut self, index: usize) -> &mut T {
        let offset = index * self.stride;
        unsafe { &mut *(self.data.as_mut_ptr().add(offset) as *mut T) }
    }

    pub fn swap_remove(&mut self, index: usize) {
        let start = index * self.stride;
        let end = self.data.len() - self.stride;
        if start < end {
            for i in 0..self.stride {
                self.data[start + i] = self.data[end + i];
            }
        }
        self.data.truncate(end);
    }

    pub fn len(&self) -> usize {
        self.data.len() / self.stride
    }
}

pub struct Archetype {
    pub id: usize,
    pub key: ArchetypeKey,
    pub entities: Vec<Entity>,
    pub columns: Vec<Column>,
}

impl Archetype {
    pub fn new(id: usize, key: ArchetypeKey, registry: &ComponentRegistry) -> Self {
        let columns = key
            .iter()
            .map(|&cid| {
                let info = registry.info(cid);
                Column::new(cid, info.size)
            })
            .collect();
        Self {
            id,
            key,
            entities: Vec::new(),
            columns,
        }
    }

    pub fn matches(&self, query: &[ComponentId]) -> bool {
        query.iter().all(|qid| self.key.contains(qid))
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

pub struct CommandBuffer {
    spawns: Vec<(Entity, Vec<(ComponentId, Box<dyn Any>)>)>,
    despawns: Vec<Entity>,
    component_adds: Vec<(Entity, ComponentId, Box<dyn Any>)>,
    component_removes: Vec<(Entity, ComponentId)>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            spawns: Vec::new(),
            despawns: Vec::new(),
            component_adds: Vec::new(),
            component_removes: Vec::new(),
        }
    }

    pub fn spawn(&mut self, entity: Entity, components: Vec<(ComponentId, Box<dyn Any>)>) {
        self.spawns.push((entity, components));
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.despawns.push(entity);
    }

    pub fn add_component(&mut self, entity: Entity, id: ComponentId, data: Box<dyn Any>) {
        self.component_adds.push((entity, id, data));
    }

    pub fn remove_component(&mut self, entity: Entity, id: ComponentId) {
        self.component_removes.push((entity, id));
    }

    pub fn apply(&mut self, world: &mut World) {
        for (entity, components) in self.spawns.drain(..) {
            world.spawn_with_components_raw(entity, components);
        }
        for &entity in &self.despawns {
            world.despawn_raw(entity);
        }
        for (entity, id, data) in self.component_adds.drain(..) {
            world.add_component_raw(entity, id, data);
        }
        for (entity, id) in self.component_removes.drain(..) {
            world.remove_component_raw(entity, id);
        }
        self.despawns.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.spawns.is_empty()
            && self.despawns.is_empty()
            && self.component_adds.is_empty()
            && self.component_removes.is_empty()
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct World {
    archetypes: Vec<Archetype>,
    entity_to_archetype: HashMap<Entity, (usize, usize)>,
    command_buffer: CommandBuffer,
    registry: ComponentRegistry,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            entity_to_archetype: HashMap::new(),
            command_buffer: CommandBuffer::new(),
            registry: ComponentRegistry::new(),
        }
    }

    pub fn spawn<T: Component>(&mut self, components: T) -> Entity {
        let entity = Entity::new();
        let cid = self.registry.register::<T>();
        let key = vec![cid];
        let archetype_id = self.find_or_create_archetype(&key);
        let archetype = &mut self.archetypes[archetype_id];
        let row = archetype.entities.len();
        archetype.entities.push(entity);
        // Ensure column exists for the component
        if archetype.columns.is_empty() {
            let info = self.registry.info(cid);
            archetype.columns.push(Column::new(cid, info.size));
        }
        archetype.columns[0].push(&components);
        self.entity_to_archetype.insert(entity, (archetype_id, row));
        entity
    }

    pub fn spawn_with_components_raw(
        &mut self,
        entity: Entity,
        components: Vec<(ComponentId, Box<dyn Any>)>,
    ) {
        let mut key: Vec<ComponentId> = components.iter().map(|(id, _)| *id).collect();
        key.sort_by_key(|id| id.0);
        let archetype_id = self.find_or_create_archetype(&key);
        let archetype = &mut self.archetypes[archetype_id];
        let row = archetype.entities.len();
        archetype.entities.push(entity);
        for (cid, data) in &components {
            let col = match archetype
                .columns
                .iter_mut()
                .position(|c| c.component_id == *cid) {
                    Some(c) => c,
                    None => continue,
                };
            let info = self.registry.info(*cid);
            let ptr = &*data as *const dyn Any as *const u8;
            // Push raw bytes
            unsafe {
                let slice = std::slice::from_raw_parts(ptr, info.size);
                archetype.columns[col].data.extend_from_slice(slice);
            }
        }
        self.entity_to_archetype.insert(entity, (archetype_id, row));
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.despawn_raw(entity);
    }

    fn despawn_raw(&mut self, entity: Entity) {
        if let Some(&(arch_id, row)) = self.entity_to_archetype.get(&entity) {
            let archetype = &mut self.archetypes[arch_id];
            let last_row = archetype.entities.len() - 1;
            if row != last_row {
                let last_entity = archetype.entities[last_row];
                archetype.entities[row] = last_entity;
                for col in &mut archetype.columns {
                    col.swap_remove(row);
                }
                self.entity_to_archetype
                    .insert(last_entity, (arch_id, row));
            } else {
                for col in &mut archetype.columns {
                    col.data.truncate(col.data.len() - col.stride);
                }
                archetype.entities.pop();
            }
            self.entity_to_archetype.remove(&entity);
        }
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        let cid = self.registry.register::<T>();
        self.add_component_raw(entity, cid, Box::new(component));
    }

    fn add_component_raw(&mut self, entity: Entity, cid: ComponentId, data: Box<dyn Any>) {
        if let Some(&(arch_id, row)) = self.entity_to_archetype.get(&entity) {
            let old_key = self.archetypes[arch_id].key.clone();
            if old_key.contains(&cid) {
                return; // Already has this component
            }
            let mut new_key = old_key.clone();
            new_key.push(cid);
            new_key.sort_by_key(|id| id.0);
            let new_arch_id = self.find_or_create_archetype(&new_key);
            // Move entity to new archetype
            let entity = self.archetypes[arch_id].entities[row];
            let mut component_data = Vec::new();
            for &ocid in &old_key {
                let col = match self.archetypes[arch_id]
                    .columns
                    .iter()
                    .position(|c| c.component_id == ocid) {
                        Some(c) => c,
                        None => continue,
                    };
                let info = self.registry.info(ocid);
                let start = row * info.size;
                let end = start + info.size;
                let bytes = self.archetypes[arch_id].columns[col].data[start..end].to_vec();
                component_data.push((ocid, bytes, info.size));
            }
            // Despawn from old archetype
            self.despawn_raw(entity);
            // Spawn into new archetype
            let new_arch = &mut self.archetypes[new_arch_id];
            let new_row = new_arch.entities.len();
            new_arch.entities.push(entity);
            for (ocid, bytes, _size) in &component_data {
                let col = match new_arch
                    .columns
                    .iter()
                    .position(|c| c.component_id == *ocid) {
                        Some(c) => c,
                        None => continue,
                    };
                new_arch.columns[col].data.extend_from_slice(bytes);
            }
            // Add new component column
            let col = match new_arch
                .columns
                .iter()
                .position(|c| c.component_id == cid) {
                    Some(c) => c,
                    None => return,
                };
            let info = self.registry.info(cid);
            let ptr = &*data as *const dyn Any as *const u8;
            unsafe {
                let bytes = std::slice::from_raw_parts(ptr, info.size);
                new_arch.columns[col].data.extend_from_slice(bytes);
            }
            self.entity_to_archetype.insert(entity, (new_arch_id, new_row));
        }
    }

    fn remove_component_raw(&mut self, entity: Entity, cid: ComponentId) {
        if let Some(&(arch_id, row)) = self.entity_to_archetype.get(&entity) {
            let old_key = self.archetypes[arch_id].key.clone();
            if !old_key.contains(&cid) {
                return;
            }
            let new_key: Vec<ComponentId> =
                old_key.iter().filter(|&&id| id != cid).copied().collect();
            if new_key.is_empty() {
                self.despawn_raw(entity);
                return;
            }
            let new_arch_id = self.find_or_create_archetype(&new_key);
            let entity = self.archetypes[arch_id].entities[row];
            let mut component_data = Vec::new();
            for &ocid in &new_key {
                let col = match self.archetypes[arch_id]
                    .columns
                    .iter()
                    .position(|c| c.component_id == ocid) {
                        Some(c) => c,
                        None => continue,
                    };
                let info = self.registry.info(ocid);
                let start = row * info.size;
                let end = start + info.size;
                let bytes = self.archetypes[arch_id].columns[col].data[start..end].to_vec();
                component_data.push((ocid, bytes));
            }
            self.despawn_raw(entity);
            let new_arch = &mut self.archetypes[new_arch_id];
            let new_row = new_arch.entities.len();
            new_arch.entities.push(entity);
            for (ocid, bytes) in &component_data {
                let col = match new_arch
                    .columns
                    .iter()
                    .position(|c| c.component_id == *ocid) {
                        Some(c) => c,
                        None => continue,
                    };
                new_arch.columns[col].data.extend_from_slice(bytes);
            }
            self.entity_to_archetype.insert(entity, (new_arch_id, new_row));
        }
    }

    fn find_or_create_archetype(&mut self, key: &[ComponentId]) -> usize {
        let mut sorted = key.to_vec();
        sorted.sort_by_key(|id| id.0);
        sorted.dedup();
        for (i, arch) in self.archetypes.iter().enumerate() {
            if arch.key == sorted {
                return i;
            }
        }
        let id = self.archetypes.len();
        let arch = Archetype::new(id, sorted.clone(), &self.registry);
        self.archetypes.push(arch);
        id
    }

    pub fn query_entities(&self, component_ids: &[ComponentId]) -> Vec<Entity> {
        let mut results = Vec::new();
        for arch in &self.archetypes {
            if component_ids.iter().all(|cid| arch.key.contains(cid)) {
                results.extend_from_slice(&arch.entities);
            }
        }
        results
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let (arch_id, row) = self.entity_to_archetype.get(&entity)?;
        let arch = &self.archetypes[*arch_id];
        let cid = self.registry.resolve::<T>();
        let col = arch.columns.iter().find(|c| c.component_id == cid)?;
        Some(col.get::<T>(*row))
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let (arch_id, row) = self.entity_to_archetype.get(&entity)?;
        let arch = &mut self.archetypes[*arch_id];
        let cid = self.registry.resolve::<T>();
        let col = arch.columns.iter_mut().find(|c| c.component_id == cid)?;
        Some(col.get_mut::<T>(*row))
    }

    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        let cid = match self.registry.id_of::<T>() {
            Some(cid) => cid,
            None => return false,
        };
        let (_arch_id, _row) = match self.entity_to_archetype.get(&entity) {
            Some(v) => v,
            None => return false,
        };
        self.archetypes[*_arch_id].columns.iter().any(|c| c.component_id == cid)
    }

    pub fn entity_count(&self) -> usize {
        self.entity_to_archetype.len()
    }

    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    pub fn registry(&self) -> &ComponentRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut ComponentRegistry {
        &mut self.registry
    }

    pub fn command_buffer(&mut self) -> &mut CommandBuffer {
        &mut self.command_buffer
    }

    pub fn flush(&mut self) {
        let mut cmd = std::mem::take(&mut self.command_buffer);
        cmd.apply(self);
    }

    pub fn clear_all(&mut self) {
        self.archetypes.clear();
        self.entity_to_archetype.clear();
        self.command_buffer = CommandBuffer::new();
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

    struct Pos { x: f32, y: f32 }
    struct Vel { x: f32, y: f32 }

    #[test]
    fn test_spawn_entity() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 1.0, y: 2.0 });
        assert!(!e.is_null());
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_despawn_entity() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 0.0, y: 0.0 });
        assert_eq!(world.entity_count(), 1);
        world.despawn(e);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_multiple_entities() {
        let mut world = World::new();
        let _e1 = world.spawn(Pos { x: 1.0, y: 2.0 });
        let _e2 = world.spawn(Pos { x: 3.0, y: 4.0 });
        let _e3 = world.spawn(Pos { x: 5.0, y: 6.0 });
        assert_eq!(world.entity_count(), 3);
    }

    #[test]
    fn test_archetype_creation() {
        let mut world = World::new();
        let _e1 = world.spawn(Pos { x: 1.0, y: 2.0 });
        let _e2 = world.spawn(Vel { x: 1.0, y: 0.0 });
        assert_eq!(world.archetype_count(), 2);
    }

    #[test]
    fn test_add_component() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 1.0, y: 2.0 });
        world.add_component(e, Vel { x: 3.0, y: 4.0 });
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_command_buffer_spawn() {
        let mut world = World::new();
        let e = Entity::new();
        let cid = world.registry_mut().register::<Pos>();
        world.command_buffer().spawn(e, vec![(cid, Box::new(Pos { x: 1.0, y: 2.0 }))]);
        world.flush();
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_command_buffer_despawn() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 0.0, y: 0.0 });
        world.command_buffer().despawn(e);
        world.flush();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_query_entities() {
        let mut world = World::new();
        let e1 = world.spawn(Pos { x: 1.0, y: 2.0 });
        let e2 = world.spawn(Pos { x: 3.0, y: 4.0 });
        let cid = world.registry().resolve::<Pos>();
        let entities = world.query_entities(&[cid]);
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
    }

    #[test]
    fn test_get_component() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 10.0, y: 20.0 });
        let pos = world.get_component::<Pos>(e).unwrap();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_get_component_mut() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 1.0, y: 2.0 });
        {
            let pos = world.get_component_mut::<Pos>(e).unwrap();
            pos.x = 100.0;
        }
        let pos = world.get_component::<Pos>(e).unwrap();
        assert_eq!(pos.x, 100.0);
    }

    #[test]
    fn test_has_component() {
        let mut world = World::new();
        let e = world.spawn(Pos { x: 0.0, y: 0.0 });
        assert!(world.has_component::<Pos>(e));
        assert!(!world.has_component::<Vel>(e));
    }

    #[test]
    fn test_bulk_spawn() {
        let mut world = World::new();
        for i in 0..1000 {
            world.spawn(Pos { x: i as f32, y: 0.0 });
        }
        assert_eq!(world.entity_count(), 1000);
    }

    #[test]
    fn test_swap_remove_preserves_order() {
        let mut world = World::new();
        let e1 = world.spawn(Pos { x: 1.0, y: 2.0 });
        let e2 = world.spawn(Pos { x: 3.0, y: 4.0 });
        let e3 = world.spawn(Pos { x: 5.0, y: 6.0 });
        world.despawn(e2);
        assert_eq!(world.entity_count(), 2);
        world.despawn(e1);
        assert_eq!(world.entity_count(), 1);
        world.despawn(e3);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_clear_all() {
        let mut world = World::new();
        world.spawn(Pos { x: 1.0, y: 2.0 });
        world.spawn(Pos { x: 3.0, y: 4.0 });
        assert_eq!(world.entity_count(), 2);
        world.clear_all();
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.archetype_count(), 0);
    }
}

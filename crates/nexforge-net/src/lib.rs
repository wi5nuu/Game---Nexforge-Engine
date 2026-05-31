#![deny(clippy::all)]

use thiserror::Error;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Rollback error: frame mismatch")]
    RollbackMismatch,
    #[error("Not connected to server")]
    NotConnected,
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerInput {
    pub frame: u32,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shoot: bool,
    pub reload: bool,
    pub sprint: bool,
    pub crouch: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub sequence: u32,
}

impl PlayerInput {
    pub fn new() -> Self {
        Self { frame: 0, forward: false, backward: false, left: false, right: false, jump: false,
            shoot: false, reload: false, sprint: false, crouch: false, mouse_x: 0.0, mouse_y: 0.0, sequence: 0 }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, NetError> {
        bincode::serialize(self).map_err(|e| NetError::SerializationError(e.to_string()))
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, NetError> {
        bincode::deserialize(data).map_err(|e| NetError::SerializationError(e.to_string()))
    }

    pub fn delta(&self, base: &PlayerInput) -> Vec<u8> {
        let mut delta = Vec::new();
        if self.forward != base.forward { delta.push(1); delta.push(self.forward as u8); }
        if self.shoot != base.shoot { delta.push(2); delta.push(self.shoot as u8); }
        if (self.mouse_x - base.mouse_x).abs() > 0.001 { delta.push(3); delta.extend(&self.mouse_x.to_le_bytes()); }
        delta
    }
}

impl Default for PlayerInput { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub frame: u32,
    pub entities: Vec<EntitySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: u64,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub velocity: [f32; 3],
    pub health: f32,
}

pub struct InputBuffer {
    inputs: Vec<PlayerInput>,
    max_size: usize,
}

impl InputBuffer {
    pub fn new(max_size: usize) -> Self { Self { inputs: Vec::new(), max_size } }

    pub fn push(&mut self, input: PlayerInput) {
        if self.inputs.len() >= self.max_size { self.inputs.remove(0); }
        self.inputs.push(input);
    }

    pub fn get(&self, frame: u32) -> Option<&PlayerInput> {
        self.inputs.iter().find(|i| i.frame == frame)
    }

    pub fn latest_frame(&self) -> u32 { self.inputs.last().map(|i| i.frame).unwrap_or(0) }

    pub fn clear(&mut self) { self.inputs.clear(); }
}

pub struct RollbackManager {
    pub current_frame: u32,
    pub max_prediction: u32,
    pub input_buffer: InputBuffer,
    pub snapshots: Vec<WorldSnapshot>,
    pub max_snapshots: usize,
    pub local_player_id: u64,
}

impl RollbackManager {
    pub fn new() -> Self {
        Self { current_frame: 0, max_prediction: 8, input_buffer: InputBuffer::new(256),
            snapshots: Vec::new(), max_snapshots: 128, local_player_id: 0 }
    }

    pub fn save_snapshot(&mut self, snapshot: WorldSnapshot) {
        if self.snapshots.len() >= self.max_snapshots { self.snapshots.remove(0); }
        self.snapshots.push(snapshot);
    }

    pub fn find_snapshot(&self, frame: u32) -> Option<&WorldSnapshot> {
        self.snapshots.iter().find(|s| s.frame == frame)
    }

    pub fn rollback(&mut self, target_frame: u32) -> Result<Option<WorldSnapshot>, NetError> {
        if target_frame >= self.current_frame { return Ok(None); }
        let snapshot = self.find_snapshot(target_frame)
            .ok_or(NetError::RollbackMismatch)?;
        Ok(Some(snapshot.clone()))
    }

    pub fn advance_frame(&mut self) { self.current_frame += 1; }

    pub fn predict(&self, snapshot: &WorldSnapshot, inputs: &[PlayerInput]) -> WorldSnapshot {
        let mut predicted = snapshot.clone();
        for input in inputs {
            if input.frame <= snapshot.frame { continue; }
            for entity in &mut predicted.entities {
                if input.forward { entity.position[2] -= 0.1; }
                if input.backward { entity.position[2] += 0.1; }
                if input.left { entity.position[0] -= 0.1; }
                if input.right { entity.position[0] += 0.1; }
            }
        }
        predicted
    }
}

impl Default for RollbackManager { fn default() -> Self { Self::new() } }

// Delta compression
pub struct DeltaCompressor;

impl DeltaCompressor {
    pub fn new() -> Self { Self }

    pub fn compress(previous: &WorldSnapshot, current: &WorldSnapshot) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(&current.frame.to_le_bytes());
        for entity in &current.entities {
            let prev = previous.entities.iter().find(|e| e.id == entity.id);
            if let Some(p) = prev {
                if p.position != entity.position {
                    data.push(0); // position changed
                    data.extend(&entity.id.to_le_bytes());
                    data.extend_from_slice(bytemuck::cast_slice(&entity.position));
                }
                if (p.health - entity.health).abs() > 0.001 {
                    data.push(1); // health changed
                    data.extend(&entity.id.to_le_bytes());
                    data.extend(&entity.health.to_le_bytes());
                }
            } else {
                data.push(2); // new entity
                data.extend(&entity.id.to_le_bytes());
                data.extend(&entity.position[0].to_le_bytes());
                data.extend(&entity.position[1].to_le_bytes());
                data.extend(&entity.position[2].to_le_bytes());
            }
        }
        data
    }
}

impl Default for DeltaCompressor { fn default() -> Self { Self::new() } }

pub struct NetEngine {
    pub rollback: RollbackManager,
    pub delta: DeltaCompressor,
    pub tick_rate: u32,
    pub is_server: bool,
    pub ping: f32,
    initialized: bool,
}

impl NetEngine {
    pub fn new() -> Self {
        Self { rollback: RollbackManager::new(), delta: DeltaCompressor::new(),
            tick_rate: 60, is_server: false, ping: 0.0, initialized: false }
    }

    pub fn initialize(&mut self) -> Result<(), NetError> {
        self.initialized = true;
        Ok(())
    }

    pub fn start_server(&mut self) { self.is_server = true; }

    pub fn connect(&mut self, _address: &str) -> Result<(), NetError> {
        // WebRTC/UDP connection placeholder
        self.initialized = true;
        Ok(())
    }

    pub fn send_input(&mut self, input: PlayerInput) -> Result<(), NetError> {
        if !self.initialized { return Err(NetError::NotConnected); }
        self.rollback.input_buffer.push(input);
        let _data = input.serialize()?;
        // Send over WebRTC/UDP — placeholder
        Ok(())
    }

    pub fn receive_snapshot(&mut self, data: &[u8]) -> Result<WorldSnapshot, NetError> {
        let snapshot: WorldSnapshot = bincode::deserialize(data)
            .map_err(|e| NetError::SerializationError(e.to_string()))?;
        self.rollback.save_snapshot(snapshot.clone());
        Ok(snapshot)
    }

    pub fn tick(&mut self) {
        self.rollback.advance_frame();
    }

    pub fn is_initialized(&self) -> bool { self.initialized }
}

impl Default for NetEngine { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_init() {
        let mut engine = NetEngine::new();
        assert!(!engine.is_initialized());
        assert!(engine.initialize().is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_input_serialization() {
        let input = PlayerInput { forward: true, shoot: true, frame: 42, ..Default::default() };
        let data = input.serialize().unwrap();
        let deserialized = PlayerInput::deserialize(&data).unwrap();
        assert_eq!(input, deserialized);
    }

    #[test]
    fn test_input_buffer() {
        let mut buf = InputBuffer::new(10);
        for i in 0..5 {
            buf.push(PlayerInput { frame: i, ..Default::default() });
        }
        assert_eq!(buf.latest_frame(), 4);
        assert!(buf.get(2).is_some());
        assert!(buf.get(10).is_none());
    }

    #[test]
    fn test_rollback_save_and_find() {
        let mut rm = RollbackManager::new();
        let snap = WorldSnapshot { frame: 10, entities: vec![] };
        rm.save_snapshot(snap.clone());
        assert!(rm.find_snapshot(10).is_some());
    }

    #[test]
    fn test_rollback_frame_advance() {
        let mut rm = RollbackManager::new();
        assert_eq!(rm.current_frame, 0);
        rm.advance_frame();
        assert_eq!(rm.current_frame, 1);
    }

    #[test]
    fn test_delta_compression() {
        let prev = WorldSnapshot {
            frame: 1,
            entities: vec![EntitySnapshot { id: 0, position: [0.0; 3], rotation: [0.0; 4], velocity: [0.0; 3], health: 100.0 }],
        };
        let current = WorldSnapshot {
            frame: 2,
            entities: vec![EntitySnapshot { id: 0, position: [1.0, 0.0, 0.0], rotation: [0.0; 4], velocity: [0.0; 3], health: 80.0 }],
        };
        let compressed = DeltaCompressor::compress(&prev, &current);
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_prediction() {
        let mut rm = RollbackManager::new();
        let snapshot = WorldSnapshot {
            frame: 0,
            entities: vec![EntitySnapshot { id: 0, position: [0.0; 3], rotation: [0.0; 4], velocity: [0.0; 3], health: 100.0 }],
        };
        let inputs = vec![PlayerInput { frame: 1, forward: true, ..Default::default() }];
        let predicted = rm.predict(&snapshot, &inputs);
        assert_eq!(predicted.entities[0].position[2], -0.1);
    }
}

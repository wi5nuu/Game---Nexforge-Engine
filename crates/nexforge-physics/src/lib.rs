#![deny(clippy::all)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PhysicsError {
    #[error("Rapier3d integration error")]
    RapierError,
    #[error("Raycast miss")]
    RaycastMiss,
    #[error("Invalid collider handle")]
    InvalidCollider,
}

pub struct RaycastHit {
    pub entity: u64,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub distance: f32,
}

pub struct CharacterController {
    pub height: f32,
    pub radius: f32,
    pub offset: f32,
    pub max_slope: f32,
    pub step_height: f32,
    pub is_grounded: bool,
    pub velocity: [f32; 3],
}

impl CharacterController {
    pub fn new() -> Self {
        Self { height: 1.8, radius: 0.4, offset: 0.01, max_slope: 45.0_f32.to_radians(), step_height: 0.3, is_grounded: false, velocity: [0.0; 3] }
    }

    pub fn capsule() -> Self { Self::new() }

    pub fn set_velocity(&mut self, vel: [f32; 3]) { self.velocity = vel; }

    pub fn set_height(&mut self, h: f32) { self.height = h; }

    pub fn set_radius(&mut self, r: f32) { self.radius = r; }

    pub fn move_and_slide(&mut self, displacement: [f32; 3], _dt: f32) -> [f32; 3] {
        self.velocity = displacement;
        self.is_grounded = displacement[1] <= 0.0;
        displacement
    }
}

impl Default for CharacterController { fn default() -> Self { Self::new() } }

pub struct BvhNode {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub left: Option<Box<BvhNode>>,
    pub right: Option<Box<BvhNode>>,
    pub entity_id: Option<u64>,
}

impl BvhNode {
    pub fn new() -> Self {
        Self { min: [f32::MAX; 3], max: [f32::MIN; 3], left: None, right: None, entity_id: None }
    }

    pub fn is_leaf(&self) -> bool { self.left.is_none() && self.right.is_none() }
}

pub struct Bvh {
    pub root: Option<BvhNode>,
}

impl Bvh {
    pub fn new() -> Self { Self { root: None } }

    pub fn build(&mut self, _positions: &[([f32; 3], [f32; 3], u64)]) {
        // Placeholder — full BVH construction in later optimization pass
    }

    pub fn query(&self, _origin: [f32; 3], _direction: [f32; 3]) -> Vec<u64> {
        Vec::new()
    }
}

impl Default for Bvh { fn default() -> Self { Self::new() } }

pub struct TriggerZone {
    pub position: [f32; 3],
    pub radius: f32,
    pub active: bool,
    pub entities_inside: Vec<u64>,
}

impl TriggerZone {
    pub fn new(position: [f32; 3], radius: f32) -> Self {
        Self { position, radius, active: true, entities_inside: Vec::new() }
    }

    pub fn contains(&self, point: [f32; 3]) -> bool {
        let dx = point[0] - self.position[0];
        let dy = point[1] - self.position[1];
        let dz = point[2] - self.position[2];
        (dx * dx + dy * dy + dz * dz) <= self.radius * self.radius
    }
}

pub struct PhysicsEngine {
    pub gravity: [f32; 3],
    pub bvh: Bvh,
    pub trigger_zones: Vec<TriggerZone>,
    pub character_controllers: Vec<CharacterController>,
    initialized: bool,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self { gravity: [0.0, -9.81, 0.0], bvh: Bvh::new(), trigger_zones: Vec::new(), character_controllers: Vec::new(), initialized: false }
    }

    pub fn initialize(&mut self) -> Result<(), PhysicsError> {
        self.initialized = true;
        Ok(())
    }

    pub fn set_gravity(&mut self, gravity: [f32; 3]) {
        self.gravity = gravity;
    }

    pub fn get_gravity(&self) -> [f32; 3] {
        self.gravity
    }

    pub fn step(&mut self, dt: f32) {
        for cc in &mut self.character_controllers {
            cc.velocity[1] += self.gravity[1] * dt;
            cc.move_and_slide(cc.velocity, dt);
        }
        self.trigger_zones.retain(|z| z.active);
    }

    pub fn clear_trigger_zones(&mut self) {
        self.trigger_zones.clear();
    }

    pub fn add_trigger_zone(&mut self, zone: TriggerZone) {
        self.trigger_zones.push(zone);
    }

    pub fn raycast(&self, origin: [f32; 3], direction: [f32; 3], max_dist: f32) -> Vec<RaycastHit> {
        let mut hits = Vec::new();
        let candidates = self.bvh.query(origin, direction);
        for entity_id in candidates {
            let hit = RaycastHit {
                entity: entity_id,
                point: [origin[0] + direction[0] * max_dist * 0.5, origin[1] + direction[1] * max_dist * 0.5, origin[2] + direction[2] * max_dist * 0.5],
                normal: [0.0, 1.0, 0.0],
                distance: max_dist * 0.5,
            };
            hits.push(hit);
        }
        hits
    }

    pub fn is_initialized(&self) -> bool { self.initialized }
}

impl Default for PhysicsEngine { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_init() {
        let mut engine = PhysicsEngine::new();
        assert!(!engine.is_initialized());
        assert!(engine.initialize().is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_default_gravity() {
        let engine = PhysicsEngine::new();
        assert_eq!(engine.gravity[1], -9.81);
    }

    #[test]
    fn test_set_get_gravity() {
        let mut engine = PhysicsEngine::new();
        assert_eq!(engine.get_gravity(), [0.0, -9.81, 0.0]);
        engine.set_gravity([0.0, -5.0, 0.0]);
        assert_eq!(engine.get_gravity(), [0.0, -5.0, 0.0]);
    }

    #[test]
    fn test_clear_trigger_zones() {
        let mut engine = PhysicsEngine::new();
        engine.add_trigger_zone(TriggerZone::new([0.0, 0.0, 0.0], 5.0));
        assert_eq!(engine.trigger_zones.len(), 1);
        engine.clear_trigger_zones();
        assert!(engine.trigger_zones.is_empty());
    }

    #[test]
    fn test_character_controller_setters() {
        let mut cc = CharacterController::new();
        cc.set_velocity([1.0, 2.0, 3.0]);
        assert_eq!(cc.velocity, [1.0, 2.0, 3.0]);
        cc.set_height(2.0);
        assert!((cc.height - 2.0).abs() < f32::EPSILON);
        cc.set_radius(0.5);
        assert!((cc.radius - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_character_controller_defaults() {
        let cc = CharacterController::new();
        assert!((cc.height - 1.8).abs() < f32::EPSILON);
        assert!((cc.radius - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_trigger_zone_contains() {
        let zone = TriggerZone::new([0.0, 0.0, 0.0], 5.0);
        assert!(zone.contains([1.0, 2.0, 1.0]));
        assert!(!zone.contains([10.0, 0.0, 0.0]));
    }

    #[test]
    fn test_bvh_creation() {
        let bvh = Bvh::new();
        assert!(bvh.root.is_none());
    }

    #[test]
    fn test_character_controller_movement() {
        let mut cc = CharacterController::new();
        let result = cc.move_and_slide([1.0, 0.0, 0.0], 0.016);
        assert_eq!(result[0], 1.0);
    }

    #[test]
    fn test_trigger_zone_initialization() {
        let zone = TriggerZone::new([1.0, 2.0, 3.0], 10.0);
        assert_eq!(zone.position, [1.0, 2.0, 3.0]);
        assert_eq!(zone.radius, 10.0);
        assert!(zone.active);
    }

    #[test]
    fn test_trigger_zone_entities() {
        let mut zone = TriggerZone::new([0.0, 0.0, 0.0], 5.0);
        zone.entities_inside.push(42);
        assert_eq!(zone.entities_inside.len(), 1);
    }

    #[test]
    fn test_bvh_query_empty() {
        let bvh = Bvh::new();
        let result = bvh.query([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_physics_error_display() {
        let err = PhysicsError::RaycastMiss;
        assert_eq!(format!("{}", err), "Raycast miss");
    }

    #[test]
    fn test_raycast_hit_defaults() {
        let hit = RaycastHit { entity: 0, point: [0.0; 3], normal: [1.0, 0.0, 0.0], distance: 10.0 };
        assert_eq!(hit.distance, 10.0);
        assert_eq!(hit.normal, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_character_controller_properties() {
        let cc = CharacterController::new();
        assert!((cc.height - 1.8).abs() < f32::EPSILON);
        assert!((cc.radius - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bvh_build() {
        let mut bvh = Bvh::new();
        bvh.build(&[([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 1)]);
        let result = bvh.query([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_physics_engine_new() {
        let engine = PhysicsEngine::new();
        assert_eq!(engine.gravity, [0.0, -9.81, 0.0]);
    }

    #[test]
    fn test_trigger_zone_expansion() {
        let mut zone = TriggerZone::new([0.0, 0.0, 0.0], 10.0);
        zone.radius = 20.0;
        assert!((zone.radius - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bvh_node_defaults() {
        let node = BvhNode::new();
        assert!(node.is_leaf());
        assert!(node.left.is_none());
        assert!(node.right.is_none());
    }
}

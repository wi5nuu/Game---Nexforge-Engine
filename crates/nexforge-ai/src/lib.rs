#![deny(clippy::all)]

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("Path not found")]
    PathNotFound,
    #[error("Invalid behavior tree node")]
    InvalidNode,
    #[error("NavMesh not built")]
    NavMeshNotBuilt,
}

// ── Behavior Tree ──────────────────────────────────────────────

pub type Blackboard = HashMap<String, f32>;

#[derive(Debug, Clone, PartialEq)]
pub enum BtStatus {
    Success,
    Failure,
    Running,
}

pub trait BtNode {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BtStatus;
}

// Sequence: run children in order, fail if any fails
pub struct Sequence {
    children: Vec<Box<dyn BtNode>>,
    current: usize,
}

impl Sequence {
    pub fn new(children: Vec<Box<dyn BtNode>>) -> Self {
        Self { children, current: 0 }
    }
}

impl BtNode for Sequence {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BtStatus {
        while self.current < self.children.len() {
            match self.children[self.current].tick(blackboard) {
                BtStatus::Success => self.current += 1,
                BtStatus::Running => return BtStatus::Running,
                BtStatus::Failure => {
                    self.current = 0;
                    return BtStatus::Failure;
                }
            }
        }
        self.current = 0;
        BtStatus::Success
    }
}

// Selector: run children in order, succeed if any succeeds
pub struct Selector {
    children: Vec<Box<dyn BtNode>>,
    current: usize,
}

impl Selector {
    pub fn new(children: Vec<Box<dyn BtNode>>) -> Self {
        Self { children, current: 0 }
    }
}

impl BtNode for Selector {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BtStatus {
        while self.current < self.children.len() {
            match self.children[self.current].tick(blackboard) {
                BtStatus::Failure => self.current += 1,
                BtStatus::Running => return BtStatus::Running,
                BtStatus::Success => {
                    self.current = 0;
                    return BtStatus::Success;
                }
            }
        }
        self.current = 0;
        BtStatus::Failure
    }
}

// Parallel: run all children, succeed/fail based on threshold
pub struct Parallel {
    children: Vec<Box<dyn BtNode>>,
    success_threshold: usize,
}

impl Parallel {
    pub fn new(children: Vec<Box<dyn BtNode>>, success_threshold: usize) -> Self {
        Self {
            children,
            success_threshold,
        }
    }
}

impl BtNode for Parallel {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BtStatus {
        let mut success = 0;
        let mut failure = 0;
        for child in &mut self.children {
            match child.tick(blackboard) {
                BtStatus::Success => success += 1,
                BtStatus::Failure => failure += 1,
                BtStatus::Running => {}
            }
        }
        if success >= self.success_threshold {
            BtStatus::Success
        } else if failure > self.children.len() - self.success_threshold {
            BtStatus::Failure
        } else {
            BtStatus::Running
        }
    }
}

// Condition: check a blackboard value
pub struct Condition {
    name: String,
    expected: f32,
    tolerance: f32,
}

impl Condition {
    pub fn new(name: &str, expected: f32) -> Self {
        Self {
            name: name.to_string(),
            expected,
            tolerance: 0.001,
        }
    }
}

impl BtNode for Condition {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BtStatus {
        match blackboard.get(&self.name) {
            Some(val) if (*val - self.expected).abs() < self.tolerance => BtStatus::Success,
            _ => BtStatus::Failure,
        }
    }
}

// Action: set a blackboard value
pub struct Action {
    name: String,
    value: f32,
}

impl Action {
    pub fn new(name: &str, value: f32) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}

impl BtNode for Action {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BtStatus {
        blackboard.insert(self.name.clone(), self.value);
        BtStatus::Success
    }
}

// Decorator: invert child result
pub struct Inverter {
    child: Box<dyn BtNode>,
}

impl Inverter {
    pub fn new(child: Box<dyn BtNode>) -> Self {
        Self { child }
    }
}

impl BtNode for Inverter {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BtStatus {
        match self.child.tick(blackboard) {
            BtStatus::Success => BtStatus::Failure,
            BtStatus::Failure => BtStatus::Success,
            BtStatus::Running => BtStatus::Running,
        }
    }
}

// Repeat: repeat child N times
pub struct Repeat {
    child: Box<dyn BtNode>,
    count: usize,
    current: usize,
}

impl Repeat {
    pub fn new(child: Box<dyn BtNode>, count: usize) -> Self {
        Self {
            child,
            count,
            current: 0,
        }
    }
}

impl BtNode for Repeat {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BtStatus {
        if self.current >= self.count {
            return BtStatus::Success;
        }
        match self.child.tick(blackboard) {
            BtStatus::Success => {
                self.current += 1;
                if self.current >= self.count {
                    BtStatus::Success
                } else {
                    BtStatus::Running
                }
            }
            BtStatus::Running => BtStatus::Running,
            BtStatus::Failure => BtStatus::Failure,
        }
    }
}

// ── A* Pathfinding ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavNode {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub idx: usize,
}

pub struct NavMesh {
    pub nodes: Vec<NavNode>,
    pub neighbors: Vec<Vec<usize>>,
    pub triangle_indices: Vec<[usize; 3]>,
}

impl NavMesh {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            neighbors: Vec::new(),
            triangle_indices: Vec::new(),
        }
    }

    pub fn has_node(&self, idx: usize) -> bool {
        idx < self.nodes.len()
    }

    pub fn remove_node(&mut self, idx: usize) -> bool {
        if idx >= self.nodes.len() {
            return false;
        }
        self.nodes.remove(idx);
        self.neighbors.remove(idx);
        for neighbors in &mut self.neighbors {
            neighbors.retain(|n| *n != idx);
            for n in neighbors.iter_mut() {
                if *n > idx {
                    *n -= 1;
                }
            }
        }
        self.triangle_indices
            .retain(|t| t[0] != idx && t[1] != idx && t[2] != idx);
        for t in &mut self.triangle_indices {
            for c in t.iter_mut() {
                if *c > idx {
                    *c -= 1;
                }
            }
        }
        true
    }

    pub fn add_node(&mut self, x: f32, y: f32, z: f32) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(NavNode { x, y, z, idx });
        self.neighbors.push(Vec::new());
        idx
    }

    pub fn has_edge(&self, a: usize, b: usize) -> bool {
        if a < self.neighbors.len() {
            self.neighbors[a].contains(&b)
        } else {
            false
        }
    }

    pub fn add_edge(&mut self, a: usize, b: usize) {
        if !self.neighbors[a].contains(&b) {
            self.neighbors[a].push(b);
        }
        if !self.neighbors[b].contains(&a) {
            self.neighbors[b].push(a);
        }
    }

    pub fn add_triangle(&mut self, a: usize, b: usize, c: usize) {
        self.triangle_indices.push([a, b, c]);
        self.add_edge(a, b);
        self.add_edge(b, c);
        self.add_edge(c, a);
    }

    pub fn find_path(&self, start: usize, goal: usize) -> Result<Vec<usize>, AiError> {
        if start >= self.nodes.len() || goal >= self.nodes.len() {
            return Err(AiError::PathNotFound);
        }
        let mut open = std::collections::BinaryHeap::<AStarNode>::new();
        let mut g_score = vec![f32::MAX; self.nodes.len()];
        let mut came_from = vec![usize::MAX; self.nodes.len()];

        g_score[start] = 0.0;
        open.push(AStarNode {
            idx: start,
            f: self.heuristic(start, goal),
        });

        while let Some(current) = open.pop() {
            if current.idx == goal {
                let mut path = Vec::new();
                let mut c = goal;
                while c != usize::MAX && c != start {
                    path.push(c);
                    c = came_from[c];
                }
                path.push(start);
                path.reverse();
                return Ok(path);
            }
            for &neighbor in &self.neighbors[current.idx] {
                let tentative = g_score[current.idx] + self.distance(current.idx, neighbor);
                if tentative < g_score[neighbor] {
                    came_from[neighbor] = current.idx;
                    g_score[neighbor] = tentative;
                    open.push(AStarNode {
                        idx: neighbor,
                        f: tentative + self.heuristic(neighbor, goal),
                    });
                }
            }
        }
        Err(AiError::PathNotFound)
    }

    fn distance(&self, a: usize, b: usize) -> f32 {
        let na = &self.nodes[a];
        let nb = &self.nodes[b];
        let dx = na.x - nb.x;
        let dy = na.y - nb.y;
        let dz = na.z - nb.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn clear_nodes(&mut self) {
        self.nodes.clear();
        self.neighbors.clear();
        self.triangle_indices.clear();
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn heuristic(&self, a: usize, b: usize) -> f32 {
        self.distance(a, b)
    }
}

impl Default for NavMesh {
    fn default() -> Self {
        Self::new()
    }
}

struct AStarNode {
    idx: usize,
    f: f32,
}

impl std::cmp::Eq for AStarNode {}
impl std::cmp::PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}
impl std::cmp::Ord for AStarNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f.total_cmp(&self.f)
    }
}
impl std::cmp::PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ── Utility AI ─────────────────────────────────────────────────

pub struct UtilityConsideration {
    pub name: String,
    pub curve: fn(f32) -> f32,
    pub input: fn(&Blackboard) -> f32,
}

impl UtilityConsideration {
    pub fn new(name: &str, curve: fn(f32) -> f32, input: fn(&Blackboard) -> f32) -> Self {
        Self {
            name: name.to_string(),
            curve,
            input,
        }
    }

    pub fn score(&self, bb: &Blackboard) -> f32 {
        let raw = (self.input)(bb);
        (self.curve)(raw.clamp(0.0, 1.0))
    }
}

pub struct UtilityAction {
    pub name: String,
    pub considerations: Vec<UtilityConsideration>,
}

impl UtilityAction {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            considerations: Vec::new(),
        }
    }

    pub fn add(&mut self, c: UtilityConsideration) {
        self.considerations.push(c);
    }

    pub fn score(&self, bb: &Blackboard) -> f32 {
        if self.considerations.is_empty() {
            return 0.0;
        }
        self.considerations.iter().map(|c| c.score(bb)).product()
    }
}

pub fn linear_curve(x: f32) -> f32 {
    x
}
pub fn quadratic_curve(x: f32) -> f32 {
    x * x
}
pub fn inverse_curve(x: f32) -> f32 {
    1.0 - x
}
pub fn logistic_curve(x: f32) -> f32 {
    1.0 / (1.0 + (-10.0 * (x - 0.5)).exp())
}

// ── AI Engine ──────────────────────────────────────────────────

pub struct CoverPoint {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub height: f32,
    pub is_occupied: bool,
}

impl CoverPoint {
    pub fn new(position: [f32; 3], direction: [f32; 3]) -> Self {
        Self {
            position,
            direction,
            height: 1.5,
            is_occupied: false,
        }
    }

    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = pos;
    }

    pub fn set_occupied(&mut self, occupied: bool) {
        self.is_occupied = occupied;
    }
}

pub struct AiEngine {
    pub behavior_trees: HashMap<String, Box<dyn BtNode>>,
    pub blackboards: HashMap<String, Blackboard>,
    pub navmesh: NavMesh,
    pub cover_points: Vec<CoverPoint>,
    initialized: bool,
}

impl AiEngine {
    pub fn new() -> Self {
        Self {
            behavior_trees: HashMap::new(),
            blackboards: HashMap::new(),
            navmesh: NavMesh::new(),
            cover_points: Vec::new(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), AiError> {
        self.initialized = true;
        Ok(())
    }

    pub fn stop(&mut self) {
        self.initialized = false;
    }

    pub fn register_behavior(&mut self, name: &str, tree: Box<dyn BtNode>) {
        self.behavior_trees.insert(name.to_string(), tree);
        self.blackboards.insert(name.to_string(), Blackboard::new());
    }

    pub fn tick(&mut self, name: &str) -> BtStatus {
        if let Some(tree) = self.behavior_trees.get_mut(name) {
            let bb = self.blackboards.entry(name.to_string()).or_default();
            tree.tick(bb)
        } else {
            BtStatus::Failure
        }
    }

    pub fn find_cover(&self, enemy_pos: [f32; 3], _threat_pos: [f32; 3]) -> Option<&CoverPoint> {
        let mut best: Option<&CoverPoint> = None;
        let mut best_score = f32::MIN;
        for cp in &self.cover_points {
            if cp.is_occupied {
                continue;
            }
            let to_enemy = [
                enemy_pos[0] - cp.position[0],
                enemy_pos[1] - cp.position[1],
                enemy_pos[2] - cp.position[2],
            ];
            let len = (to_enemy[0] * to_enemy[0] + to_enemy[1] * to_enemy[1] + to_enemy[2] * to_enemy[2]).sqrt();
            let score = if cp.direction[0] * (to_enemy[0] / len) + cp.direction[2] * (to_enemy[2] / len) > 0.7 {
                1.0
            } else {
                0.0
            };
            if score > best_score {
                best_score = score;
                best = Some(cp);
            }
        }
        best
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for AiEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn always_succeed() -> Box<dyn BtNode> {
        Box::new(Action::new("_success", 1.0))
    }
    fn always_fail() -> Box<dyn BtNode> {
        Box::new(Selector::new(vec![]))
    }

    #[test]
    fn test_ai_init() {
        let mut engine = AiEngine::new();
        assert!(!engine.is_initialized());
        assert!(engine.initialize().is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_sequence_all_success() {
        let mut seq = Sequence::new(vec![always_succeed(), always_succeed()]);
        assert_eq!(seq.tick(&mut Blackboard::new()), BtStatus::Success);
    }

    #[test]
    fn test_selector_all_fail() {
        let mut sel = Selector::new(vec![always_fail(), always_fail()]);
        assert_eq!(sel.tick(&mut Blackboard::new()), BtStatus::Failure);
    }

    #[test]
    fn test_selector_first_succeeds() {
        let mut sel = Selector::new(vec![always_succeed(), always_fail()]);
        assert_eq!(sel.tick(&mut Blackboard::new()), BtStatus::Success);
    }

    #[test]
    fn test_condition_true() {
        let mut bb = Blackboard::new();
        bb.insert("health".to_string(), 100.0);
        let mut cond = Condition::new("health", 100.0);
        assert_eq!(cond.tick(&mut bb), BtStatus::Success);
    }

    #[test]
    fn test_condition_false() {
        let mut bb = Blackboard::new();
        bb.insert("health".to_string(), 50.0);
        let mut cond = Condition::new("health", 100.0);
        assert_eq!(cond.tick(&mut bb), BtStatus::Failure);
    }

    #[test]
    fn test_inverter() {
        let mut inv = Inverter::new(always_fail());
        assert_eq!(inv.tick(&mut Blackboard::new()), BtStatus::Success);
    }

    #[test]
    fn test_navmesh_pathfinding() {
        let mut nav = NavMesh::new();
        let a = nav.add_node(0.0, 0.0, 0.0);
        let b = nav.add_node(10.0, 0.0, 0.0);
        nav.add_edge(a, b);
        let path = nav.find_path(a, b).unwrap();
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn test_navmesh_no_path() {
        let nav = NavMesh::new();
        let result = nav.find_path(0, 1);
        assert!(matches!(result, Err(AiError::PathNotFound)));
    }

    #[test]
    fn test_utility_ai_scoring() {
        let mut bb = Blackboard::new();
        bb.insert("distance".to_string(), 10.0);
        bb.insert("health".to_string(), 50.0);

        let mut action = UtilityAction::new("attack");
        action.add(UtilityConsideration::new("distance", inverse_curve, |bb| {
            bb.get("distance").copied().unwrap_or(0.0) / 100.0
        }));
        action.add(UtilityConsideration::new("health", linear_curve, |bb| {
            bb.get("health").copied().unwrap_or(0.0) / 100.0
        }));
        let score = action.score(&bb);
        assert!(score > 0.0);
    }

    #[test]
    fn test_behavior_tree_registration() {
        let mut engine = AiEngine::new();
        engine.register_behavior("patrol", Box::new(Sequence::new(vec![always_succeed()])));
        assert_eq!(engine.tick("patrol"), BtStatus::Success);
    }

    #[test]
    fn test_cover_system() {
        let mut engine = AiEngine::new();
        engine
            .cover_points
            .push(CoverPoint::new([5.0, 0.0, 5.0], [1.0, 0.0, 0.0]));
        let cover = engine.find_cover([0.0, 0.0, 0.0], [10.0, 0.0, 10.0]);
        assert!(cover.is_some());
    }

    #[test]
    fn test_cover_occupied() {
        let cp = CoverPoint::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(!cp.is_occupied);
    }

    #[test]
    fn test_inverter_node() {
        let mut bb = Blackboard::new();
        let mut invert = Inverter::new(always_fail());
        assert_eq!(invert.tick(&mut bb), BtStatus::Success);
    }

    #[test]
    fn test_repeat_node() {
        let mut bb = Blackboard::new();
        let mut repeat = Repeat::new(always_succeed(), 1);
        assert_eq!(repeat.tick(&mut bb), BtStatus::Success);
    }

    #[test]
    fn test_parallel_node() {
        let mut bb = Blackboard::new();
        let mut parallel = Parallel::new(vec![always_succeed(), always_succeed()], 2);
        assert_eq!(parallel.tick(&mut bb), BtStatus::Success);
    }

    #[test]
    fn test_navmesh_distance() {
        let mut mesh = NavMesh::new();
        let a = mesh.add_node(0.0, 0.0, 0.0);
        let b = mesh.add_node(3.0, 0.0, 4.0);
        let d = mesh.distance(a, b);
        assert!((d - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_a_star_pathfinding() {
        let mut mesh = NavMesh::new();
        let n0 = mesh.add_node(0.0, 0.0, 0.0);
        let n1 = mesh.add_node(10.0, 0.0, 0.0);
        let n2 = mesh.add_node(10.0, 0.0, 10.0);
        mesh.add_edge(n0, n1);
        mesh.add_edge(n1, n2);
        let path = mesh.find_path(n0, n2);
        assert!(path.is_ok());
        assert_eq!(path.unwrap().len(), 3);
    }

    #[test]
    fn test_utility_linear_curve() {
        let score = linear_curve(0.5);
        assert!((score - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_utility_quadratic_curve() {
        let score = quadratic_curve(0.5);
        assert!((score - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_cover_occupancy() {
        let mut cover = CoverPoint::new([0.0; 3], [0.0, 0.0, 1.0]);
        assert!(!cover.is_occupied);
        cover.is_occupied = true;
        assert!(cover.is_occupied);
    }

    #[test]
    fn test_navmesh_add_node() {
        let mut mesh = NavMesh::new();
        let id = mesh.add_node(1.0, 2.0, 3.0);
        assert_eq!(id, 0);
        let id2 = mesh.add_node(4.0, 5.0, 6.0);
        assert_eq!(id2, 1);
    }

    #[test]
    fn test_navmesh_no_edges() {
        let mut mesh = NavMesh::new();
        let n0 = mesh.add_node(0.0, 0.0, 0.0);
        let n1 = mesh.add_node(10.0, 0.0, 0.0);
        let path = mesh.find_path(n0, n1);
        assert!(path.is_err());
    }

    #[test]
    fn test_ai_engine_defaults() {
        let mut engine = AiEngine::new();
        assert!(engine.initialize().is_ok());
    }

    #[test]
    fn test_navmesh_node_count() {
        let mut mesh = NavMesh::new();
        assert_eq!(mesh.node_count(), 0);
        mesh.add_node(1.0, 2.0, 3.0);
        assert_eq!(mesh.node_count(), 1);
        mesh.add_node(4.0, 5.0, 6.0);
        assert_eq!(mesh.node_count(), 2);
    }

    #[test]
    fn test_navmesh_clear_nodes() {
        let mut mesh = NavMesh::new();
        mesh.add_node(0.0, 0.0, 0.0);
        mesh.add_node(1.0, 0.0, 0.0);
        assert_eq!(mesh.node_count(), 2);
        mesh.clear_nodes();
        assert_eq!(mesh.node_count(), 0);
    }

    #[test]
    fn test_navmesh_has_node() {
        let mesh = NavMesh::new();
        assert!(!mesh.has_node(0));
    }

    #[test]
    fn test_navmesh_has_edge() {
        let mut mesh = NavMesh::new();
        let n0 = mesh.add_node(0.0, 0.0, 0.0);
        let n1 = mesh.add_node(1.0, 0.0, 0.0);
        assert!(!mesh.has_edge(n0, n1));
        mesh.add_edge(n0, n1);
        assert!(mesh.has_edge(n0, n1));
    }

    #[test]
    fn test_cover_point_setters() {
        let mut cp = CoverPoint::new([0.0; 3], [0.0, 0.0, 1.0]);
        cp.set_position([1.0, 2.0, 3.0]);
        assert_eq!(cp.position, [1.0, 2.0, 3.0]);
        cp.set_occupied(true);
        assert!(cp.is_occupied);
    }

    #[test]
    fn test_ai_engine_stop() {
        let mut engine = AiEngine::new();
        assert!(!engine.is_initialized());
        engine.initialize().unwrap();
        assert!(engine.is_initialized());
        engine.stop();
        assert!(!engine.is_initialized());
    }

    #[test]
    fn test_navmesh_remove_node() {
        let mut mesh = NavMesh::new();
        let n0 = mesh.add_node(0.0, 0.0, 0.0);
        let n1 = mesh.add_node(1.0, 0.0, 0.0);
        let _n2 = mesh.add_node(2.0, 0.0, 0.0);
        mesh.add_edge(n0, n1);
        assert_eq!(mesh.node_count(), 3);
        assert!(mesh.remove_node(n1));
        assert_eq!(mesh.node_count(), 2);
        assert!(!mesh.remove_node(99));
    }
}

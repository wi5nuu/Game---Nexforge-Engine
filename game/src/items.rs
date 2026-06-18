use crate::game::{ARENA_SIZE, COLLECT_RADIUS, ITEM_COUNT};
use rand::Rng;

#[derive(Clone)]
pub struct Item {
    pub position: [f32; 3],
    pub collected: bool,
    pub phase: f32,
    pub is_health: bool,
}

pub struct Items {
    pub items: Vec<Item>,
}

impl Items {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let half = ARENA_SIZE / 2.0 - 1.5;
        let mut items = Vec::new();
        for i in 0..ITEM_COUNT {
            let angle = (i as f32 / ITEM_COUNT as f32) * std::f32::consts::TAU;
            let radius = rng.gen_range(2.0..half);
            items.push(Item {
                position: [angle.cos() * radius, 0.5, angle.sin() * radius],
                collected: false,
                phase: rng.gen_range(0.0..std::f32::consts::TAU),
                is_health: i % 3 == 0,
            });
        }
        Self { items }
    }

    pub fn reset(&mut self) {
        let mut rng = rand::thread_rng();
        let half = ARENA_SIZE / 2.0 - 1.5;
        for i in 0..ITEM_COUNT as usize {
            let angle = (i as f32 / ITEM_COUNT as f32) * std::f32::consts::TAU;
            let radius = rng.gen_range(2.0..half);
            self.items[i] = Item {
                position: [angle.cos() * radius, 0.5, angle.sin() * radius],
                collected: false,
                phase: rng.gen_range(0.0..std::f32::consts::TAU),
                is_health: i % 3 == 0,
            };
        }
    }

    pub fn check_collection(&mut self, player_pos: [f32; 3]) -> Vec<usize> {
        let mut collected = Vec::new();
        for (idx, item) in self.items.iter_mut().enumerate() {
            if item.collected {
                continue;
            }
            let dx = player_pos[0] - item.position[0];
            let dz = player_pos[2] - item.position[2];
            let dist = (dx * dx + dz * dz).sqrt();
            if dist < COLLECT_RADIUS {
                item.collected = true;
                collected.push(idx);
            }
        }
        collected
    }

    pub fn update(&mut self, dt: f64) {
        let dt32 = dt as f32;
        for item in &mut self.items {
            item.phase += dt32 * 2.0;
        }
    }
}

impl Default for Items {
    fn default() -> Self {
        Self::new()
    }
}

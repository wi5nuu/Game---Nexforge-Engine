use crate::game::{ARENA_SIZE, ENEMY_SPEED};

#[derive(Clone)]
pub struct Enemy {
    pub position: [f32; 3],
    pub health: f32,
    pub max_health: f32,
    pub active: bool,
    pub waypoints: [[f32; 3]; 3],
    pub current_waypoint: usize,
    pub stun_timer: f64,
    pub phase: f32,
    pub chase_mode: bool,
    pub speed_mult: f32,
    pub shoot_cooldown: f64,
    pub shoot_interval: f64,
    pub can_shoot: bool,
    pub enemy_type: u32,
}

pub struct Enemies {
    pub enemies: Vec<Enemy>,
}

impl Enemies {
    pub fn new_for_wave(count: u32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let half = ARENA_SIZE / 2.0 - 2.0;
        let mut enemies = Vec::new();
        for i in 0..count {
            let x = rng.gen_range(-half..half);
            let z = rng.gen_range(-half..half);
            let waypoints = [
                [x, 0.5, z],
                [(x + rng.gen_range(-3.0..3.0)).clamp(-half, half), 0.5, (z + rng.gen_range(-3.0..3.0)).clamp(-half, half)],
                [(x + rng.gen_range(-3.0..3.0)).clamp(-half, half), 0.5, (z + rng.gen_range(-3.0..3.0)).clamp(-half, half)],
            ];
            let (health, speed_mult, shoot_interval) = match i % 3 {
                0 => (30.0, 1.4, 1.2),
                1 => (50.0, 1.0, 2.0),
                _ => (80.0, 0.7, 3.0),
            };
            enemies.push(Enemy {
                position: waypoints[0],
                health, max_health: health, active: true, waypoints,
                current_waypoint: 1, stun_timer: 0.0,
                phase: rng.gen_range(0.0..std::f32::consts::TAU),
                chase_mode: false, speed_mult, shoot_cooldown: rng.gen_range(0.5..shoot_interval), shoot_interval, can_shoot: false,
                enemy_type: i % 3,
            });
        }
        Self { enemies }
    }

    pub fn update(&mut self, dt: f64, player_pos: [f32; 3]) -> (f32, Vec<([f32;3], [f32;3])>) {
        let mut damage = 0.0;
        let mut enemy_bullets = Vec::new();
        let half = ARENA_SIZE / 2.0 - 1.0;
        for enemy in &mut self.enemies {
            if !enemy.active { continue; }
            enemy.can_shoot = false;
            let dx = player_pos[0] - enemy.position[0];
            let dz = player_pos[2] - enemy.position[2];
            let dist = (dx * dx + dz * dz).sqrt();
            if dist < 1.8 && enemy.stun_timer <= 0.0 {
                damage += 5.0 * enemy.speed_mult;
            }
            let chase_range = 8.0;
            enemy.chase_mode = dist < chase_range;
            let speed = ENEMY_SPEED * enemy.speed_mult;
            if enemy.chase_mode {
                let len = dist.max(0.01);
                let move_x = dx / len * speed * dt as f32;
                let move_z = dz / len * speed * dt as f32;
                enemy.position[0] += move_x;
                enemy.position[2] += move_z;
            } else {
                let wp = enemy.waypoints[enemy.current_waypoint];
                let wdx = wp[0] - enemy.position[0];
                let wdz = wp[2] - enemy.position[2];
                let wdist = (wdx * wdx + wdz * wdz).sqrt();
                if wdist < 0.5 { enemy.current_waypoint = (enemy.current_waypoint + 1) % 3; }
                let len = wdist.max(0.01);
                enemy.position[0] += wdx / len * speed * dt as f32 * 0.5;
                enemy.position[2] += wdz / len * speed * dt as f32 * 0.5;
            }
            enemy.shoot_cooldown -= dt;
            if enemy.chase_mode && dist > 2.0 && enemy.shoot_cooldown <= 0.0 && enemy.stun_timer <= 0.0 {
                enemy.can_shoot = true;
                enemy.shoot_cooldown = enemy.shoot_interval;
                let len = dist.max(0.01);
                enemy_bullets.push((
                    [enemy.position[0], enemy.position[1] + 1.0, enemy.position[2]],
                    [dx / len, 0.0, dz / len],
                ));
            }
            if enemy.stun_timer > 0.0 { enemy.stun_timer -= dt; }
            enemy.phase += dt as f32 * 2.0;
            enemy.position[0] = enemy.position[0].clamp(-half, half);
            enemy.position[2] = enemy.position[2].clamp(-half, half);
        }
        (damage, enemy_bullets)
    }

    pub fn clear(&mut self) {
        self.enemies.clear();
    }

    pub fn spawn_for_wave(&mut self, count: u32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let half = ARENA_SIZE / 2.0 - 2.0;
        for i in 0..count {
            let x = rng.gen_range(-half..half);
            let z = rng.gen_range(-half..half);
            let waypoints = [
                [x, 0.5, z],
                [(x + rng.gen_range(-3.0..3.0)).clamp(-half, half), 0.5, (z + rng.gen_range(-3.0..3.0)).clamp(-half, half)],
                [(x + rng.gen_range(-3.0..3.0)).clamp(-half, half), 0.5, (z + rng.gen_range(-3.0..3.0)).clamp(-half, half)],
            ];
            let (health, speed_mult, shoot_interval) = match i % 3 {
                0 => (30.0, 1.4, 1.2),
                1 => (50.0, 1.0, 2.0),
                _ => (80.0, 0.7, 3.0),
            };
            self.enemies.push(Enemy {
                position: waypoints[0],
                health, max_health: health, active: true, waypoints,
                current_waypoint: 1, stun_timer: 0.0,
                phase: rng.gen_range(0.0..std::f32::consts::TAU),
                chase_mode: false, speed_mult,
                shoot_cooldown: rng.gen_range(0.5..shoot_interval), shoot_interval, can_shoot: false,
                enemy_type: i % 3,
            });
        }
    }
}

impl Default for Enemies {
    fn default() -> Self {
        Self::new_for_wave(0)
    }
}

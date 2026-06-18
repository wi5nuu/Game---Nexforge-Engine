use crate::game::ARENA_SIZE;

const PLAYER_BULLET_SPEED: f32 = 28.0;
const ENEMY_BULLET_SPEED: f32 = 14.0;
const BULLET_LIFETIME: f64 = 2.5;

#[derive(Clone)]
pub struct Bullet {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub from_player: bool,
    pub lifetime: f64,
}

pub struct Bullets {
    pub bullets: Vec<Bullet>,
}

impl Bullets {
    pub fn new() -> Self {
        Self { bullets: Vec::new() }
    }

    pub fn fire_player(&mut self, origin: [f32; 3], direction: [f32; 3]) {
        self.bullets.push(Bullet {
            position: origin,
            velocity: [direction[0] * PLAYER_BULLET_SPEED, direction[1] * PLAYER_BULLET_SPEED, direction[2] * PLAYER_BULLET_SPEED],
            from_player: true,
            lifetime: BULLET_LIFETIME,
        });
    }

    pub fn fire_enemy(&mut self, origin: [f32; 3], direction: [f32; 3]) {
        self.bullets.push(Bullet {
            position: origin,
            velocity: [direction[0] * ENEMY_BULLET_SPEED, direction[1] * ENEMY_BULLET_SPEED, direction[2] * ENEMY_BULLET_SPEED],
            from_player: false,
            lifetime: BULLET_LIFETIME,
        });
    }

    pub fn update(&mut self, dt: f64) {
        let dt32 = dt as f32;
        let half = ARENA_SIZE / 2.0 - 0.5;
        self.bullets.retain_mut(|bullet| {
            bullet.lifetime -= dt;
            if bullet.lifetime <= 0.0 { return false; }
            bullet.position[0] += bullet.velocity[0] * dt32;
            bullet.position[1] += bullet.velocity[1] * dt32;
            bullet.position[2] += bullet.velocity[2] * dt32;
            if bullet.position[0].abs() > half || bullet.position[2].abs() > half { return false; }
            if bullet.position[1] < 0.0 || bullet.position[1] > 4.0 { return false; }
            true
        });
    }

    pub fn clear(&mut self) {
        self.bullets.clear();
    }
}

impl Default for Bullets {
    fn default() -> Self {
        Self::new()
    }
}

use crate::game::ARENA_SIZE;
use nexforge_renderer::camera::Camera;

pub struct Player {
    pub position: [f32; 3],
    pub health: f32,
    pub max_health: f32,
    pub hit_timer: f64,
    pub invulnerable: bool,
    pub invulnerable_timer: f64,
    pub speed: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.5, 0.0],
            health: 100.0,
            max_health: 100.0,
            hit_timer: 0.0,
            invulnerable: false,
            invulnerable_timer: 0.0,
            speed: 6.0,
        }
    }

    pub fn reset(&mut self) {
        self.position = [0.0, 0.5, 0.0];
        self.health = self.max_health;
        self.hit_timer = 0.0;
        self.invulnerable = false;
        self.invulnerable_timer = 0.0;
    }

    pub fn take_damage(&mut self, amount: f32) -> bool {
        if self.invulnerable {
            return false;
        }
        self.health = (self.health - amount).max(0.0);
        self.hit_timer = 0.3;
        self.invulnerable = true;
        self.invulnerable_timer = 1.0;
        true
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn update(&mut self, dt: f64, horizontal: f32, vertical: f32, sprint: bool, camera: &Camera) {
        let half = ARENA_SIZE / 2.0 - 0.5;
        let dt32 = dt as f32;
        let speed = if sprint { self.speed * 2.0 } else { self.speed };
        if vertical != 0.0 || horizontal != 0.0 {
            let fwd = camera.forward();
            let right = camera.right();
            let len_fwd = (fwd[0] * fwd[0] + fwd[2] * fwd[2]).sqrt().max(0.01);
            let flat_fwd = [fwd[0] / len_fwd, 0.0, fwd[2] / len_fwd];
            let len_right = (right[0] * right[0] + right[2] * right[2]).sqrt().max(0.01);
            let flat_right = [right[0] / len_right, 0.0, right[2] / len_right];
            let input_len = (horizontal * horizontal + vertical * vertical).sqrt();
            let nx = horizontal / input_len;
            let nz = vertical / input_len;
            self.position[0] += (flat_right[0] * nx + flat_fwd[0] * nz) * speed * dt32;
            self.position[2] += (flat_right[2] * nx + flat_fwd[2] * nz) * speed * dt32;
        }
        self.position[0] = self.position[0].clamp(-half, half);
        self.position[2] = self.position[2].clamp(-half, half);
        self.hit_timer = (self.hit_timer - dt).max(0.0);
        if self.invulnerable {
            self.invulnerable_timer -= dt;
            if self.invulnerable_timer <= 0.0 {
                self.invulnerable = false;
            }
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    Victory,
}

pub const ARENA_SIZE: f32 = 14.0;
pub const ITEM_COUNT: u32 = 8;
pub const ENEMY_SPEED: f32 = 2.5;
pub const COLLECT_RADIUS: f32 = 1.5;
pub const WALL_HEIGHT: f32 = 3.0;
pub const WALL_THICKNESS: f32 = 0.4;
pub const PLAYER_BULLET_DAMAGE: f32 = 25.0;
pub const ENEMY_BULLET_DAMAGE: f32 = 10.0;
pub const MAX_AMMO: u32 = 30;
pub const RELOAD_TIME: f64 = 1.5;

pub fn enemies_for_wave(wave: u32) -> u32 {
    match wave {
        1 => 3,
        2 => 5,
        _ => 7,
    }
}

pub struct MiniGame {
    pub state: GameState,
    pub health: f32,
    pub max_health: f32,
    pub score: u32,
    pub collected_items: u32,
    pub game_time: f64,
    pub message: String,
    pub message_timer: f64,
    pub flash_timer: f64,
    pub attack_flash: f64,
    pub ammo: u32,
    pub reloading: bool,
    pub reload_timer: f64,
    pub current_wave: u32,
    pub enemies_in_wave: u32,
    pub enemies_spawned: u32,
    pub enemies_alive: u32,
    pub wave_cooldown: f64,
    pub kills: u32,
}

impl MiniGame {
    pub fn new() -> Self {
        Self {
            state: GameState::Menu,
            health: 100.0,
            max_health: 100.0,
            score: 0,
            collected_items: 0,
            game_time: 0.0,
            message: String::new(),
            message_timer: 0.0,
            flash_timer: 0.0,
            attack_flash: 0.0,
            ammo: MAX_AMMO,
            reloading: false,
            reload_timer: 0.0,
            current_wave: 1,
            enemies_in_wave: enemies_for_wave(1),
            enemies_spawned: 0,
            enemies_alive: 0,
            wave_cooldown: 0.0,
            kills: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state = GameState::Playing;
        self.health = self.max_health;
        self.score = 0;
        self.collected_items = 0;
        self.game_time = 0.0;
        self.message.clear();
        self.message_timer = 0.0;
        self.flash_timer = 0.0;
        self.attack_flash = 0.0;
        self.ammo = MAX_AMMO;
        self.reloading = false;
        self.reload_timer = 0.0;
        self.current_wave = 1;
        self.enemies_in_wave = enemies_for_wave(1);
        self.enemies_spawned = 0;
        self.enemies_alive = 0;
        self.wave_cooldown = 0.0;
        self.kills = 0;
    }

    pub fn take_damage(&mut self, amount: f32) {
        if self.health > 0.0 {
            self.health = (self.health - amount).max(0.0);
            self.flash_timer = 0.3;
            if self.health <= 0.0 {
                self.state = GameState::GameOver;
                self.message = "Game Over! Press ENTER to restart".to_string();
                self.message_timer = 999.0;
            }
        }
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn start_reload(&mut self) {
        if self.ammo < MAX_AMMO && !self.reloading {
            self.reloading = true;
            self.reload_timer = RELOAD_TIME;
        }
    }

    pub fn try_shoot(&mut self) -> bool {
        if self.reloading { return false; }
        if self.ammo == 0 {
            self.start_reload();
            return false;
        }
        self.ammo -= 1;
        self.attack_flash = 0.18;
        true
    }

    pub fn defeat_enemy(&mut self) {
        self.kills += 1;
        self.enemies_alive = self.enemies_alive.saturating_sub(1);
        self.score += 100;
    }

    pub fn set_message(&mut self, msg: String, duration: f64) {
        self.message = msg;
        self.message_timer = duration;
    }

    pub fn update(&mut self, dt: f64) {
        self.game_time += dt;
        if self.message_timer > 0.0 {
            self.message_timer -= dt;
            if self.message_timer <= 0.0 { self.message.clear(); }
        }
        self.flash_timer = (self.flash_timer - dt).max(0.0);
        self.attack_flash = (self.attack_flash - dt).max(0.0);
        if self.reloading {
            self.reload_timer -= dt;
            if self.reload_timer <= 0.0 {
                self.reloading = false;
                self.ammo = MAX_AMMO;
            }
        }
    }}

impl Default for MiniGame {
    fn default() -> Self {
        Self::new()
    }
}

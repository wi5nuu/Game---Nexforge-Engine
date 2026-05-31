#![deny(clippy::all)]

use nexscript::ScriptRuntime;

pub struct GameManagerSystem {
    pub score: i32,
    pub round: i32,
    pub enemies_per_round: i32,
    pub is_game_over: bool,
}

impl GameManagerSystem {
    pub fn new() -> Self {
        Self { score: 0, round: 1, enemies_per_round: 5, is_game_over: false }
    }

    pub fn initialize(&mut self, runtime: &mut ScriptRuntime) {
        if let Some(&id) = runtime.entities.iter()
            .find(|(_, e)| e.name == "GameManager")
            .map(|(id, _)| id)
        {
            let _ = runtime.fire_event(id, &nexscript::ScriptEvent::Spawn);
        }
    }

    pub fn add_score(&mut self, runtime: &mut ScriptRuntime, points: i32) {
        self.score += points;
        self.enemies_per_round = 5 + (self.round - 1) * 2;
    }

    pub fn start_round(&mut self, _runtime: &mut ScriptRuntime) {
        self.round += 1;
        self.enemies_per_round = 5 + (self.round - 1) * 2;
    }

    pub fn update(&mut self, runtime: &mut ScriptRuntime, dt: f32) {
        if self.is_game_over { return; }
        if let Some(&id) = runtime.entities.iter()
            .find(|(_, e)| e.name == "GameManager")
            .map(|(id, _)| id)
        {
            let _ = runtime.fire_event(id, &nexscript::ScriptEvent::Update(dt));
        }
    }
}

impl Default for GameManagerSystem { fn default() -> Self { Self::new() } }

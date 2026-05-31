#![deny(clippy::all)]

use nexscript::ScriptRuntime;

pub struct EnemySystem {
    pub enemy_ids: Vec<u64>,
}

impl EnemySystem {
    pub fn new() -> Self { Self { enemy_ids: Vec::new() } }

    pub fn initialize(&mut self, runtime: &mut ScriptRuntime) {
        self.enemy_ids = runtime.entities.iter()
            .filter(|(_, e)| e.name == "Enemy")
            .map(|(&id, _)| id)
            .collect();
    }

    pub fn update(&mut self, runtime: &mut ScriptRuntime, dt: f32) {
        for &id in &self.enemy_ids {
            let _ = runtime.fire_event(id, &nexscript::ScriptEvent::Update(dt));
        }
    }

    pub fn kill_enemy(&mut self, runtime: &mut ScriptRuntime, id: u64) {
        let _ = runtime.kill_entity(id);
        self.enemy_ids.retain(|&eid| eid != id);
    }
}

impl Default for EnemySystem { fn default() -> Self { Self::new() } }

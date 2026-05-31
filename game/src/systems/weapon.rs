#![deny(clippy::all)]

use nexscript::ScriptRuntime;

pub struct WeaponSystem {
    pub cooldowns: Vec<(u64, f32)>,
}

impl WeaponSystem {
    pub fn new() -> Self { Self { cooldowns: Vec::new() } }

    pub fn fire(&mut self, runtime: &mut ScriptRuntime, entity_id: u64) {
        // Weapon fire delegates to NexScript script
        let _ = runtime.fire_event(entity_id, &nexscript::ScriptEvent::Update(0.0));
    }

    pub fn update(&mut self, _runtime: &mut ScriptRuntime, dt: f32) {
        self.cooldowns.iter_mut().for_each(|(_, cd)| *cd -= dt);
        self.cooldowns.retain(|(_, cd)| *cd > 0.0);
    }
}

impl Default for WeaponSystem { fn default() -> Self { Self::new() } }

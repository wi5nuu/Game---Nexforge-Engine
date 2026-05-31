use nexscript::{ScriptRuntime, InputState};

pub struct PlayerSystem {
    pub player_id: Option<u64>,
    pub speed: f32,
    pub mouse_sensitivity: f32,
    pub jump_force: f32,
}

impl PlayerSystem {
    pub fn new() -> Self {
        Self { player_id: None, speed: 6.0, mouse_sensitivity: 0.002, jump_force: 8.0 }
    }

    pub fn initialize(&mut self, runtime: &mut ScriptRuntime) {
        self.player_id = runtime.player_entity;
    }

    pub fn update(&mut self, runtime: &mut ScriptRuntime, dt: f32, input: &InputState) {
        runtime.set_input(InputState {
            horizontal: input.horizontal,
            vertical: input.vertical,
            mouse_x: input.mouse_x,
            mouse_y: input.mouse_y,
            jump: input.jump,
            shoot: input.shoot,
            reload: input.reload,
            sprint: input.sprint,
            crouch: input.crouch,
        });

        if let Some(pid) = self.player_id {
            let _ = runtime.fire_event(pid, &nexscript::ScriptEvent::Update(dt));
        }
    }
}

impl Default for PlayerSystem { fn default() -> Self { Self::new() } }

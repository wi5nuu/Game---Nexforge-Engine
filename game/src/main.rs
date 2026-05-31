#![deny(clippy::all)]

mod systems;

use nexscript::{ScriptRuntime, InputState};
use systems::{PlayerSystem, EnemySystem, WeaponSystem, GameManagerSystem};

fn main() {
    env_logger::init();
    log::info!("Nexforge Engine — Game starting...");

    // Initialize Script Runtime
    let mut runtime = ScriptRuntime::new();
    let script_files = vec![
        "game/scripts/player.nxs",
        "game/scripts/enemy_ai.nxs",
        "game/scripts/weapon.nxs",
        "game/scripts/game_manager.nxs",
    ];

    for script_path in &script_files {
        match runtime.load_script_file(script_path) {
            Ok(()) => log::info!("Loaded script: {}", script_path),
            Err(e) => log::warn!("Failed to load {}: {}", script_path, e),
        }
    }

    log::info!("Loaded {} entity definitions", runtime.entities.len());

    // Initialize Game Systems
    let mut player = PlayerSystem::new();
    let mut enemy = EnemySystem::new();
    let mut weapon = WeaponSystem::new();
    let mut game_manager = GameManagerSystem::new();

    player.initialize(&mut runtime);
    enemy.initialize(&mut runtime);
    game_manager.initialize(&mut runtime);

    // Spawn player
    if let Some(pid) = runtime.player_entity {
        let _ = runtime.fire_event(pid, &nexscript::ScriptEvent::Spawn);
        log::info!("Player spawned (entity #{})", pid);
    }

    // Simulate game loop (20 frames for demo)
    let dt = 1.0 / 60.0;
    let mut input = InputState::new();

    for frame in 0..20 {
        // Simulate input
        input.horizontal = 0.5;
        input.vertical = 1.0;
        if frame == 5 {
            input.jump = true;
        } else {
            input.jump = false;
        }
        if frame % 10 == 0 {
            input.shoot = true;
        } else {
            input.shoot = false;
        }

        // Update systems
        player.update(&mut runtime, dt, &input);
        enemy.update(&mut runtime, dt);
        weapon.update(&mut runtime, dt);
        game_manager.update(&mut runtime, dt);

        log::info!(
            "Frame {}: entities={}, score={}, round={}",
            frame,
            runtime.entities.len(),
            game_manager.score,
            game_manager.round,
        );
    }

    log::info!("Game demo complete.");
}

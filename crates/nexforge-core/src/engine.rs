#![deny(clippy::all)]

use std::collections::HashMap;
use std::time::Instant;

use thiserror::Error;

use nexforge_ecs::{Entity, World, SystemScheduler};
use nexforge_renderer::pipeline::{RenderContext, RenderError};
use nexforge_physics::{PhysicsEngine, PhysicsError};
use nexforge_audio::{AudioEngine, AudioError};
use nexforge_ai::AiEngine;
use nexforge_net::NetEngine;
use nexscript::{ScriptRuntime, EntityId, InputState};
use nexforge_dev_console::DevConsole;
use nexforge_profiler::FrameProfiler;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Renderer: {0}")]
    Renderer(#[from] RenderError),
    #[error("Physics: {0}")]
    Physics(#[from] PhysicsError),
    #[error("Audio: {0}")]
    Audio(#[from] AudioError),
    #[error("Script: {0}")]
    Script(String),
    #[error("Window: {0}")]
    Window(String),
}

impl From<String> for EngineError {
    fn from(s: String) -> Self { EngineError::Script(s) }
}

pub struct EntityMapper {
    ecs_to_nex: HashMap<Entity, EntityId>,
    nex_to_ecs: HashMap<EntityId, Entity>,
    next_nex_id: EntityId,
}

impl EntityMapper {
    pub fn new() -> Self {
        Self { ecs_to_nex: HashMap::new(), nex_to_ecs: HashMap::new(), next_nex_id: 1 }
    }

    pub fn register(&mut self, ecs: Entity) -> EntityId {
        let nex = self.next_nex_id;
        self.next_nex_id += 1;
        self.ecs_to_nex.insert(ecs, nex);
        self.nex_to_ecs.insert(nex, ecs);
        nex
    }

    pub fn nex_to_ecs(&self, nex: EntityId) -> Option<Entity> {
        self.nex_to_ecs.get(&nex).copied()
    }

    pub fn ecs_to_nex(&self, ecs: &Entity) -> Option<EntityId> {
        self.ecs_to_nex.get(ecs).copied()
    }

    pub fn remove_ecs(&mut self, ecs: &Entity) {
        if let Some(nex) = self.ecs_to_nex.remove(ecs) {
            self.nex_to_ecs.remove(&nex);
        }
    }
}

impl Default for EntityMapper { fn default() -> Self { Self::new() } }

pub enum EngineMode {
    Editor,
    Game,
    Headless,
}

pub struct Engine<'a> {
    pub running: bool,
    pub frame_count: u64,
    pub fixed_timestep: f64,
    pub accumulator: f64,
    pub mode: EngineMode,
    pub last_frame_time: Instant,

    pub world: World,
    pub scheduler: SystemScheduler,
    pub entity_mapper: EntityMapper,

    pub renderer: Option<RenderContext<'a>>,
    pub physics: PhysicsEngine,
    pub audio: AudioEngine,
    pub ai: AiEngine,
    pub net: Option<NetEngine>,

    pub script: ScriptRuntime,
    pub console: DevConsole,
    pub profiler: FrameProfiler,
}

impl<'a> Engine<'a> {
    pub fn new(mode: EngineMode) -> Self {
        Self {
            running: false,
            frame_count: 0,
            fixed_timestep: 1.0 / 60.0,
            accumulator: 0.0,
            mode,
            last_frame_time: Instant::now(),
            world: World::new(),
            scheduler: SystemScheduler::new(),
            entity_mapper: EntityMapper::new(),
            renderer: None,
            physics: PhysicsEngine::new(),
            audio: AudioEngine::new(),
            ai: AiEngine::new(),
            net: None,
            script: ScriptRuntime::new(),
            console: DevConsole::new(),
            profiler: FrameProfiler::new(256),
        }
    }

    pub fn initialize(&mut self) -> Result<(), EngineError> {
        log::info!("Nexforge Engine v{} initializing...", env!("CARGO_PKG_VERSION"));
        self.physics.initialize()?;
        self.audio.initialize()?;
        self.ai.initialize().ok();
        Ok(())
    }

    pub fn init_renderer(&mut self, window: &'a winit::window::Window) -> Result<(), EngineError> {
        let size = window.inner_size();
        let aspect = if size.height > 0 { size.width as f32 / size.height as f32 } else { 16.0 / 9.0 };
        let mut renderer = RenderContext::new(aspect);
        renderer.initialize(window)?;
        self.renderer = Some(renderer);
        Ok(())
    }

    pub fn load_script(&mut self, path: &str) -> Result<(), EngineError> {
        self.script.load_script_file(path)?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), EngineError> {
        self.running = true;
        self.frame_count = 0;
        self.accumulator = 0.0;
        self.last_frame_time = Instant::now();
        self.profiler.begin_frame();
        self.profiler.end_frame();
        log::info!("Engine entering main loop");
        Ok(())
    }

    pub fn fixed_update(&mut self, dt: f32) {
        self.profiler.begin_sample("physics", Instant::now());
        self.physics.step(dt);
        self.profiler.end_sample("physics", Instant::now(), Instant::now().elapsed());

        self.profiler.begin_sample("script_update", Instant::now());
        let _ = self.script.update_all(dt);
        self.profiler.end_sample("script_update", Instant::now(), Instant::now().elapsed());

        self.profiler.begin_sample("ecs_systems", Instant::now());
        self.scheduler.run(&mut self.world);
        self.profiler.end_sample("ecs_systems", Instant::now(), Instant::now().elapsed());
    }

    pub fn handle_input(&mut self, input: &InputState) {
        self.script.set_input(InputState {
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
    }

    pub fn tick(&mut self, dt: f64) {
        self.profiler.begin_frame();
        self.frame_count += 1;
        self.accumulator += dt;

        while self.accumulator >= self.fixed_timestep {
            self.fixed_update(self.fixed_timestep as f32);
            self.accumulator -= self.fixed_timestep;
        }

        self.profiler.end_frame();
    }

    pub fn shutdown(&mut self) {
        self.running = false;
        if let Ok(json) = self.profiler.export_json() {
            log::info!("Profiler data:\n{}", json);
        }
        log::info!("Engine shutdown complete");
    }

    pub fn toggle_console(&mut self) { self.console.toggle(); }
    pub fn toggle_profiler(&mut self) { self.profiler.toggle(); }
    pub fn console_active(&self) -> bool { self.console.visible }
    pub fn profiler_active(&self) -> bool { self.profiler.visible }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new(EngineMode::Headless);
        assert!(!engine.running);
        assert_eq!(engine.frame_count, 0);
    }

    #[test]
    fn test_engine_initialize() {
        let mut engine = Engine::new(EngineMode::Headless);
        assert!(engine.initialize().is_ok());
    }

    #[test]
    fn test_entity_mapper() {
        let mut mapper = EntityMapper::new();
        let e1 = Entity::new();
        let id = mapper.register(e1);
        assert_eq!(mapper.ecs_to_nex(&e1), Some(id));
        assert_eq!(mapper.nex_to_ecs(id), Some(e1));
    }

    #[test]
    fn test_entity_mapper_remove() {
        let mut mapper = EntityMapper::new();
        let e = Entity::new();
        let id = mapper.register(e);
        mapper.remove_ecs(&e);
        assert!(mapper.ecs_to_nex(&e).is_none());
        assert!(mapper.nex_to_ecs(id).is_none());
    }

    #[test]
    fn test_tick_increments_frame() {
        let mut engine = Engine::new(EngineMode::Headless);
        let _ = engine.initialize();
        engine.tick(0.016);
        assert_eq!(engine.frame_count, 1);
    }

    #[test]
    fn test_tick_fixed_update() {
        let mut engine = Engine::new(EngineMode::Headless);
        let _ = engine.initialize();
        engine.tick(0.05); // ~3 fixed steps
        assert!(engine.frame_count > 0);
    }

    #[test]
    fn test_start_stop() {
        let mut engine = Engine::new(EngineMode::Headless);
        let _ = engine.initialize();
        assert!(engine.start().is_ok());
        assert!(engine.running);
        engine.shutdown();
        assert!(!engine.running);
    }

    #[test]
    fn test_toggle_console() {
        let mut engine = Engine::new(EngineMode::Headless);
        assert!(!engine.console_active());
        engine.toggle_console();
        assert!(engine.console_active());
    }

    #[test]
    fn test_toggle_profiler() {
        let mut engine = Engine::new(EngineMode::Headless);
        assert!(!engine.profiler_active());
        engine.toggle_profiler();
        assert!(engine.profiler_active());
    }
}

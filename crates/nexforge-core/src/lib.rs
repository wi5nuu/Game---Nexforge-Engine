#![deny(clippy::all)]

pub mod engine;

pub use engine::Engine;
pub use engine::EngineError;
pub use engine::EngineMode;
pub use engine::EntityMapper;

#[cfg(test)]
mod tests {
    use super::*;
    use nexforge_ecs::Entity;

    #[test]
    fn test_engine_mode_game() {
        let engine = Engine::new(EngineMode::Game);
        assert!(matches!(engine.mode, EngineMode::Game));
    }

    #[test]
    fn test_engine_mode_editor() {
        let engine = Engine::new(EngineMode::Editor);
        assert!(matches!(engine.mode, EngineMode::Editor));
    }

    #[test]
    fn test_engine_mode_headless() {
        let engine = Engine::new(EngineMode::Headless);
        assert!(matches!(engine.mode, EngineMode::Headless));
    }

    #[test]
    fn test_entity_mapper_default() {
        let mapper = EntityMapper::new();
        assert_eq!(mapper.nex_to_ecs(1), None);
    }

    #[test]
    fn test_entity_mapper_roundtrip() {
        let mut mapper = EntityMapper::new();
        let e = Entity::new();
        let nex_id = mapper.register(e);
        assert_eq!(mapper.nex_to_ecs(nex_id), Some(e));
        assert_eq!(mapper.ecs_to_nex(&e), Some(nex_id));
    }

    #[test]
    fn test_engine_tick_fixed_timestep() {
        let mut engine = Engine::new(EngineMode::Headless);
        assert!((engine.fixed_timestep - 1.0 / 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_engine_frame_count_after_tick() {
        let mut engine = Engine::new(EngineMode::Headless);
        let _ = engine.initialize();
        assert_eq!(engine.frame_count, 0);
        engine.start().ok();
        engine.tick(0.016);
        assert_eq!(engine.frame_count, 1);
    }

    #[test]
    fn test_version_string() {
        let engine = Engine::new(EngineMode::Headless);
        let ver = engine.version_string();
        assert!(ver.contains("Nexforge Engine v"));
    }

    #[test]
    fn test_set_fixed_timestep() {
        let mut engine = Engine::new(EngineMode::Headless);
        assert!((engine.fixed_timestep - 1.0 / 60.0).abs() < f64::EPSILON);
        engine.set_fixed_timestep(1.0 / 30.0);
        assert!((engine.fixed_timestep - 1.0 / 30.0).abs() < f64::EPSILON);
    }
}

#![deny(clippy::all)]

use crate::ast::*;
use crate::compiler::{Bytecode, Compiler};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::vm::{Value, Vm};
use std::collections::HashMap;

pub type EntityId = u64;

pub struct InputState {
    pub horizontal: f32,
    pub vertical: f32,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub jump: bool,
    pub shoot: bool,
    pub reload: bool,
    pub sprint: bool,
    pub crouch: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self { horizontal: 0.0, vertical: 0.0, mouse_x: 0.0, mouse_y: 0.0, jump: false, shoot: false, reload: false, sprint: false, crouch: false }
    }
}

impl Default for InputState { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone)]
pub enum ScriptEvent {
    Update(f32),
    Spawn,
    Death,
    Collision(EntityId),
    TriggerEnter(EntityId),
    TriggerExit(EntityId),
}

pub struct ScriptContext {
    pub input: InputState,
    pub global_vars: HashMap<String, Value>,
    pub query_results: Vec<EntityId>,
}

impl ScriptContext {
    pub fn new() -> Self { Self { input: InputState::new(), global_vars: HashMap::new(), query_results: Vec::new() } }
}

impl Default for ScriptContext { fn default() -> Self { Self::new() } }

pub struct ScriptEntity {
    pub id: EntityId,
    pub name: String,
    pub components: Vec<String>,
    pub component_values: HashMap<String, HashMap<String, f64>>,
    pub event_handlers: HashMap<String, (Vec<Bytecode>, Vec<String>)>,
    pub state_vars: HashMap<String, Value>,
}

pub struct ScriptRuntime {
    pub entities: HashMap<EntityId, ScriptEntity>,
    pub scripts: Vec<String>,
    pub context: ScriptContext,
    pub vm: Vm,
    pub string_pool: Vec<String>,
    pub bytecode: Vec<Bytecode>,
    pub next_entity_id: EntityId,
    pub player_entity: Option<EntityId>,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            scripts: Vec::new(),
            context: ScriptContext::new(),
            vm: Vm::new(vec![Bytecode::Halt], vec![]),
            string_pool: vec![],
            bytecode: vec![Bytecode::Halt],
            next_entity_id: 1,
            player_entity: None,
        }
    }

    pub fn load_script(&mut self, source: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| e.to_string())?;

        let mut compiler = Compiler::new();
        let (bytecode, string_pool) = compiler.compile(&ast).map_err(|e| e.to_string())?;
        self.bytecode = bytecode;
        self.string_pool = string_pool;

        // Extract entity definitions from AST
        self.extract_entities(&ast)?;

        self.vm = Vm::new(self.bytecode.clone(), self.string_pool.clone());
        Ok(())
    }

    pub fn load_script_file(&mut self, path: &str) -> Result<(), String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;
        self.load_script(&source)?;
        self.scripts.push(path.to_string());
        Ok(())
    }

    fn extract_entities(&mut self, ast: &AstNode) -> Result<(), String> {
        match ast {
            AstNode::Program(nodes) => {
                for node in nodes {
                    self.extract_entities(node)?;
                }
            }
            AstNode::EntityDef { name, components, events } => {
                let id = self.next_entity_id;
                self.next_entity_id += 1;
                let mut comp_names = Vec::new();
                let mut comp_values = HashMap::new();

                for comp in components {
                    comp_names.push(comp.name.clone());
                    let mut fields = HashMap::new();
                    for (field_name, field_val) in &comp.fields {
                        if let AstNode::Int(v) = field_val {
                            fields.insert(field_name.clone(), *v as f64);
                        } else if let AstNode::Float(v) = field_val {
                            fields.insert(field_name.clone(), *v);
                        }
                    }
                    comp_values.insert(comp.name.clone(), fields);
                }

                let mut handlers = HashMap::new();
                for event in events {
                    let handler_source = self.event_handler_to_bytecode(&event)?;
                    let param_names: Vec<String> = event.params.iter().map(|p| p.name.clone()).collect();
                    handlers.insert(event.name.clone(), (handler_source, param_names));
                }

                let entity = ScriptEntity {
                    id, name: name.clone(), components: comp_names,
                    component_values: comp_values, event_handlers: handlers,
                    state_vars: HashMap::new(),
                };
                self.entities.insert(id, entity);

                if name == "Player" { self.player_entity = Some(id); }
            }
            _ => {}
        }
        Ok(())
    }

    fn event_handler_to_bytecode(&self, handler: &EventHandler) -> Result<Vec<Bytecode>, String> {
        let body = AstNode::Block(handler.body.clone());
        let fn_wrapper = AstNode::FnDef {
            name: format!("__handler_{}", handler.name),
            params: handler.params.clone(),
            return_type: None,
            is_coroutine: false,
            body: Box::new(body),
        };
        let mut compiler = Compiler::new();
        let prog = AstNode::Program(vec![fn_wrapper]);
        let (bc, _) = compiler.compile(&prog).map_err(|e| e.to_string())?;
        Ok(bc)
    }

    pub fn fire_event(&mut self, entity_id: EntityId, event: &ScriptEvent) -> Result<(), String> {
        let entity = self.entities.get(&entity_id)
            .ok_or_else(|| format!("Entity {} not found", entity_id))?;

        let event_name = match event {
            ScriptEvent::Update(_) => "on_update",
            ScriptEvent::Spawn => "on_spawn",
            ScriptEvent::Death => "on_death",
            ScriptEvent::Collision(_) => "on_collision",
            ScriptEvent::TriggerEnter(_) => "on_trigger_enter",
            ScriptEvent::TriggerExit(_) => "on_trigger_exit",
        };

        if let Some((bytecode, params)) = entity.event_handlers.get(event_name) {
            if !bytecode.is_empty() {
                self.vm = Vm::new(bytecode.clone(), self.string_pool.clone());
                match event {
                    ScriptEvent::Update(dt) => self.context.global_vars.insert("dt".to_string(), Value::Float(*dt as f64)),
                    ScriptEvent::Collision(other) => { self.context.global_vars.insert("other".to_string(), Value::Int(*other as i32)); }
                    _ => {}
                }
                self.vm.run().ok();
            }
        }
        Ok(())
    }

    pub fn update_all(&mut self, dt: f32) -> Result<(), String> {
        let ids: Vec<EntityId> = self.entities.keys().copied().collect();
        for id in ids {
            if self.entities.contains_key(&id) {
                self.fire_event(id, &ScriptEvent::Update(dt))?;
            }
        }
        Ok(())
    }

    pub fn spawn_entity(&mut self, name: &str) -> Option<EntityId> {
        // Find entity definition by name in stored data
        for (&id, entity) in &self.entities {
            if entity.name == name {
                self.fire_event(id, &ScriptEvent::Spawn).ok();
                return Some(id);
            }
        }
        None
    }

    pub fn kill_entity(&mut self, entity_id: EntityId) -> Result<(), String> {
        self.fire_event(entity_id, &ScriptEvent::Death)?;
        Ok(())
    }

    pub fn set_input(&mut self, input: InputState) {
        self.context.input = input;
    }
}

impl Default for ScriptRuntime { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let rt = ScriptRuntime::new();
        assert_eq!(rt.next_entity_id, 1);
    }

    #[test]
    fn test_load_empty_script() {
        let mut rt = ScriptRuntime::new();
        assert!(rt.load_script("").is_ok());
    }

    #[test]
    fn test_load_player_script() {
        let mut rt = ScriptRuntime::new();
        let source = r#"
            entity Player {
                component Transform
                component Health { max: 100, current: 100 }
                on_update(dt: float) {
                    let x = 1;
                }
                on_death() {
                    let x = 2;
                }
            }
        "#;
        rt.load_script(source).unwrap();
        assert_eq!(rt.entities.len(), 1);
        assert!(rt.player_entity.is_some());
    }

    #[test]
    fn test_fire_event() {
        let mut rt = ScriptRuntime::new();
        let source = r#"
            entity Player {
                component Transform
                on_spawn() {
                    let x = 42;
                }
                on_update(dt: float) {
                    let x = dt;
                }
            }
        "#;
        rt.load_script(source).unwrap();
        let pid = rt.player_entity.unwrap();
        assert!(rt.fire_event(pid, &ScriptEvent::Spawn).is_ok());
        assert!(rt.fire_event(pid, &ScriptEvent::Update(0.016)).is_ok());
    }

    #[test]
    fn test_update_all() {
        let mut rt = ScriptRuntime::new();
        let source = r#"
            entity Player {
                on_update(dt: float) {}
            }
            entity Enemy {
                on_update(dt: float) {}
            }
        "#;
        rt.load_script(source).unwrap();
        assert!(rt.update_all(0.016).is_ok());
    }

    #[test]
    fn test_entity_extraction() {
        let mut rt = ScriptRuntime::new();
        let source = r#"
            entity Enemy {
                component Health { max: 50, current: 50 }
                component AIState { current: 0 }
                on_update(dt: float) {}
            }
        "#;
        rt.load_script(source).unwrap();
        let entity = rt.entities.values().next().unwrap();
        assert_eq!(entity.name, "Enemy");
        assert!(entity.components.contains(&"Health".to_string()));
    }
}

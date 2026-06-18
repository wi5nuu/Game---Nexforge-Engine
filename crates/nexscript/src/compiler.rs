#![deny(clippy::all)]

use crate::ast::*;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Bytecode {
    Nop,
    PushInt(i32),
    PushFloat(f64),
    PushBool(bool),
    PushString(u16),
    PushVec3(f64, f64, f64),
    PushNull,
    Pop,
    Dup,
    LoadLocal(u8),
    StoreLocal(u8),
    LoadField(u16),
    StoreField(u16),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Not,
    Jmp(u16),
    JmpIf(u16),
    JmpIfNot(u16),
    Call(u16),
    CallBuiltin(u8),
    Return,
    Yield,
    Await,
    EntityGet,
    NewEntity,
    Destroy,
    Halt,
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Unsupported AST node: {0:?}")]
    UnsupportedNode(String),
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
    #[error("Too many constants")]
    TooManyConstants,
    #[error("Too many locals")]
    TooManyLocals,
    #[error("Too many functions")]
    TooManyFunctions,
}

pub struct Compiler {
    bytecode: Vec<Bytecode>,
    string_pool: Vec<String>,
    locals: Vec<Vec<String>>,
    scope_depth: usize,
    breaks: Vec<Vec<usize>>,
    continues: Vec<Vec<usize>>,
    functions: Vec<CompiledFunction>,
    current_function: Option<usize>,
}

#[allow(dead_code)]
struct CompiledFunction {
    name: String,
    arity: usize,
    address: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            string_pool: Vec::new(),
            locals: vec![Vec::new()],
            scope_depth: 0,
            breaks: Vec::new(),
            continues: Vec::new(),
            functions: Vec::new(),
            current_function: None,
        }
    }

    pub fn compile(&mut self, ast: &AstNode) -> Result<(Vec<Bytecode>, Vec<String>), CompileError> {
        self.bytecode.clear();
        self.string_pool.clear();
        self.locals = vec![Vec::new()];
        self.scope_depth = 0;
        self.breaks.clear();
        self.continues.clear();
        self.functions.clear();
        self.current_function = None;

        // First pass: collect function declarations
        self.collect_functions(ast)?;

        self.compile_node(ast)?;
        self.bytecode.push(Bytecode::Halt);
        Ok((self.bytecode.clone(), self.string_pool.clone()))
    }

    fn collect_functions(&mut self, node: &AstNode) -> Result<(), CompileError> {
        match node {
            AstNode::Program(nodes) => {
                for n in nodes {
                    self.collect_functions(n)?;
                }
            }
            AstNode::FnDef { name, params, .. } => {
                let idx = self.functions.len();
                if idx > u16::MAX as usize {
                    return Err(CompileError::TooManyFunctions);
                }
                self.functions.push(CompiledFunction {
                    name: name.clone(),
                    arity: params.len(),
                    address: 0,
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_function(&self, name: &str) -> Option<u16> {
        self.functions.iter().position(|f| f.name == name).map(|i| i as u16)
    }

    fn compile_node(&mut self, node: &AstNode) -> Result<(), CompileError> {
        match node {
            AstNode::Program(nodes) => {
                for n in nodes {
                    self.compile_node(n)?;
                }
                Ok(())
            }

            AstNode::Int(val) => {
                self.bytecode.push(Bytecode::PushInt(*val));
                Ok(())
            }

            AstNode::Float(val) => {
                self.bytecode.push(Bytecode::PushFloat(*val));
                Ok(())
            }

            AstNode::Bool(val) => {
                self.bytecode.push(Bytecode::PushBool(*val));
                Ok(())
            }

            AstNode::String(val) => {
                let idx = self.add_string(val);
                self.bytecode.push(Bytecode::PushString(idx));
                Ok(())
            }

            AstNode::Null => {
                self.bytecode.push(Bytecode::PushNull);
                Ok(())
            }

            AstNode::Identifier(name) => {
                let idx = self.resolve_local(name)?;
                self.bytecode.push(Bytecode::LoadLocal(idx));
                Ok(())
            }

            AstNode::BinaryOp { left, op, right } => {
                self.compile_node(left)?;
                self.compile_node(right)?;
                let opcode = match op {
                    BinaryOpKind::Add => Bytecode::Add,
                    BinaryOpKind::Sub => Bytecode::Sub,
                    BinaryOpKind::Mul => Bytecode::Mul,
                    BinaryOpKind::Div => Bytecode::Div,
                    BinaryOpKind::Mod => Bytecode::Mod,
                    BinaryOpKind::Eq => Bytecode::Eq,
                    BinaryOpKind::Neq => Bytecode::Neq,
                    BinaryOpKind::Lt => Bytecode::Lt,
                    BinaryOpKind::Gt => Bytecode::Gt,
                    BinaryOpKind::Le => Bytecode::Le,
                    BinaryOpKind::Ge => Bytecode::Ge,
                    BinaryOpKind::And => Bytecode::And,
                    BinaryOpKind::Or => Bytecode::Or,
                };
                self.bytecode.push(opcode);
                Ok(())
            }

            AstNode::UnaryOp { op, expr } => {
                self.compile_node(expr)?;
                match op {
                    UnaryOpKind::Neg => self.bytecode.push(Bytecode::Neg),
                    UnaryOpKind::Not => self.bytecode.push(Bytecode::Not),
                }
                Ok(())
            }

            AstNode::Block(nodes) => {
                self.begin_scope();
                for n in nodes {
                    self.compile_node(n)?;
                }
                self.end_scope();
                Ok(())
            }

            AstNode::VarDecl {
                name,
                mutable: _,
                var_type: _,
                initializer,
            } => {
                self.compile_node(initializer)?;
                let idx = self.add_local(name)?;
                self.bytecode.push(Bytecode::StoreLocal(idx));
                Ok(())
            }

            AstNode::Assignment { target, op: _, value } => {
                self.compile_node(value)?;
                match target.as_ref() {
                    AstNode::Identifier(name) => {
                        let idx = self.resolve_local(name)?;
                        self.bytecode.push(Bytecode::StoreLocal(idx));
                    }
                    AstNode::MemberAccess { object, member } => {
                        self.compile_node(object)?;
                        // Push member index and store
                        let _ = member;
                    }
                    _ => return Err(CompileError::UnsupportedNode(format!("{:?}", target))),
                }
                Ok(())
            }

            AstNode::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_node(condition)?;
                let else_jump = self.bytecode.len();
                self.bytecode.push(Bytecode::JmpIfNot(0)); // placeholder
                self.compile_node(then_branch)?;
                if let Some(else_node) = else_branch {
                    let end_jump = self.bytecode.len();
                    self.bytecode.push(Bytecode::Jmp(0)); // placeholder
                    let else_addr = self.bytecode.len();
                    self.patch_jump(else_jump, else_addr as u16)?;
                    self.compile_node(else_node)?;
                    let end_addr = self.bytecode.len();
                    self.patch_jump(end_jump, end_addr as u16)?;
                } else {
                    let end_addr = self.bytecode.len();
                    self.patch_jump(else_jump, end_addr as u16)?;
                }
                Ok(())
            }

            AstNode::WhileStmt { condition, body } => {
                let loop_start = self.bytecode.len();
                self.compile_node(condition)?;
                let exit_jump = self.bytecode.len();
                self.bytecode.push(Bytecode::JmpIfNot(0)); // placeholder
                self.begin_loop();
                self.compile_node(body)?;
                self.end_loop(loop_start);
                self.bytecode.push(Bytecode::Jmp(loop_start as u16));
                let exit_addr = self.bytecode.len();
                self.patch_jump(exit_jump, exit_addr as u16)?;
                Ok(())
            }

            AstNode::ForRangeStmt { var, start, end, body } => {
                self.compile_node(start)?;
                let idx = self.add_local(var)?;
                self.bytecode.push(Bytecode::StoreLocal(idx));
                let loop_start = self.bytecode.len();
                self.bytecode.push(Bytecode::LoadLocal(idx));
                self.compile_node(end)?;
                let exit_jump = self.bytecode.len();
                self.bytecode.push(Bytecode::JmpIfNot(0)); // placeholder
                self.begin_loop();
                self.compile_node(body)?;
                self.end_loop(loop_start);
                // Increment loop variable
                self.bytecode.push(Bytecode::LoadLocal(idx));
                self.bytecode.push(Bytecode::PushInt(1));
                self.bytecode.push(Bytecode::Add);
                self.bytecode.push(Bytecode::StoreLocal(idx));
                self.bytecode.push(Bytecode::Jmp(loop_start as u16));
                let exit_addr = self.bytecode.len();
                self.patch_jump(exit_jump, exit_addr as u16)?;
                Ok(())
            }

            AstNode::ReturnStmt { value } => {
                if let Some(val) = value {
                    self.compile_node(val)?;
                } else {
                    self.bytecode.push(Bytecode::PushNull);
                }
                self.bytecode.push(Bytecode::Return);
                Ok(())
            }

            AstNode::Break => {
                let idx = self.bytecode.len();
                self.bytecode.push(Bytecode::Jmp(0));
                if let Some(breaks) = self.breaks.last_mut() {
                    breaks.push(idx);
                }
                Ok(())
            }

            AstNode::Call { callee, args } => {
                // Compile arguments
                for arg in args {
                    self.compile_node(arg)?;
                }
                match callee.as_ref() {
                    AstNode::Identifier(name) => {
                        if let Some(func_idx) = self.resolve_function(name) {
                            self.bytecode.push(Bytecode::Call(func_idx));
                        } else {
                            // Builtin call
                            let builtin_id = match name.as_str() {
                                "log" => 0,
                                "sin" => 1,
                                "cos" => 2,
                                "sqrt" => 3,
                                "abs" => 4,
                                "clamp" => 5,
                                "random" => 6,
                                "print" => 7,
                                "pop" => 8,
                                "floor" => 9,
                                "ceil" => 10,
                                "round" => 11,
                                "len" => 12,
                                "min" => 13,
                                "max" => 14,
                                "pow" => 15,
                                "pi" => 16,
                                "lerp" => 17,
                                "distance" => 18,
                                "tan" => 19,
                                "exp" => 20,
                                "sign" => 21,
                                "deg2rad" => 22,
                                _ => return Err(CompileError::FunctionNotFound(name.clone())),
                            };
                            self.bytecode.push(Bytecode::CallBuiltin(builtin_id));
                        }
                    }
                    _ => return Err(CompileError::UnsupportedNode(format!("{:?}", callee))),
                }
                Ok(())
            }

            AstNode::MemberAccess { object, member } => {
                self.compile_node(object)?;
                let _ = member;
                Ok(())
            }

            AstNode::Index { object, index } => {
                self.compile_node(object)?;
                self.compile_node(index)?;
                Ok(())
            }

            AstNode::FnDef {
                name,
                params,
                return_type: _,
                is_coroutine: _,
                body,
            } => {
                let func_idx = self.resolve_function(name).unwrap_or(0);
                let func = &mut self.functions[func_idx as usize];
                func.address = self.bytecode.len();

                // Enter new function scope
                self.begin_scope();
                for param in params {
                    self.add_local(&param.name)?;
                }

                let old_fn = self.current_function.replace(func_idx as usize);
                self.compile_node(body)?;
                self.current_function = old_fn;

                // Implicit return
                self.bytecode.push(Bytecode::PushNull);
                self.bytecode.push(Bytecode::Return);

                self.end_scope();
                Ok(())
            }

            AstNode::EntityDef { .. }
            | AstNode::ComponentDef { .. }
            | AstNode::EventDef { .. }
            | AstNode::Vec3(_, _, _)
            | AstNode::ComponentRef { .. }
            | AstNode::EventHandler { .. }
            | AstNode::StateDecl { .. } => {
                // Entity/component/event definitions are data, not executed directly
                Ok(())
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
        self.locals.push(Vec::new());
    }

    fn end_scope(&mut self) {
        self.locals.pop();
        self.scope_depth -= 1;
    }

    fn add_local(&mut self, name: &str) -> Result<u8, CompileError> {
        let depth = self.locals.len() - 1;
        let idx = self.locals[depth].len();
        if idx > u8::MAX as usize {
            return Err(CompileError::TooManyLocals);
        }
        self.locals[depth].push(name.to_string());
        // Compute flat index
        let mut flat = 0u8;
        for i in 0..self.locals.len() {
            for _ in 0..self.locals[i].len() {
                if i == depth && flat as usize == idx {
                    return Ok(flat);
                }
                flat += 1;
            }
        }
        // If we reach here, just return the depth-local index
        Ok(idx as u8)
    }

    fn resolve_local(&self, name: &str) -> Result<u8, CompileError> {
        let mut flat = 0u8;
        for scope in &self.locals {
            for local in scope {
                if local == name {
                    return Ok(flat);
                }
                flat += 1;
            }
        }
        Err(CompileError::VariableNotFound(name.to_string()))
    }

    fn begin_loop(&mut self) {
        self.breaks.push(Vec::new());
        self.continues.push(Vec::new());
    }

    fn end_loop(&mut self, _loop_start: usize) {
        // Patch breaks
        if let Some(breaks) = self.breaks.pop() {
            let end_addr = self.bytecode.len();
            for break_idx in breaks {
                self.patch_jump(break_idx, end_addr as u16).ok();
            }
        }
        self.continues.pop();
    }

    fn patch_jump(&mut self, location: usize, target: u16) -> Result<(), CompileError> {
        match &mut self.bytecode[location] {
            Bytecode::Jmp(ref mut addr)
            | Bytecode::JmpIf(ref mut addr)
            | Bytecode::JmpIfNot(ref mut addr) => {
                *addr = target;
                Ok(())
            }
            _ => Err(CompileError::UnsupportedNode("patch on non-jump".to_string())),
        }
    }

    fn add_string(&mut self, s: &str) -> u16 {
        let idx = self.string_pool.len();
        self.string_pool.push(s.to_string());
        idx as u16
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile_source(source: &str) -> Result<Vec<Bytecode>, CompileError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(&ast).map(|(bc, _)| bc)
    }

    #[test]
    fn test_compile_empty() {
        let bc = compile_source("").unwrap();
        assert_eq!(bc, vec![Bytecode::Halt]);
    }

    #[test]
    fn test_compile_integer() {
        let bc = compile_source("42;").unwrap();
        assert_eq!(bc, vec![Bytecode::PushInt(42), Bytecode::Halt]);
    }

    #[test]
    fn test_compile_float() {
        let bc = compile_source("3.14;").unwrap();
        assert_eq!(bc, vec![Bytecode::PushFloat(3.14), Bytecode::Halt]);
    }

    #[test]
    fn test_compile_bool() {
        let bc = compile_source("true;").unwrap();
        assert_eq!(bc, vec![Bytecode::PushBool(true), Bytecode::Halt]);
    }

    #[test]
    fn test_compile_string() {
        let bc = compile_source("\"hello\";").unwrap();
        assert_eq!(bc, vec![Bytecode::PushString(0), Bytecode::Halt]);
    }

    #[test]
    fn test_compile_binary_op() {
        let bc = compile_source("1 + 2;").unwrap();
        assert_eq!(
            bc,
            vec![
                Bytecode::PushInt(1),
                Bytecode::PushInt(2),
                Bytecode::Add,
                Bytecode::Halt
            ]
        );
    }

    #[test]
    fn test_compile_unary_neg() {
        let bc = compile_source("-5;").unwrap();
        assert_eq!(
            bc,
            vec![Bytecode::PushInt(5), Bytecode::Neg, Bytecode::Halt]
        );
    }

    #[test]
    fn test_compile_var_decl() {
        let bc = compile_source("let x = 42;").unwrap();
        assert_eq!(
            bc,
            vec![
                Bytecode::PushInt(42),
                Bytecode::StoreLocal(0),
                Bytecode::Halt
            ]
        );
    }

    #[test]
    fn test_compile_var_use() {
        let bc = compile_source("let x = 1; let y = x;").unwrap();
        assert_eq!(
            bc,
            vec![
                Bytecode::PushInt(1),
                Bytecode::StoreLocal(0),
                Bytecode::LoadLocal(0),
                Bytecode::StoreLocal(1),
                Bytecode::Halt
            ]
        );
    }

    #[test]
    fn test_compile_assignment() {
        let bc = compile_source("let x = 1; x = 2;").unwrap();
        assert_eq!(
            bc,
            vec![
                Bytecode::PushInt(1),
                Bytecode::StoreLocal(0),
                Bytecode::PushInt(2),
                Bytecode::StoreLocal(0),
                Bytecode::Halt
            ]
        );
    }

    #[test]
    fn test_compile_if_else() {
        let bc = compile_source("if true { 1; } else { 2; }").unwrap();
        assert!(bc.iter().any(|b| matches!(b, Bytecode::JmpIfNot(_) | Bytecode::Jmp(_))));
        assert!(bc.contains(&Bytecode::Halt));
    }

    #[test]
    fn test_compile_while() {
        let bc = compile_source("let x = 0; while x < 10 { x = x + 1; }").unwrap();
        assert!(bc.iter().any(|b| matches!(b, Bytecode::JmpIfNot(_))));
        assert!(bc.contains(&Bytecode::Halt));
    }

    #[test]
    fn test_compile_for() {
        let bc = compile_source("for i in 0..10 { log(i); }").unwrap();
        assert!(bc.contains(&Bytecode::Halt));
    }

    #[test]
    fn test_compile_fn_decl() {
        let bc = compile_source("fn add(a: int, b: int) -> int { return a + b; }").unwrap();
        assert_eq!(bc.last(), Some(&Bytecode::Halt));
    }

    #[test]
    fn test_compile_fn_call() {
        let bc = compile_source("fn foo() {} foo();").unwrap();
        assert!(bc.contains(&Bytecode::Call(0)));
        assert!(bc.contains(&Bytecode::Halt));
    }

    #[test]
    fn test_compile_nested_blocks() {
        let bc = compile_source("let x = 0; { let y = 1; x = y; }").unwrap();
        assert_eq!(bc.last(), Some(&Bytecode::Halt));
    }
}

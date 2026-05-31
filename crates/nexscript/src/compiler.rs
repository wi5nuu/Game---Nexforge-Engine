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
    #[error("Unsupported AST node")]
    UnsupportedNode,
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    #[error("Too many constants")]
    TooManyConstants,
}

pub struct Compiler {
    bytecode: Vec<Bytecode>,
    constants: Vec<Value>,
    string_pool: Vec<String>,
    locals: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(u16),
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: Vec::new(),
            string_pool: Vec::new(),
            locals: Vec::new(),
        }
    }

    pub fn compile(&mut self, ast: &AstNode) -> Result<Vec<Bytecode>, CompileError> {
        self.bytecode.clear();
        self.compile_node(ast)?;
        self.bytecode.push(Bytecode::Halt);
        Ok(self.bytecode.clone())
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
            AstNode::Identifier(_) => {
                // Placeholder
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
                    BinaryOpKind::Mod => Bytecode::Add, // placeholder
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
                for n in nodes {
                    self.compile_node(n)?;
                }
                Ok(())
            }
            AstNode::VarDecl { name, .. } => {
                self.locals.push(name.clone());
                Ok(())
            }
            AstNode::FnDef { .. }
            | AstNode::EntityDef { .. }
            | AstNode::ComponentDef { .. }
            | AstNode::EventDef { .. }
            | AstNode::IfStmt { .. }
            | AstNode::WhileStmt { .. }
            | AstNode::ForRangeStmt { .. }
            | AstNode::ReturnStmt { .. }
            | AstNode::Break
            | AstNode::Call { .. }
            | AstNode::MemberAccess { .. }
            | AstNode::Index { .. }
            | AstNode::Assignment { .. }
            | AstNode::Vec3(_, _, _)
            | AstNode::ComponentRef { .. }
            | AstNode::EventHandler { .. }
            | AstNode::StateDecl { .. } => {
                // Placeholder — will be fully implemented in later phases
                Ok(())
            }
        }
    }

    fn add_string(&mut self, s: &str) -> u16 {
        let idx = self.string_pool.len();
        self.string_pool.push(s.to_string());
        idx as u16
    }

    pub fn into_bytecode(self) -> Vec<Bytecode> {
        self.bytecode
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
        compiler.compile(&ast)
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
    fn test_compile_binary_op() {
        let bc = compile_source("1 + 2;").unwrap();
        assert_eq!(
            bc,
            vec![Bytecode::PushInt(1), Bytecode::PushInt(2), Bytecode::Add, Bytecode::Halt]
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
}

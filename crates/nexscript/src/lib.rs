#![deny(clippy::all)]

pub mod ast;
pub mod compiler;
pub mod hot_reload;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod vm;

pub use ast::AstNode;
pub use compiler::Bytecode;
pub use compiler::CompileError;
pub use compiler::Compiler;
pub use lexer::Lexer;
pub use parser::Parser;
pub use runtime::EntityId;
pub use runtime::InputState;
pub use runtime::ScriptContext;
pub use runtime::ScriptEvent;
pub use runtime::ScriptRuntime;
pub use vm::Value;
pub use vm::Vm;
pub use vm::VmError;

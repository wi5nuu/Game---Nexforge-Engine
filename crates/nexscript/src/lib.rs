#![deny(clippy::all)]

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod compiler;
pub mod vm;
pub mod runtime;
pub mod hot_reload;

pub use lexer::Lexer;
pub use parser::Parser;
pub use ast::AstNode;
pub use compiler::Compiler;
pub use vm::Vm;
pub use vm::Value;
pub use compiler::Bytecode;
pub use compiler::CompileError;
pub use vm::VmError;
pub use runtime::ScriptRuntime;
pub use runtime::InputState;
pub use runtime::ScriptEvent;
pub use runtime::ScriptContext;
pub use runtime::EntityId;

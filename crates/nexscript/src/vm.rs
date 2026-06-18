#![deny(clippy::all)]

use crate::compiler::Bytecode;
use thiserror::Error;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Vec3(f64, f64, f64),
    Null,
}

#[derive(Debug, Error)]
pub enum VmError {
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Unknown opcode at {0}")]
    UnknownOpcode(usize),
    #[error("Type mismatch: cannot {op} {lhs:?} with {rhs:?}")]
    TypeError { op: String, lhs: Value, rhs: Value },
    #[error("Unary type error: cannot {op} {val:?}")]
    UnaryTypeError { op: String, val: Value },
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Halt signal received")]
    Halt,
    #[error("Function index {0} out of bounds")]
    FunctionNotFound(u16),
    #[error("Call stack overflow (max 256 frames)")]
    CallStackOverflow,
    #[error("Variable {0} not found")]
    VariableNotFound(String),
    #[error("Arity mismatch: expected {expected}, got {got}")]
    ArityMismatch { expected: usize, got: usize },
    #[error("Coroutine not found")]
    CoroutineNotFound,
}

const MAX_CALL_STACK: usize = 256;

#[derive(Debug, Clone)]
struct StackFrame {
    return_ip: usize,
    locals: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct Coroutine {
    ip: usize,
    stack: Vec<Value>,
    frames: Vec<StackFrame>,
    completed: bool,
}

#[allow(dead_code)]
pub struct Vm {
    bytecode: Vec<Bytecode>,
    string_pool: Vec<String>,
    ip: usize,
    value_stack: Vec<Value>,
    call_stack: Vec<StackFrame>,
    global_locals: HashMap<String, Value>,
    should_halt: bool,
    coroutines: Vec<Coroutine>,
    running_coroutine: Option<usize>,
    function_addresses: Vec<usize>,
}

impl Vm {
    pub fn new(bytecode: Vec<Bytecode>, string_pool: Vec<String>) -> Self {
        let func_addrs = Vec::new();
        Self {
            bytecode,
            string_pool,
            ip: 0,
            value_stack: Vec::new(),
            call_stack: Vec::new(),
            global_locals: HashMap::new(),
            should_halt: false,
            coroutines: Vec::new(),
            running_coroutine: None,
            function_addresses: func_addrs,
        }
    }

    pub fn run(&mut self) -> Result<Option<Value>, VmError> {
        while !self.should_halt && self.ip < self.bytecode.len() {
            self.step()?;
        }
        Ok(self.value_stack.pop())
    }

    pub fn step(&mut self) -> Result<(), VmError> {
        if self.should_halt || self.ip >= self.bytecode.len() {
            return Err(VmError::Halt);
        }
        let op = self.bytecode[self.ip].clone();
        self.ip += 1;
        self.execute(op)
    }

    pub fn run_frame(&mut self) -> Result<(), VmError> {
        while !self.should_halt && self.ip < self.bytecode.len() {
            let op = self.bytecode[self.ip].clone();
            match op {
                Bytecode::Halt => {
                    self.should_halt = true;
                    return Ok(());
                }
                Bytecode::Return => {
                    let val = self.value_stack.pop().unwrap_or(Value::Null);
                    self.do_return(val)?;
                    return Ok(());
                }
                Bytecode::Yield => {
                    self.ip += 1;
                    return Ok(());
                }
                _ => {
                    self.ip += 1;
                    self.execute(op)?;
                }
            }
        }
        Ok(())
    }

    fn execute(&mut self, op: Bytecode) -> Result<(), VmError> {
        match op {
            Bytecode::Nop => {}
            Bytecode::PushInt(val) => self.value_stack.push(Value::Int(val)),
            Bytecode::PushFloat(val) => self.value_stack.push(Value::Float(val)),
            Bytecode::PushBool(val) => self.value_stack.push(Value::Bool(val)),
            Bytecode::PushString(idx) => {
                let s = self.string_pool.get(idx as usize)
                    .cloned()
                    .unwrap_or_default();
                self.value_stack.push(Value::String(s));
            }
            Bytecode::PushVec3(x, y, z) => self.value_stack.push(Value::Vec3(x, y, z)),
            Bytecode::PushNull => self.value_stack.push(Value::Null),
            Bytecode::Pop => {
                self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
            }
            Bytecode::Dup => {
                let val = self.value_stack.last()
                    .ok_or(VmError::StackUnderflow)?
                    .clone();
                self.value_stack.push(val);
            }
            Bytecode::LoadLocal(idx) => {
                let val = self.resolve_local(idx as usize)?;
                self.value_stack.push(val);
            }
            Bytecode::StoreLocal(idx) => {
                let val = self.value_stack.pop()
                    .ok_or(VmError::StackUnderflow)?;
                self.store_local(idx as usize, val)?;
            }
            Bytecode::Add => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(binary_op(BinaryOpKind::Add, lhs, rhs)?);
            }
            Bytecode::Sub => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(binary_op(BinaryOpKind::Sub, lhs, rhs)?);
            }
            Bytecode::Mul => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(binary_op(BinaryOpKind::Mul, lhs, rhs)?);
            }
            Bytecode::Div => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(binary_op(BinaryOpKind::Div, lhs, rhs)?);
            }
            Bytecode::Mod => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(binary_op(BinaryOpKind::Mod, lhs, rhs)?);
            }
            Bytecode::Neg => {
                let val = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(unary_op(UnaryOpKind::Neg, val)?);
            }
            Bytecode::Eq => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(Value::Bool(values_equal(&lhs, &rhs)));
            }
            Bytecode::Neq => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(Value::Bool(!values_equal(&lhs, &rhs)));
            }
            Bytecode::Lt => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(compare_op(CompareOp::Lt, lhs, rhs)?);
            }
            Bytecode::Gt => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(compare_op(CompareOp::Gt, lhs, rhs)?);
            }
            Bytecode::Le => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(compare_op(CompareOp::Le, lhs, rhs)?);
            }
            Bytecode::Ge => {
                let rhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(compare_op(CompareOp::Ge, lhs, rhs)?);
            }
            Bytecode::Not => {
                let val = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                self.value_stack.push(unary_op(UnaryOpKind::Not, val)?);
            }
            Bytecode::Jmp(addr) => {
                self.ip = addr as usize;
            }
            Bytecode::JmpIf(addr) => {
                let val = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                if is_truthy(&val) {
                    self.ip = addr as usize;
                }
            }
            Bytecode::JmpIfNot(addr) => {
                let val = self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
                if !is_truthy(&val) {
                    self.ip = addr as usize;
                }
            }
            Bytecode::Call(func_idx) => {
                self.do_call(func_idx)?;
            }
            Bytecode::CallBuiltin(id) => {
                self.call_builtin(id)?;
            }
            Bytecode::Return => {
                let val = self.value_stack.pop().unwrap_or(Value::Null);
                self.do_return(val)?;
            }
            Bytecode::Yield => {
                // Save coroutine state and return to caller
                // Currently a no-op in single-frame mode
            }
            Bytecode::Halt => {
                self.should_halt = true;
            }
            _ => {
                return Err(VmError::UnknownOpcode(self.ip - 1));
            }
        }
        Ok(())
    }

    fn resolve_local(&self, idx: usize) -> Result<Value, VmError> {
        if let Some(frame) = self.call_stack.last() {
            if idx < frame.locals.len() {
                return Ok(frame.locals[idx].clone());
            }
        }
        // Check global scope
        if let Some(val) = self.global_locals.get(&format!("_{}", idx)) {
            return Ok(val.clone());
        }
        Ok(Value::Null)
    }

    fn store_local(&mut self, idx: usize, val: Value) -> Result<(), VmError> {
        if let Some(frame) = self.call_stack.last_mut() {
            if idx < frame.locals.len() {
                frame.locals[idx] = val;
                return Ok(());
            }
            // Extend locals
            frame.locals.resize(idx + 1, Value::Null);
            frame.locals[idx] = val;
            return Ok(());
        }
        // Global scope fallback
        self.global_locals.insert(format!("_{}", idx), val);
        Ok(())
    }

    fn do_call(&mut self, func_idx: u16) -> Result<(), VmError> {
        let func_addrs = self.collect_function_addresses();
        if let Some(&addr) = func_addrs.get(func_idx as usize) {
            if self.call_stack.len() >= MAX_CALL_STACK {
                return Err(VmError::CallStackOverflow);
            }
            let frame = StackFrame {
                return_ip: self.ip,
                locals: Vec::new(),
            };
            self.call_stack.push(frame);
            self.ip = addr;
            Ok(())
        } else {
            Err(VmError::FunctionNotFound(func_idx))
        }
    }

    fn collect_function_addresses(&self) -> Vec<usize> {
        let mut addrs = Vec::new();
        let mut i = 0;
        while i < self.bytecode.len() {
            match &self.bytecode[i] {
                Bytecode::Call(idx) => {
                    let idx = *idx as usize;
                    if idx >= addrs.len() {
                        addrs.resize(idx + 1, 0);
                    }
                    addrs[idx] = i + 1;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
        for i in 0..self.bytecode.len() {
            if self.bytecode[i] == Bytecode::Halt {
                let mut func_idx = 0usize;
                let mut j = i + 1;
                while j < self.bytecode.len() {
                    if func_idx >= addrs.len() {
                        addrs.push(j);
                    } else {
                        addrs[func_idx] = j;
                    }
                    func_idx += 1;
                    while j < self.bytecode.len() {
                        if j + 1 < self.bytecode.len()
                            && self.bytecode[j] == Bytecode::PushNull
                            && self.bytecode[j + 1] == Bytecode::Return
                        {
                            j += 2;
                            break;
                        }
                        j += 1;
                    }
                }
                break;
            }
        }
        addrs
    }

    fn do_return(&mut self, val: Value) -> Result<(), VmError> {
        if let Some(frame) = self.call_stack.pop() {
            self.ip = frame.return_ip;
            self.value_stack.push(val);
        }
        Ok(())
    }

    fn call_builtin(&mut self, id: u8) -> Result<(), VmError> {
        match id {
            0 => { // log
                let msg = self.value_stack.pop().unwrap_or(Value::String(String::new()));
                if let Value::String(s) = msg {
                    println!("[NexScript] {}", s);
                }
                self.value_stack.push(Value::Null);
            }
            1 => { // sin
                let val = self.value_stack.pop().unwrap_or(Value::Float(0.0));
                let f = match val {
                    Value::Float(v) => v,
                    Value::Int(v) => v as f64,
                    _ => 0.0,
                };
                self.value_stack.push(Value::Float(f.sin()));
            }
            2 => { // cos
                let val = self.value_stack.pop().unwrap_or(Value::Float(0.0));
                let f = match val {
                    Value::Float(v) => v,
                    Value::Int(v) => v as f64,
                    _ => 0.0,
                };
                self.value_stack.push(Value::Float(f.cos()));
            }
            3 => { // sqrt
                let val = self.value_stack.pop().unwrap_or(Value::Float(0.0));
                let f = match val {
                    Value::Float(v) => v,
                    Value::Int(v) => v as f64,
                    _ => 0.0,
                };
                self.value_stack.push(Value::Float(f.sqrt()));
            }
            4 => { // abs
                let val = self.value_stack.pop().unwrap_or(Value::Int(0));
                match val {
                    Value::Int(v) => self.value_stack.push(Value::Int(v.abs())),
                    Value::Float(v) => self.value_stack.push(Value::Float(v.abs())),
                    _ => self.value_stack.push(Value::Null),
                }
            }
            5 => { // clamp
                let max = self.value_stack.pop().unwrap_or(Value::Float(1.0));
                let min = self.value_stack.pop().unwrap_or(Value::Float(0.0));
                let val = self.value_stack.pop().unwrap_or(Value::Float(0.0));
                let (vf, minf, maxf) = match (val, min, max) {
                    (Value::Float(v), Value::Float(mn), Value::Float(mx)) => (v, mn, mx),
                    (Value::Int(v), Value::Int(mn), Value::Int(mx)) => {
                        self.value_stack.push(Value::Int(v.clamp(mn, mx)));
                        return Ok(());
                    }
                    _ => (0.0, 0.0, 1.0),
                };
                self.value_stack.push(Value::Float(vf.clamp(minf, maxf)));
            }
            6 => { // random
                self.value_stack.push(Value::Float(rand::random::<f64>()));
            }
            7 => { // print
                let val = self.value_stack.pop().unwrap_or(Value::Null);
                print!("{}", value_display(&val));
                self.value_stack.push(Value::Null);
            }
            8 => { // pop — discard top of stack
                self.value_stack.pop().ok_or(VmError::StackUnderflow)?;
            }
            _ => {}
        }
        Ok(())
    }

    // Coroutine support
    pub fn create_coroutine(&mut self, ip: usize) -> usize {
        let idx = self.coroutines.len();
        self.coroutines.push(Coroutine {
            ip,
            stack: Vec::new(),
            frames: Vec::new(),
            completed: false,
        });
        idx
    }

    pub fn resume_coroutine(&mut self, idx: usize) -> Result<Option<Value>, VmError> {
        let (coro_ip, coro_stack, coro_frames) = {
            let coro = self.coroutines.get_mut(idx)
                .ok_or(VmError::CoroutineNotFound)?;
            if coro.completed {
                return Ok(Some(Value::Null));
            }
            (coro.ip, std::mem::take(&mut coro.stack), std::mem::take(&mut coro.frames))
        };

        let saved_ip = self.ip;
        let saved_stack = std::mem::take(&mut self.value_stack);
        let saved_frames = std::mem::take(&mut self.call_stack);

        self.ip = coro_ip;
        self.value_stack = coro_stack;
        self.call_stack = coro_frames;

        let result = self.run_frame();

        if let Some(coro) = self.coroutines.get_mut(idx) {
            coro.ip = self.ip;
            coro.stack = std::mem::take(&mut self.value_stack);
            coro.frames = std::mem::take(&mut self.call_stack);
            if self.should_halt {
                coro.completed = true;
            }
        }

        self.ip = saved_ip;
        self.value_stack = saved_stack;
        self.call_stack = saved_frames;

        result?;
        Ok(Some(Value::Null))
    }

    // Hot-reload support
    pub fn hot_reload(&mut self, new_bytecode: Vec<Bytecode>, new_string_pool: Vec<String>) {
        self.bytecode = new_bytecode;
        self.string_pool = new_string_pool;
        self.ip = 0;
        self.value_stack.clear();
        self.call_stack.clear();
        self.should_halt = false;
    }

    pub fn get_ip(&self) -> usize {
        self.ip
    }

    pub fn stack_depth(&self) -> usize {
        self.value_stack.len()
    }

    pub fn is_running(&self) -> bool {
        !self.should_halt
    }
}

// Helper enums for internal dispatch
enum BinaryOpKind { Add, Sub, Mul, Div, Mod }
enum UnaryOpKind { Neg, Not }
enum CompareOp { Lt, Gt, Le, Ge }

fn binary_op(kind: BinaryOpKind, a: Value, b: Value) -> Result<Value, VmError> {
    let op_name = match kind {
        BinaryOpKind::Add => "+",
        BinaryOpKind::Sub => "-",
        BinaryOpKind::Mul => "*",
        BinaryOpKind::Div => "/",
        BinaryOpKind::Mod => "%",
    };
    match (a, b) {
        (Value::Int(l), Value::Int(r)) => match kind {
            BinaryOpKind::Add => Ok(Value::Int(l + r)),
            BinaryOpKind::Sub => Ok(Value::Int(l - r)),
            BinaryOpKind::Mul => Ok(Value::Int(l * r)),
            BinaryOpKind::Div => {
                if r == 0 { Err(VmError::DivisionByZero) }
                else { Ok(Value::Int(l / r)) }
            }
            BinaryOpKind::Mod => {
                if r == 0 { Err(VmError::DivisionByZero) }
                else { Ok(Value::Int(l % r)) }
            }
        },
        (Value::Float(l), Value::Float(r)) => match kind {
            BinaryOpKind::Add => Ok(Value::Float(l + r)),
            BinaryOpKind::Sub => Ok(Value::Float(l - r)),
            BinaryOpKind::Mul => Ok(Value::Float(l * r)),
            BinaryOpKind::Div => {
                if r == 0.0 { Err(VmError::DivisionByZero) }
                else { Ok(Value::Float(l / r)) }
            }
            BinaryOpKind::Mod => {
                if r == 0.0 { Err(VmError::DivisionByZero) }
                else { Ok(Value::Float(l % r)) }
            }
        },
        (Value::Int(l), Value::Float(r)) => match kind {
            BinaryOpKind::Add => Ok(Value::Float(l as f64 + r)),
            BinaryOpKind::Sub => Ok(Value::Float(l as f64 - r)),
            BinaryOpKind::Mul => Ok(Value::Float(l as f64 * r)),
            BinaryOpKind::Div => {
                if r == 0.0 { Err(VmError::DivisionByZero) }
                else { Ok(Value::Float(l as f64 / r)) }
            }
            BinaryOpKind::Mod => {
                if r == 0.0 { Err(VmError::DivisionByZero) }
                else { Ok(Value::Float((l as f64) % r)) }
            }
        },
        (Value::Float(l), Value::Int(r)) => match kind {
            BinaryOpKind::Add => Ok(Value::Float(l + r as f64)),
            BinaryOpKind::Sub => Ok(Value::Float(l - r as f64)),
            BinaryOpKind::Mul => Ok(Value::Float(l * r as f64)),
            BinaryOpKind::Div => {
                if r == 0 { Err(VmError::DivisionByZero) }
                else { Ok(Value::Float(l / r as f64)) }
            }
            BinaryOpKind::Mod => {
                if r == 0 { Err(VmError::DivisionByZero) }
                else { Ok(Value::Float(l % (r as f64))) }
            }
        },
        (Value::String(l), Value::String(r)) if matches!(kind, BinaryOpKind::Add) => {
            Ok(Value::String(l + &r))
        }
        (l, r) => Err(VmError::TypeError {
            op: op_name.to_string(),
            lhs: l,
            rhs: r,
        }),
    }
}

fn unary_op(kind: UnaryOpKind, a: Value) -> Result<Value, VmError> {
    match kind {
        UnaryOpKind::Neg => match a {
            Value::Int(v) => Ok(Value::Int(-v)),
            Value::Float(v) => Ok(Value::Float(-v)),
            v => Err(VmError::UnaryTypeError {
                op: "neg".to_string(),
                val: v,
            }),
        },
        UnaryOpKind::Not => match a {
            Value::Bool(v) => Ok(Value::Bool(!v)),
            v => Err(VmError::UnaryTypeError {
                op: "not".to_string(),
                val: v,
            }),
        },
    }
}

fn compare_op(kind: CompareOp, a: Value, b: Value) -> Result<Value, VmError> {
    let result = match (a, b) {
        (Value::Int(l), Value::Int(r)) => match kind {
            CompareOp::Lt => l < r,
            CompareOp::Gt => l > r,
            CompareOp::Le => l <= r,
            CompareOp::Ge => l >= r,
        },
        (Value::Float(l), Value::Float(r)) => match kind {
            CompareOp::Lt => l < r,
            CompareOp::Gt => l > r,
            CompareOp::Le => l <= r,
            CompareOp::Ge => l >= r,
        },
        (Value::Int(l), Value::Float(r)) => match kind {
            CompareOp::Lt => (l as f64) < r,
            CompareOp::Gt => (l as f64) > r,
            CompareOp::Le => (l as f64) <= r,
            CompareOp::Ge => (l as f64) >= r,
        },
        (Value::Float(l), Value::Int(r)) => match kind {
            CompareOp::Lt => l < (r as f64),
            CompareOp::Gt => l > (r as f64),
            CompareOp::Le => l <= (r as f64),
            CompareOp::Ge => l >= (r as f64),
        },
        _ => false,
    };
    Ok(Value::Bool(result))
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Null => false,
        Value::String(s) => !s.is_empty(),
        Value::Vec3(_, _, _) => true,
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(l), Value::Int(r)) => l == r,
        (Value::Int(l), Value::Float(r)) => (*l as f64 - r).abs() < f64::EPSILON,
        (Value::Float(l), Value::Int(r)) => (*l - *r as f64).abs() < f64::EPSILON,
        (Value::Float(l), Value::Float(r)) => (l - r).abs() < f64::EPSILON,
        (Value::Bool(l), Value::Bool(r)) => l == r,
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

fn value_display(val: &Value) -> String {
    match val {
        Value::Int(v) => v.to_string(),
        Value::Float(v) => v.to_string(),
        Value::Bool(v) => v.to_string(),
        Value::String(v) => v.clone(),
        Value::Vec3(x, y, z) => format!("({}, {}, {})", x, y, z),
        Value::Null => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn run_source(source: &str) -> Result<Option<Value>, VmError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut compiler = Compiler::new();
        let (bytecode, string_pool) = compiler.compile(&ast).unwrap();
        let mut vm = Vm::new(bytecode, string_pool);
        vm.run()
    }

    fn extract_int(val: Option<Value>) -> i32 {
        match val {
            Some(Value::Int(v)) => v,
            _ => panic!("expected Int, got {:?}", val),
        }
    }

    fn extract_float(val: Option<Value>) -> f64 {
        match val {
            Some(Value::Float(v)) => v,
            _ => panic!("expected Float, got {:?}", val),
        }
    }

    fn extract_bool(val: Option<Value>) -> bool {
        match val {
            Some(Value::Bool(v)) => v,
            _ => panic!("expected Bool, got {:?}", val),
        }
    }

    #[test]
    fn test_push_int() {
        let result = run_source("42;").unwrap();
        assert_eq!(extract_int(result), 42);
    }

    #[test]
    fn test_push_float() {
        let result = run_source("3.14;").unwrap();
        assert!((extract_float(result) - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_push_bool() {
        let result = run_source("true;").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_add_ints() {
        let result = run_source("10 + 20;").unwrap();
        assert_eq!(extract_int(result), 30);
    }

    #[test]
    fn test_sub_floats() {
        let result = run_source("10.0 - 3.0;").unwrap();
        assert!((extract_float(result) - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mul_mixed() {
        let result = run_source("5 * 2.5;").unwrap();
        assert!((extract_float(result) - 12.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_div_by_zero() {
        let result = run_source("10 / 0;");
        assert!(matches!(result, Err(VmError::DivisionByZero)));
    }

    #[test]
    fn test_negate() {
        let result = run_source("-7;").unwrap();
        assert_eq!(extract_int(result), -7);
    }

    #[test]
    fn test_not() {
        let result = run_source("!true;").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_equality() {
        let result = run_source("5 == 5;").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_inequality() {
        let result = run_source("5 != 3;").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_less_than() {
        let result = run_source("3 < 5;").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_complex_expression() {
        let result = run_source("(2 + 3) * 4;").unwrap();
        assert_eq!(extract_int(result), 20);
    }

    #[test]
    fn test_var_decl_and_use() {
        let result = run_source("let x = 42; x;").unwrap();
        assert_eq!(extract_int(result), 42);
    }

    #[test]
    fn test_multi_var() {
        let result = run_source("let x = 1; let y = 2; x + y;").unwrap();
        assert_eq!(extract_int(result), 3);
    }

    #[test]
    fn test_reassign() {
        let result = run_source("let x = 1; x = 2; x;").unwrap();
        assert_eq!(extract_int(result), 2);
    }

    #[test]
    fn test_if_true() {
        let result = run_source("if true { 1; } else { 2; }").unwrap();
        assert_eq!(extract_int(result), 1);
    }

    #[test]
    fn test_if_false() {
        let result = run_source("if false { 1; } else { 2; }").unwrap();
        assert_eq!(extract_int(result), 2);
    }

    #[test]
    fn test_while_loop() {
        let result = run_source("let x = 0; while x < 5 { x = x + 1; } x;").unwrap();
        assert_eq!(extract_int(result), 5);
    }

    #[test]
    fn test_nested_if() {
        let result = run_source("let x = 5; if x > 3 { if x < 10 { 1; } else { 2; } } else { 3; }").unwrap();
        assert_eq!(extract_int(result), 1);
    }

    #[test]
    fn test_stack_underflow() {
        // Direct bytecode: push one value, pop it, pop again on empty stack
        let mut vm = Vm::new(
            vec![Bytecode::PushInt(42), Bytecode::Pop, Bytecode::Pop, Bytecode::Halt],
            vec![],
        );
        let result = vm.run();
        assert!(matches!(result, Err(VmError::StackUnderflow)));
    }

    #[test]
    fn test_string_concat() {
        let result = run_source("\"hello\" + \" world\";").unwrap();
        match result {
            Some(Value::String(s)) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_halt() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut compiler = Compiler::new();
        let (bytecode, string_pool) = compiler.compile(&ast).unwrap();
        let mut vm = Vm::new(bytecode, string_pool);
        let result = vm.run().unwrap();
        assert!(matches!(result, None));
    }

    #[test]
    fn test_hot_reload() {
        let mut lexer = Lexer::new("42;");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut compiler = Compiler::new();
        let (bc, sp) = compiler.compile(&ast).unwrap();
        let mut vm = Vm::new(bc, sp);
        vm.run().unwrap();

        // Reload with new code
        let mut lexer2 = Lexer::new("100;");
        let tokens2 = lexer2.tokenize().unwrap();
        let mut parser2 = Parser::new(tokens2);
        let ast2 = parser2.parse().unwrap();
        let mut compiler2 = Compiler::new();
        let (bc2, sp2) = compiler2.compile(&ast2).unwrap();
        vm.hot_reload(bc2, sp2);
        let result = vm.run().unwrap();
        assert_eq!(extract_int(result), 100);
    }
}

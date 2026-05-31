#![deny(clippy::all)]

use crate::compiler::Bytecode;
use thiserror::Error;

#[derive(Debug, Clone)]
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
    #[error("Unknown opcode")]
    UnknownOpcode,
    #[error("Type mismatch: cannot {op} {lhs:?} with {rhs:?}")]
    TypeError {
        op: String,
        lhs: Value,
        rhs: Value,
    },
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Halt")]
    Halt,
}

pub struct Vm {
    bytecode: Vec<Bytecode>,
    ip: usize,          // instruction pointer
    stack: Vec<Value>,
    call_stack: Vec<usize>,
    should_halt: bool,
}

impl Vm {
    pub fn new(bytecode: Vec<Bytecode>) -> Self {
        Self {
            bytecode,
            ip: 0,
            stack: Vec::new(),
            call_stack: Vec::new(),
            should_halt: false,
        }
    }

    pub fn run(&mut self) -> Result<Option<Value>, VmError> {
        while !self.should_halt && self.ip < self.bytecode.len() {
            let op = self.bytecode[self.ip].clone();
            self.ip += 1;
            self.execute(op)?;
        }
        Ok(self.stack.pop())
    }

    pub fn step(&mut self) -> Result<(), VmError> {
        if self.should_halt || self.ip >= self.bytecode.len() {
            return Err(VmError::Halt);
        }
        let op = self.bytecode[self.ip].clone();
        self.ip += 1;
        self.execute(op)
    }

    fn execute(&mut self, op: Bytecode) -> Result<(), VmError> {
        match op {
            Bytecode::Nop => {}
            Bytecode::PushInt(val) => self.stack.push(Value::Int(val)),
            Bytecode::PushFloat(val) => self.stack.push(Value::Float(val)),
            Bytecode::PushBool(val) => self.stack.push(Value::Bool(val)),
            Bytecode::PushString(idx) => {
                self.stack.push(Value::String(format!("str_{}", idx)));
            }
            Bytecode::PushVec3(x, y, z) => self.stack.push(Value::Vec3(x, y, z)),
            Bytecode::PushNull => self.stack.push(Value::Null),
            Bytecode::Pop => {
                self.stack.pop().ok_or(VmError::StackUnderflow)?;
            }
            Bytecode::Dup => {
                let val = self.stack.last().ok_or(VmError::StackUnderflow)?.clone();
                self.stack.push(val);
            }
            Bytecode::Add => {
                let rhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.stack.push(add_values(lhs, rhs)?);
            }
            Bytecode::Sub => {
                let rhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.stack.push(sub_values(lhs, rhs)?);
            }
            Bytecode::Mul => {
                let rhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.stack.push(mul_values(lhs, rhs)?);
            }
            Bytecode::Div => {
                let rhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.stack.push(div_values(lhs, rhs)?);
            }
            Bytecode::Neg => {
                let val = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.stack.push(neg_value(val)?);
            }
            Bytecode::Eq => {
                let rhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                let lhs = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                self.stack.push(Value::Bool(values_equal(&lhs, &rhs)));
            }
            Bytecode::Halt => {
                self.should_halt = true;
            }
            _ => {} // Placeholder for other opcodes
        }
        Ok(())
    }

    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn reset(&mut self, bytecode: Vec<Bytecode>) {
        self.bytecode = bytecode;
        self.ip = 0;
        self.stack.clear();
        self.call_stack.clear();
        self.should_halt = false;
    }
}

fn add_values(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
        (Value::Int(l), Value::Float(r)) => Ok(Value::Float(l as f64 + r)),
        (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l + r as f64)),
        (l, r) => Err(VmError::TypeError {
            op: "add".to_string(),
            lhs: l,
            rhs: r,
        }),
    }
}

fn sub_values(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
        (Value::Int(l), Value::Float(r)) => Ok(Value::Float(l as f64 - r)),
        (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l - r as f64)),
        (l, r) => Err(VmError::TypeError {
            op: "sub".to_string(),
            lhs: l,
            rhs: r,
        }),
    }
}

fn mul_values(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
        (Value::Int(l), Value::Float(r)) => Ok(Value::Float(l as f64 * r)),
        (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l * r as f64)),
        (l, r) => Err(VmError::TypeError {
            op: "mul".to_string(),
            lhs: l,
            rhs: r,
        }),
    }
}

fn div_values(a: Value, b: Value) -> Result<Value, VmError> {
    match (a, b) {
        (_, Value::Int(0)) | (_, Value::Float(0.0)) => Err(VmError::DivisionByZero),
        (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l / r)),
        (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l / r)),
        (Value::Int(l), Value::Float(r)) => Ok(Value::Float(l as f64 / r)),
        (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l / r as f64)),
        (l, r) => Err(VmError::TypeError {
            op: "div".to_string(),
            lhs: l,
            rhs: r,
        }),
    }
}

fn neg_value(a: Value) -> Result<Value, VmError> {
    match a {
        Value::Int(v) => Ok(Value::Int(-v)),
        Value::Float(v) => Ok(Value::Float(-v)),
        v => Err(VmError::TypeError {
            op: "neg".to_string(),
            lhs: v,
            rhs: Value::Null,
        }),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(l), Value::Int(r)) => l == r,
        (Value::Float(l), Value::Float(r)) => (l - r).abs() < f64::EPSILON,
        (Value::Bool(l), Value::Bool(r)) => l == r,
        (Value::String(l), Value::String(r)) => l == r,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_bytecode(bc: Vec<Bytecode>) -> Result<Option<Value>, VmError> {
        let mut vm = Vm::new(bc);
        vm.run()
    }

    #[test]
    fn test_push_int() {
        let result = run_bytecode(vec![Bytecode::PushInt(42), Bytecode::Halt]);
        assert!(matches!(result, Ok(Some(Value::Int(42)))));
    }

    #[test]
    fn test_add_ints() {
        let result = run_bytecode(vec![
            Bytecode::PushInt(10),
            Bytecode::PushInt(20),
            Bytecode::Add,
            Bytecode::Halt,
        ]);
        assert!(matches!(result, Ok(Some(Value::Int(30)))));
    }

    #[test]
    fn test_sub_floats() {
        let result = run_bytecode(vec![
            Bytecode::PushFloat(10.0),
            Bytecode::PushFloat(3.0),
            Bytecode::Sub,
            Bytecode::Halt,
        ]);
        assert!(matches!(result, Ok(Some(Value::Float(v))) if (v - 7.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_mul_mixed() {
        let result = run_bytecode(vec![
            Bytecode::PushInt(5),
            Bytecode::PushFloat(2.5),
            Bytecode::Mul,
            Bytecode::Halt,
        ]);
        assert!(matches!(result, Ok(Some(Value::Float(v))) if (v - 12.5).abs() < f64::EPSILON));
    }

    #[test]
    fn test_div_by_zero() {
        let result = run_bytecode(vec![
            Bytecode::PushInt(10),
            Bytecode::PushInt(0),
            Bytecode::Div,
            Bytecode::Halt,
        ]);
        assert!(matches!(result, Err(VmError::DivisionByZero)));
    }

    #[test]
    fn test_negate() {
        let result = run_bytecode(vec![Bytecode::PushInt(7), Bytecode::Neg, Bytecode::Halt]);
        assert!(matches!(result, Ok(Some(Value::Int(-7)))));
    }

    #[test]
    fn test_equality() {
        let result = run_bytecode(vec![
            Bytecode::PushInt(5),
            Bytecode::PushInt(5),
            Bytecode::Eq,
            Bytecode::Halt,
        ]);
        assert!(matches!(result, Ok(Some(Value::Bool(true)))));
    }

    #[test]
    fn test_stack_underflow() {
        let result = run_bytecode(vec![Bytecode::Pop, Bytecode::Halt]);
        assert!(matches!(result, Err(VmError::StackUnderflow)));
    }

    #[test]
    fn test_halt() {
        let result = run_bytecode(vec![Bytecode::Halt]);
        assert!(matches!(result, Ok(None)));
    }
}

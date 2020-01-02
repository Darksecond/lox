use crate::bettergc::{Trace, Gc};
use crate::bytecode::ChunkIndex;

pub struct NativeFunction {
    pub name: String,
    pub code: fn(&[Value]) -> Value,
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native function {}>", self.name)
    }
}

impl Trace for NativeFunction { fn trace(&self) {} }

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub chunk_index: ChunkIndex,
    pub arity: usize,
}

impl Trace for Function { fn trace(&self) {} }

impl From<&crate::bytecode::Function> for Function {
    fn from(value: &crate::bytecode::Function) -> Self {
        Function {
            name: value.name.clone(),
            chunk_index: value.chunk_index,
            arity: value.arity,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Object {
    String(Gc<String>),
    Function(Gc<Function>),
    NativeFunction(Gc<NativeFunction>),
}

impl Trace for Object {
    fn trace(&self) {
        match self {
            Object::String(string) => string.trace(),
            Object::Function(function) => function.trace(),
            Object::NativeFunction(function) => function.trace(),
        }
    }
}

#[derive(Debug, Copy, Clone)] //TODO Double check we want Copy
pub enum Value {
    Number(f64),
    Object(Object),
    Nil,
    True,
    False,
}

impl Trace for Value {
    fn trace(&self) {
        match self {
            Value::Object(obj) => obj.trace(),
            _ => (),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::False => true,
            Value::Nil => true,
            _ => false,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        if value {
            Value::True
        } else {
            Value::False
        }
    }
}
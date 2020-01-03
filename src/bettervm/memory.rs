use crate::bettergc::{Trace, Gc};
use crate::bytecode::ChunkIndex;
use std::cell::RefCell;

#[derive(Debug, Copy, Clone)]
pub enum Upvalue {
    Open(usize),
    Closed(Value),
}

impl Upvalue {
    pub fn is_open_with_index(&self, index: usize) -> bool {
        match self {
            Self::Open(i) => {
                if *i == index { true } else { false }
            },
            Self::Closed(_) => false,
        }
    }
}

impl Trace for Upvalue {
    fn trace(&self) {
        match self {
            Upvalue::Closed(value) => value.trace(),
            Upvalue::Open(_) => (),
        }
    }
}

#[derive(Debug)]
pub struct Closure {
    pub function: Gc<Function>,
    pub upvalues: Vec<Gc<RefCell<Upvalue>>>,
}

impl Trace for Closure {
    fn trace(&self) {
        self.function.trace();
        self.upvalues.trace();
    }
}

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

//TODO Think about merging this into Value
//     It's no longer stringly required as a seperate thing.
#[derive(Debug, Copy, Clone)]
pub enum Object {
    String(Gc<String>),
    Closure(Gc<Closure>),
    NativeFunction(Gc<NativeFunction>),
}

impl Trace for Object {
    fn trace(&self) {
        match self {
            Object::String(string) => string.trace(),
            Object::NativeFunction(function) => function.trace(),
            Object::Closure(closure) => closure.trace(),
        }
    }
}

//TODO Possibly merge True and False
#[derive(Debug, Copy, Clone)]
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
use crate::bettergc::{Trace, Gc};
use crate::bytecode::ChunkIndex;
use std::cell::RefCell;
use std::collections::HashMap;

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

    pub fn is_open(&self) -> bool {
        match self {
            Self::Open(_) => true,
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
pub struct Instance {
    pub class: Gc<RefCell<Class>>,
    pub fields: HashMap<String, Value>,
}

impl Trace for Instance {
    fn trace(&self) {
        self.class.trace();
        self.fields.trace();
    }
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
}

impl Trace for Class {
    fn trace(&self) {
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

//TODO Drop this entirely and merge this into Closure
//     We'll wait and see how methods will be implemented before we do this though
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
pub enum Value {
    Number(f64),
    String(Gc<String>),
    Closure(Gc<Closure>),
    NativeFunction(Gc<NativeFunction>),
    Boolean(bool),
    Class(Gc<RefCell<Class>>),
    Instance(Gc<RefCell<Instance>>),
    Nil,
}

impl Trace for Value {
    fn trace(&self) {
        match self {
            Value::String(string) => string.trace(),
            Value::NativeFunction(function) => function.trace(),
            Value::Closure(closure) => closure.trace(),
            Value::Class(class) => class.trace(),
            Value::Instance(instance) => instance.trace(),
            Value::Number(_) => (),
            Value::Nil => (),
            Value::Boolean(_) => (),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Boolean(boolean) => !boolean,
            Value::Nil => true,
            _ => false,
        }
    }

    pub fn is_same_type(a: &Value, b: &Value) -> bool {
        match (b,a) {
            (Value::Number(_), Value::Number(_)) => true,
            (Value::Boolean(_), Value::Boolean(_)) => true,
            (Value::String(_), Value::String(_)) => true,
            (Value::NativeFunction(_), Value::NativeFunction(_)) => true,
            (Value::Closure(_), Value::Closure(_)) => true,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        if value {
            Value::Boolean(true)
        } else {
            Value::Boolean(false)
        }
    }
}
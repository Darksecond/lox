mod object;
mod import;
mod list;

pub use object::*;
pub use import::*;
pub use list::*;

use crate::value::Value;
use super::gc::{Gc, Trace};
use lox_bytecode::bytecode::ChunkIndex;
use std::cell::{Cell, UnsafeCell};
use super::interner::Symbol;
use super::table::Table;

#[derive(Debug, Copy, Clone)]
pub enum Upvalue {
    Open(usize),
    Closed(Value),
}

impl Upvalue {
    pub const fn is_open_with_range(&self, index: usize) -> Option<usize> {
        match self {
            Self::Open(i) => {
                if *i >= index {
                    Some(*i)
                } else {
                    None
                }
            }
            Self::Closed(_) => None,
        }
    }

    pub const fn is_open_with_index(&self, index: usize) -> bool {
        match self {
            Self::Open(i) => {
                *i == index
            }
            Self::Closed(_) => false,
        }
    }

    pub const fn is_open(&self) -> bool {
        match self {
            Self::Open(_) => true,
            Self::Closed(_) => false,
        }
    }
}

impl Trace for Upvalue {
    #[inline]
    fn trace(&self) {
        match self {
            Upvalue::Closed(value) => value.trace(),
            Upvalue::Open(_) => (),
        }
    }
}

#[derive(Debug)]
pub struct Instance {
    pub class: Gc<Object<Class>>,
    fields: UnsafeCell<Table>,
}

impl Instance {
    pub fn new(klass: Gc<Object<Class>>) -> Self {
        Self {
            class: klass,
            fields: Default::default(),
        }
    }

    #[inline]
    pub fn field(&self, symbol: Symbol) -> Option<Value> {
        self.fields().get(symbol)
    }

    pub fn set_field(&self, symbol: Symbol, value: Value) {
        let fields = unsafe { &mut *self.fields.get() };
        fields.set(symbol, value);
    }

    fn fields(&self) -> &Table {
        unsafe {
            &*self.fields.get()
        }
    }
}

impl Trace for Instance {
    #[inline]
    fn trace(&self) {
        self.class.trace();
        self.fields().trace();
    }

    fn size_hint(&self) -> usize {
        let fields = unsafe { &*self.fields.get() };
        fields.capacity() * std::mem::size_of::<Value>()
    }
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    methods: UnsafeCell<Table>,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self {
            name,
            methods: Default::default(),
        }
    }

    #[inline]
    pub fn method(&self, symbol: Symbol) -> Option<Value> {
        self.methods().get(symbol)
    }

    // Make closure Gc<ErasedObject>
    pub fn set_method(&self, symbol: Symbol, closure: Value) {
        let methods = unsafe { &mut *self.methods.get() };
        methods.set(symbol, closure);
    }

    fn methods(&self) -> &Table {
        unsafe {
            &*self.methods.get()
        }
    }
}

impl Trace for Class {
    #[inline]
    fn trace(&self) {
        self.methods().trace();
    }
}

#[derive(Debug)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<Gc<Cell<Upvalue>>>,
}

impl Trace for Closure {
    #[inline]
    fn trace(&self) {
        self.upvalues.trace();
        self.function.import.trace();
    }
}

pub struct NativeFunction {
    pub name: String,
    pub code: fn(Value, &[Value]) -> Value,
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native function {}>", self.name)
    }
}

impl Trace for NativeFunction {
    #[inline]
    fn trace(&self) {}
}

//TODO Drop this entirely and merge this into Closure
pub struct Function {
    pub name: String,
    pub chunk_index: ChunkIndex,
    pub import: Gc<Object<Import>>,
    pub arity: usize,
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
        .field("name", &self.name)
        .finish()
    }
}

impl Function {
    pub(crate) fn new(value: &lox_bytecode::bytecode::Function, import: Gc<Object<Import>>) -> Self {
        Self {
            name: value.name.clone(),
            chunk_index: value.chunk_index,
            arity: value.arity,
            import,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BoundMethod {
    pub receiver: Gc<ErasedObject>,
    pub method: Value,
}

impl Trace for BoundMethod {
    #[inline]
    fn trace(&self) {
        self.receiver.trace();
        self.method.trace();
    }
}

use lox_bytecode::bytecode::{Chunk, Constant, ConstantIndex, Module, ClosureIndex, ClassIndex};

use crate::bettergc::{Gc, Trace};
use crate::bytecode::{ChunkIndex, self};
use std::cell::{Cell, UnsafeCell};
use fxhash::FxHashMap;
use super::interner::{Symbol, Interner};

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
                if *i == index {
                    true
                } else {
                    false
                }
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
    pub class: Gc<Class>,
    fields: UnsafeCell<FxHashMap<Symbol, Value>>,
}

impl Instance {
    pub fn new(klass: Gc<Class>) -> Self {
        Self {
            class: klass,
            fields: Default::default(),
        }
    }

    pub fn field(&self, symbol: &Symbol) -> Option<Value> {
        self.fields().get(symbol).cloned()
    }

    pub fn set_field(&self, symbol: Symbol, value: Value) {
        self.fields_mut().insert(symbol, value);
    }

    fn fields(&self) -> &FxHashMap<Symbol, Value> {
        unsafe {
            &*self.fields.get()
        }
    }

    fn fields_mut(&self) -> &mut FxHashMap<Symbol, Value> {
        unsafe {
            &mut *self.fields.get()
        }
    }
}

impl Trace for Instance {
    #[inline]
    fn trace(&self) {
        self.class.trace();
        self.fields().trace();
    }
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    methods: UnsafeCell<FxHashMap<Symbol, Gc<Closure>>>,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self {
            name,
            methods: Default::default(),
        }
    }

    pub fn method(&self, symbol: Symbol) -> Option<Gc<Closure>> {
        self.methods().get(&symbol).cloned()
    }

    pub fn set_method(&self, symbol: Symbol, closure: Gc<Closure>) {
        self.methods_mut().insert(symbol, closure);
    }

    fn methods(&self) -> &FxHashMap<Symbol, Gc<Closure>> {
        unsafe {
            &*self.methods.get()
        }
    }

    fn methods_mut(&self) -> &mut FxHashMap<Symbol, Gc<Closure>> {
        unsafe {
            &mut *self.methods.get()
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
    pub code: fn(&[Value]) -> Value,
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
    pub import: Gc<Import>,
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
    pub fn new(value: &crate::bytecode::Function, import: Gc<Import>) -> Self {
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
    pub receiver: Gc<Instance>,
    pub method: Gc<Closure>,
}

impl Trace for BoundMethod {
    #[inline]
    fn trace(&self) {
        self.receiver.trace();
        self.method.trace();
    }
}

#[derive(Debug)]
pub struct Import {
    module: Module,
    pub globals: UnsafeCell<FxHashMap<Symbol, Value>>,
    pub symbols: Vec<Symbol>,
}

impl Trace for Import {
    #[inline]
    fn trace(&self) {
        self.globals().trace();
    }
}

impl Import {
    pub fn new(module: Module, interner: &mut Interner) -> Self {
        let symbols = module.identifiers().iter().map(|identifier| {
            interner.intern(identifier)
        }).collect();

        Self {
            module,
            globals: UnsafeCell::new(FxHashMap::default()),
            symbols,
        }
    }

    fn globals(&self) -> &FxHashMap<Symbol, Value> {
        unsafe {
            &*self.globals.get()
        }
    }

    fn globals_mut(&self) -> &mut FxHashMap<Symbol, Value> {
        unsafe {
            &mut *self.globals.get()
        }
    }

    pub fn symbol(&self, index: ConstantIndex) -> Symbol {
        unsafe {
            *self.symbols.get_unchecked(index)
        }
    }

    pub fn chunk(&self, index: usize) -> &Chunk {
        self.module.chunk(index)
    }

    pub fn constant(&self, index: ConstantIndex) -> &Constant {
        self.module.constant(index)
    }

    //TODO rename to make it clear this is not an alive closure.
    pub fn class(&self, index: ClassIndex) -> &bytecode::Class {
        self.module.class(index)
    }

    //TODO rename to make it clear this is not an alive closure.
    pub fn closure(&self, index: ClosureIndex) -> &bytecode::Closure {
        self.module.closure(index)
    }

    pub fn set_global(&self, key: Symbol, value: Value) {
        self.globals_mut().insert(key, value);
    }

    pub fn has_global(&self, key: Symbol) -> bool {
        self.globals().contains_key(&key)
    }

    pub fn global(&self, key: Symbol) -> Option<Value> {
        self.globals().get(&key).cloned()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Number(f64),
    String(Gc<String>),
    Closure(Gc<Closure>),
    BoundMethod(Gc<BoundMethod>),
    NativeFunction(Gc<NativeFunction>),
    Boolean(bool),
    Class(Gc<Class>),
    Instance(Gc<Instance>),
    Import(Gc<Import>),
    Nil,
}

impl Trace for Value {
    #[inline]
    fn trace(&self) {
        match self {
            Value::String(string) => string.trace(),
            Value::NativeFunction(function) => function.trace(),
            Value::Closure(closure) => closure.trace(),
            Value::BoundMethod(bound_method) => bound_method.trace(),
            Value::Class(class) => class.trace(),
            Value::Instance(instance) => instance.trace(),
            Value::Number(_) => (),
            Value::Nil => (),
            Value::Boolean(_) => (),
            Value::Import(import) => import.trace(),
        }
    }
}

impl Value {
    pub const fn is_falsey(&self) -> bool {
        match self {
            Value::Boolean(true) => false,
            Value::Boolean(false) => true,
            Value::Nil => true,
            _ => false,
        }
    }

    pub const fn is_same_type(a: &Value, b: &Value) -> bool {
        match (b, a) {
            (Value::Number(_), Value::Number(_)) => true,
            (Value::Boolean(_), Value::Boolean(_)) => true,
            (Value::String(_), Value::String(_)) => true,
            (Value::NativeFunction(_), Value::NativeFunction(_)) => true,
            (Value::Closure(_), Value::Closure(_)) => true,
            (Value::BoundMethod(_), Value::BoundMethod(_)) => true,
            (Value::Nil, Value::Nil) => true,
            (Value::Class(_), Value::Class(_)) => true,
            (Value::Instance(_), Value::Instance(_)) => true,
            (Value::Import(_), Value::Import(_)) => true,
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

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Nil => write!(f, "nil"),
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::String(string) => write!(f, "{}", string),
            Value::NativeFunction(_function) => write!(f, "<native fn>"),
            Value::Closure(closure) => write!(f, "<fn {}>", closure.function.name),
            Value::Class(class) => write!(f, "{}", class.name),
            Value::Instance(instance) => write!(f, "{} instance", instance.class.name),
            Value::BoundMethod(bind) => write!(f, "<fn {}>", bind.method.function.name),
            Value::Import(_) => write!(f, "<import>"),
        }
    }
}

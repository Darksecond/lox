use lox_bytecode::bytecode::{Chunk, Constant, ConstantIndex, Module, ClosureIndex, ClassIndex};

use super::gc::{Gc, Trace};
use lox_bytecode::bytecode::{ChunkIndex, self};
use std::cell::{Cell, UnsafeCell};
use super::interner::{Symbol, Interner};

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
    pub class: Gc<Class>,
    fields: UnsafeCell<Table>,
}

impl Instance {
    pub fn new(klass: Gc<Class>) -> Self {
        Self {
            class: klass,
            fields: Default::default(),
        }
    }

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

    pub fn method(&self, symbol: Symbol) -> Option<Value> {
        self.methods().get(symbol)
    }

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
    pub(crate) fn new(value: &lox_bytecode::bytecode::Function, import: Gc<Import>) -> Self {
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
    pub method: Value,
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
    globals: UnsafeCell<Table>,
    symbols: Vec<Symbol>,
}

impl Trace for Import {
    #[inline]
    fn trace(&self) {
        self.globals().trace();
    }
}

impl Import {
    pub fn new() -> Self {
        Self {
            module: Module::new(),
            globals: Default::default(),
            symbols: Default::default(),
        }
    }

    pub(crate) fn with_module(module: Module, interner: &mut Interner) -> Self {
        let symbols = module.identifiers().iter().map(|identifier| {
            interner.intern(identifier)
        }).collect();

        Self {
            module,
            globals: Default::default(),
            symbols,
        }
    }

    pub fn copy_to(&self, other: &Import) {
        let dst = unsafe { &mut *other.globals.get() };
        self.globals().copy_to(dst);
    }

    fn globals(&self) -> &Table {
        unsafe {
            &*self.globals.get()
        }
    }

    #[inline]
    pub(crate) fn symbol(&self, index: ConstantIndex) -> Symbol {
        unsafe {
            *self.symbols.get_unchecked(index)
        }
    }

    pub(crate) fn chunk(&self, index: usize) -> &Chunk {
        self.module.chunk(index)
    }

    #[inline]
    pub(crate) fn constant(&self, index: ConstantIndex) -> &Constant {
        self.module.constant(index)
    }

    //TODO rename to make it clear this is not an alive closure.
    pub(crate) fn class(&self, index: ClassIndex) -> &bytecode::Class {
        self.module.class(index)
    }

    //TODO rename to make it clear this is not an alive closure.
    pub(crate) fn closure(&self, index: ClosureIndex) -> &bytecode::Closure {
        self.module.closure(index)
    }

    pub fn set_global(&self, key: Symbol, value: Value) {
        let globals = unsafe { &mut *self.globals.get() };
        globals.set(key, value);
    }

    pub fn has_global(&self, key: Symbol) -> bool {
        self.globals().has(key)
    }

    #[inline]
    pub fn global(&self, key: Symbol) -> Option<Value> {
        self.globals().get(key)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Nil,
    Number(f64),
    String(Gc<String>),
    Closure(Gc<Closure>),
    BoundMethod(Gc<BoundMethod>),
    NativeFunction(Gc<NativeFunction>),
    Boolean(bool),
    Class(Gc<Class>),
    Instance(Gc<Instance>),
    Import(Gc<Import>),
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
        matches!((b,a), 
                 (Value::Number(_), Value::Number(_))
                 | (Value::Boolean(_), Value::Boolean(_))
                 | (Value::String(_), Value::String(_))
                 | (Value::NativeFunction(_), Value::NativeFunction(_))
                 | (Value::Closure(_), Value::Closure(_))
                 | (Value::BoundMethod(_), Value::BoundMethod(_))
                 | (Value::Nil, Value::Nil)
                 | (Value::Class(_), Value::Class(_))
                 | (Value::Instance(_), Value::Instance(_))
                 | (Value::Import(_), Value::Import(_))
                )
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
            Value::BoundMethod(bind) => write!(f, "<bound {}>", bind.method),
            Value::Import(_) => write!(f, "<import>"),
        }
    }
}

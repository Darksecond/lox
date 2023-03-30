use lox_bytecode::bytecode::{Chunk, Constant, ConstantIndex, Module, ClosureIndex, ClassIndex};
use crate::value::Value;

use super::gc::{Gc, Trace};
use lox_bytecode::bytecode::{ChunkIndex, self};
use std::cell::{Cell, UnsafeCell};
use std::fmt::Display;
use std::ops::Deref;
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
    pub receiver: Gc<Object<Instance>>,
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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ObjectTag {
    String,
    Closure,
    BoundMethod,
    NativeFunction,
    Class,
    Instance,
    Import,
}

#[derive(Debug)]
#[repr(C)]
pub struct ErasedObject {
    pub tag: ObjectTag,
}

impl ErasedObject {
    fn as_object<T: Trace>(&self) -> &T {
        let ptr = self as *const ErasedObject;
        let ptr = ptr as *const Object<T>;
        unsafe {
            &*ptr
        }
    }
}

impl Trace for ErasedObject {
    fn trace(&self) {
        match self.tag {
            ObjectTag::String => self.as_object::<String>().trace(),
            ObjectTag::Closure => self.as_object::<Closure>().trace(),
            ObjectTag::BoundMethod => self.as_object::<BoundMethod>().trace(),
            ObjectTag::NativeFunction => self.as_object::<NativeFunction>().trace(),
            ObjectTag::Class => self.as_object::<Class>().trace(),
            ObjectTag::Instance => self.as_object::<Instance>().trace(),
            ObjectTag::Import => self.as_object::<Import>().trace(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Object<T> {
    pub tag: ObjectTag,
    data: T,
}

impl<T> Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Trace> Trace for Object<T> {
    fn trace(&self) {
        self.data.trace();
    }
}

impl Object<NativeFunction> {
    pub fn from_native_function(data: NativeFunction) -> Object<NativeFunction> {
        Self {
            tag: ObjectTag::NativeFunction,
            data,
        }
    }
}

impl From<Closure> for Object<Closure> {
    fn from(value: Closure) -> Self {
        Self {
            tag: ObjectTag::Closure,
            data: value,
        }
    }
}

impl From<Instance> for Object<Instance> {
    fn from(value: Instance) -> Self {
        Self {
            tag: ObjectTag::Instance,
            data: value,
        }
    }
}

impl From<Class> for Object<Class> {
    fn from(value: Class) -> Self {
        Self {
            tag: ObjectTag::Class,
            data: value,
        }
    }
}

impl From<String> for Object<String> {
    fn from(value: String) -> Self {
        Self {
            tag: ObjectTag::String,
            data: value,
        }
    }
}

impl From<Import> for Object<Import> {
    fn from(value: Import) -> Self {
        Self {
            tag: ObjectTag::Import,
            data: value,
        }
    }
}

impl From<BoundMethod> for Object<BoundMethod> {
    fn from(value: BoundMethod) -> Self {
        Self {
            tag: ObjectTag::BoundMethod,
            data: value,
        }
    }
}

impl<T> From<Gc<Object<T>>> for Gc<ErasedObject> where T: Trace {
    fn from(value: Gc<Object<T>>) -> Self {
        unsafe {
            Gc::from_bits(value.to_bits())
        }
    }
}

impl Gc<ErasedObject> {
    pub fn as_string(self) -> Gc<Object<String>> {
        debug_assert_eq!(self.tag, ObjectTag::String);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_closure(self) -> Gc<Object<Closure>> {
        debug_assert_eq!(self.tag, ObjectTag::Closure);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_bound_method(self) -> Gc<Object<BoundMethod>> {
        debug_assert_eq!(self.tag, ObjectTag::BoundMethod);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_native_function(self) -> Gc<Object<NativeFunction>> {
        debug_assert_eq!(self.tag, ObjectTag::NativeFunction);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_class(self) -> Gc<Object<Class>> {
        debug_assert_eq!(self.tag, ObjectTag::Class);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_instance(self) -> Gc<Object<Instance>> {
        debug_assert_eq!(self.tag, ObjectTag::Instance);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_import(self) -> Gc<Object<Import>> {
        debug_assert_eq!(self.tag, ObjectTag::Import);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }
}

impl<T> Object<T> {
    pub fn is_same_type(a: &Self, b: &Self) -> bool {
        a.tag == b.tag
    }
}

impl ErasedObject {
    pub fn is_same_type(a: &Self, b: &Self) -> bool {
        a.tag == b.tag
    }
}

impl PartialEq for Gc<ErasedObject> {
    fn eq(&self, other: &Self) -> bool {
        if self.tag == ObjectTag::String && other.tag == ObjectTag::String {
            self.as_string().as_str() == other.as_string().as_str()
        } else {
            Gc::ptr_eq(self, other)
        }
    }
}

impl Display for Gc<ErasedObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            ObjectTag::String => write!(f, "{}", self.as_string().as_str()),
            ObjectTag::Closure => write!(f, "<fn {}>", self.as_closure().function.name),
            ObjectTag::BoundMethod => write!(f, "<bound {}>", self.as_bound_method().method),
            ObjectTag::NativeFunction => write!(f, "<native fn>"),
            ObjectTag::Class => write!(f, "{}", self.as_class().name),
            ObjectTag::Instance => write!(f, "{} instance", self.as_instance().class.name),
            ObjectTag::Import => write!(f, "<import>"),
        }
    }
}

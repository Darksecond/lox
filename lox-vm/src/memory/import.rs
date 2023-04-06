use lox_bytecode::bytecode::{Chunk, ConstantIndex, Module, ClosureIndex, ClassIndex};
use crate::gc::{Trace, Heap, Gc};
use std::cell::UnsafeCell;
use crate::interner::{Symbol, Interner};
use lox_bytecode::bytecode;
use crate::table::Table;
use crate::value::Value;
use crate::memory::Object;

#[derive(Debug)]
pub struct Import {
    pub name: String,
    module: Module,
    globals: UnsafeCell<Table>,
    symbols: Vec<Symbol>,
    strings: Vec<Gc<Object<String>>>,
}

impl Trace for Import {
    #[inline]
    fn trace(&self) {
        self.globals().trace();
        self.strings.trace();
    }
}

impl Import {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            module: Module::new(),
            globals: Default::default(),
            symbols: Default::default(),
            strings: Default::default(),
        }
    }

    pub(crate) fn with_module(name: impl Into<String>, module: Module, interner: &mut Interner, heap: &Heap) -> Self {
        let symbols = module.identifiers().iter().map(|identifier| {
            interner.intern(identifier)
        }).collect();

        let strings = module.strings.iter().map(|value| {
            heap.manage(value.clone().into())
        }).collect();

        Self {
            name: name.into(),
            module,
            globals: Default::default(),
            symbols,
            strings,
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
    pub(crate) fn number(&self, index: ConstantIndex) -> f64 {
        self.module.number(index)
    }

    #[inline]
    pub(crate) fn string(&self, index: ConstantIndex) -> Gc<Object<String>> {
        unsafe {
            *self.strings.get_unchecked(index)
        }
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

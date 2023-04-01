use lox_bytecode::bytecode::{Chunk, Constant, ConstantIndex, Module, ClosureIndex, ClassIndex};
use crate::gc::Trace;
use std::cell::UnsafeCell;
use crate::interner::{Symbol, Interner};
use lox_bytecode::bytecode;
use crate::table::Table;
use crate::value::Value;

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

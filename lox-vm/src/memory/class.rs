use std::cell::UnsafeCell;
use crate::interner::Symbol;
use crate::table::Table;
use crate::value::Value;
use crate::gc::{Trace, Tracer};
use crate::string::LoxString;

#[derive(Debug)]
pub struct Class {
    pub name: LoxString,
    methods: UnsafeCell<Table>,
}

impl Class {
    pub fn new(name: impl Into<LoxString>) -> Self {
        Self {
            name: name.into(),
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

unsafe impl Trace for Class {
    #[inline]
    fn trace(&self, tracer: &mut Tracer) {
        self.name.trace(tracer);
        self.methods.trace(tracer);
    }
}

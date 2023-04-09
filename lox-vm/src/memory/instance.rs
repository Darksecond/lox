use crate::gc::{Trace, Gc};
use crate::memory::{Object, Class};
use crate::interner::Symbol;
use crate::value::Value;
use crate::table::Table;
use std::cell::UnsafeCell;

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

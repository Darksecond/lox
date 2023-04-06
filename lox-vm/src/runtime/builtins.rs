use crate::gc::{Gc, Heap, Trace};
use crate::memory::{Object, Import, Class};

pub struct Builtins {
    pub list_class: Gc<Object<Class>>,
    pub globals_import: Gc<Object<Import>>,
}

impl Builtins {
    pub fn new(heap: &Heap) -> Self {
        Self {
            globals_import: heap.manage(Import::new("globals").into()),
            list_class: heap.manage(Class::new("List".to_string()).into()),
        }
    }
}

impl Trace for Builtins {
    fn trace(&self) {
        self.list_class.trace();
        self.globals_import.trace();
    }
}

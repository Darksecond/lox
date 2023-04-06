use crate::gc::{Gc, Heap, Trace};
use crate::memory::{Object, Import, Class, ErasedObject};

pub struct Builtins {
    pub list_class: Gc<Object<Class>>,
    pub string_class: Gc<Object<Class>>,
    pub globals_import: Gc<Object<Import>>,
}

impl Builtins {
    pub fn new(heap: &Heap) -> Self {
        Self {
            globals_import: heap.manage(Import::new("globals").into()),
            list_class: heap.manage(Class::new("List".to_string()).into()),
            string_class: heap.manage(Class::new("String".to_string()).into()),
        }
    }

    pub fn class_for_object(&self, object: Gc<ErasedObject>) -> Option<Gc<Object<Class>>> {
        use crate::memory::{Instance, List};

        if object.is::<Instance>() {
            Some(object.cast::<Instance>().class)
        } else if object.is::<List>() {
            Some(self.list_class)
        } else if object.is::<String>() {
            Some(self.string_class)
        } else {
            None
        }
    }
}

impl Trace for Builtins {
    fn trace(&self) {
        self.list_class.trace();
        self.globals_import.trace();
    }
}

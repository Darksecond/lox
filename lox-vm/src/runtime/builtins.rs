use crate::gc::{Gc, Heap, Trace};
use crate::memory::{Object, Import, Class, ErasedObject};

pub struct Builtins {
    pub empty_class: Gc<Object<Class>>,
    pub list_class: Gc<Object<Class>>,
    pub string_class: Gc<Object<Class>>,
    pub globals_import: Gc<Object<Import>>,
}

impl Builtins {
    pub fn new(heap: &Heap) -> Self {
        Self {
            empty_class: heap.manage(Class::new("".to_string()).into()),
            globals_import: heap.manage(Import::new("globals").into()),
            list_class: heap.manage(Class::new("List".to_string()).into()),
            string_class: heap.manage(Class::new("String".to_string()).into()),
        }
    }

    pub fn class_for_object(&self, object: Gc<ErasedObject>) -> Gc<Object<Class>> {
        use crate::memory::{Instance, List};

        if object.is::<Instance>() {
            object.cast::<Instance>().class
        } else if object.is::<List>() {
            self.list_class
        } else if object.is::<String>() {
            self.string_class
        } else {
            self.empty_class
        }
    }
}

impl Trace for Builtins {
    fn trace(&self) {
        self.string_class.trace();
        self.empty_class.trace();
        self.list_class.trace();
        self.globals_import.trace();
    }
}

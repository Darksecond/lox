use crate::gc::{Gc, Trace, Tracer};
use crate::memory::{Import, Class};

pub struct Builtins {
    pub empty_class: Gc<Class>,
    pub list_class: Gc<Class>,
    pub string_class: Gc<Class>,
    pub globals_import: Gc<Import>,
}

impl Builtins {
    pub fn new() -> Self {
        Self {
            empty_class: lox_gc::manage(Class::new("".to_string()).into()),
            globals_import: lox_gc::manage(Import::new("globals").into()),
            list_class: lox_gc::manage(Class::new("List".to_string()).into()),
            string_class: lox_gc::manage(Class::new("String".to_string()).into()),
        }
    }

    pub fn class_for_object(&self, object: Gc<()>) -> Gc<Class> {
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

unsafe impl Trace for Builtins {
    fn trace(&self, tracer: &mut Tracer) {
        self.string_class.trace(tracer);
        self.empty_class.trace(tracer);
        self.list_class.trace(tracer);
        self.globals_import.trace(tracer);
    }
}

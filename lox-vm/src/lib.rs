pub mod gc;
pub mod memory;
pub mod interner;
pub mod value;

mod runtime;
mod stack;
mod ops;
mod fiber;
mod table;

use lox_bytecode::bytecode::Module;
use runtime::Runtime;
use interner::Symbol;
use memory::{Import, NativeFunction, Object, Class};
use value::Value;
use gc::{Gc, Trace};

pub use runtime::VmError;

pub struct VirtualMachine {
    runtime: Runtime,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    pub fn set_stdout(&mut self, print: for<'r> fn(&'r str)) {
        self.runtime.print = print;
    }

    pub fn set_import(&mut self, import: for<'r> fn(&'r str) -> Option<Module>) {
        self.runtime.import = import;
    }

    pub fn interpret(&mut self, module: Module) -> Result<(), VmError> {
        self.runtime.with_module(module);
        self.runtime.interpret()
    }

    pub fn native(&mut self) -> Native {
        Native {
            runtime: &mut self.runtime,
        }
    }
}

pub struct Native<'a> {
    runtime: &'a mut Runtime,
}

impl Native<'_> {
    pub fn intern(&mut self, value: &str) -> Symbol {
        self.runtime.interner.intern(value)
    }

    pub fn manage<T: 'static + Trace>(&self, value: T) -> Gc<T> {
        self.runtime.heap.manage(value)
    }

    pub fn build_fn(&self, identifier: &str, code: fn(Value, &[Value]) -> Value) -> Gc<Object<NativeFunction>> {
        self.runtime.heap.manage((NativeFunction {
            name: identifier.to_string(),
            code,
        }).into())
    }

    pub fn set_fn(&mut self, import: Gc<Object<Import>>, identifier: &str, code: fn(Value, &[Value]) -> Value) {
        let root = self.build_fn(identifier, code);
        let identifier = self.runtime.interner.intern(identifier);
        import.set_global(identifier, Value::from_object(root))
    }

    pub fn set_method(&mut self, class: Gc<Object<Class>>, identifier: &str, code: fn(Value, &[Value]) -> Value) {
        let root = self.build_fn(identifier, code);
        let identifier = self.runtime.interner.intern(identifier);
        class.set_method(identifier, Value::from_object(root));
    }

    pub fn set_global_fn(&mut self, identifier: &str, code: fn(Value, &[Value]) -> Value) {
        self.set_fn(self.global_import(), identifier, code)
    }

    pub fn global_import(&self) -> Gc<Object<Import>> {
        self.runtime.globals_import()
    }

    pub fn list_class(&self) -> Gc<Object<Class>> {
        self.runtime.builtins.list_class
    }

    pub fn string_class(&self) -> Gc<Object<Class>> {
        self.runtime.builtins.string_class
    }

    pub fn add_import(&mut self, import: Gc<Object<Import>>) {
        self.runtime.imports.insert(import.name.clone(), import);
    }
}

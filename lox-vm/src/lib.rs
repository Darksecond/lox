pub mod gc;
pub mod memory;
pub mod interner;

mod runtime;
mod stack;
mod ops;
mod fiber;
mod table;

use lox_bytecode::bytecode::Module;
use runtime::Runtime;
use interner::Symbol;
use memory::{Import, NativeFunction, Value};
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

    pub fn set_fn(&mut self, import: Gc<Import>, identifier: &str, code: fn(&[Value]) -> Value) {
        let native_function = NativeFunction {
            name: identifier.to_string(),
            code,
        };

        let identifier = self.runtime.interner.intern(identifier);
        let root = self.runtime.manage(native_function);

        import.set_global(identifier, Value::NativeFunction(root))
    }

    pub fn set_global_fn(&mut self, identifier: &str, code: fn(&[Value]) -> Value) {
        self.set_fn(self.global_import(), identifier, code)
    }

    pub fn global_import(&self) -> Gc<Import> {
        self.runtime.globals_import()
    }
}

pub mod gc;
pub mod memory;
mod runtime;
mod interner;
mod stack;
mod ops;
mod fiber;
mod table;

use lox_bytecode::bytecode::Module;
use self::{runtime::Runtime, memory::Value};

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

    pub fn set_native_fn(&mut self, identifier: &str, code: fn(&[Value]) -> Value) {
        self.runtime.set_native_fn(identifier, code)
    }
}

/// Add the lox standard library to a Vm instance.
/// Right now the stdlib consists of 'clock'.
pub fn set_stdlib(outer: &mut VirtualMachine) {
    outer.set_native_fn("clock", |_args| {
        use std::time::{SystemTime, UNIX_EPOCH};

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        memory::Value::Number(time)
    });
}

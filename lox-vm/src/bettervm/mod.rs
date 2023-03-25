mod memory;
mod vm;
mod interner;
mod stack;
mod ops;
mod fiber;
mod table;

use crate::bytecode::Module;
pub use vm::VmError;
use self::{vm::Runtime, memory::Value};

//TODO consider removing
pub struct Vm {
    pub runtime: Runtime, //TODO Replace with Root<RefCell<Fiber>>
}

impl Vm {
    pub fn with_stdout(module: Module, print: for<'r> fn(&'r str), import: for<'r> fn(&'r str) -> Option<Module>) -> Self {
        let mut vm = Runtime::new(print, import);
        vm.with_module(module);

        Self {
            runtime: vm,
        }
    }

    pub fn interpret(&mut self) -> Result<(), VmError> {
        self.runtime.interpret()
    }

    pub fn set_native_fn(&mut self, identifier: &str, code: fn(&[Value]) -> Value) {
        self.runtime.set_native_fn(identifier, code)
    }
}

/// Add the lox standard library to a Vm instance.
/// Right now the stdlib consists of 'clock'.
pub fn set_stdlib(outer: &mut Vm) {
    outer.set_native_fn("clock", |_args| {
        use std::time::{SystemTime, UNIX_EPOCH};

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        memory::Value::Number(time)
    });
}

pub fn execute(module: Module, import: for<'r> fn(&'r str) -> Option<Module>) -> Result<(), VmError> {
    let mut vm = Vm::with_stdout(module, print_stdout, import);
    set_stdlib(&mut vm);

    vm.interpret()
}

pub fn print_stdout(value: &str) {
    println!("{}", value);
}

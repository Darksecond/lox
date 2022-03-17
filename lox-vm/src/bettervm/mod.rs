mod memory;
pub mod vm;
mod interner;
mod context;

use std::io::{Write, stdout};
use crate::{bytecode::Module, bettergc::UniqueRoot};
pub use vm::VmError;
use self::{vm::{Fiber, InterpretResult}, context::VmContext, memory::Value};

pub struct Vm<W> where W: Write {
    pub vm: UniqueRoot<Fiber>, //TODO Replace with Root<RefCell<Fiber>>
    pub context: VmContext<W>, //TODO Replace with Root<VmContext<W>>
}

impl<W> Vm<W> where W: Write {
    pub fn with_stdout(module: Module, stdout: W) -> Self {
        let mut context = VmContext::new(stdout);
        let closure = context.prepare_interpret(module);
        let vm = context.unique(Fiber::with_closure(closure.as_gc()));
        
        Self {
            context,
            vm,
        }
    }

    pub fn interpret(&mut self) -> Result<(), VmError> {
        while self.vm.interpret_next(&mut self.context)? == InterpretResult::More {
            self.context.collect();
        }

        Ok(())
    }

    pub fn set_native_fn(&mut self, identifier: &str, code: fn(&[Value]) -> Value) {
        self.vm.set_native_fn(identifier, code, &mut self.context)
    }
}

/// Add the lox standard library to a Vm instance.
/// Right now the stdlib consists of 'clock'.
pub fn set_stdlib<W>(outer: &mut Vm<W>)
where
    W: Write,
{
    outer.set_native_fn("clock", |_args| {
        use std::time::{SystemTime, UNIX_EPOCH};

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        memory::Value::Number(time)
    });
}

pub fn execute(module: Module) -> Result<(), VmError> {
    let mut vm = Vm::with_stdout(module, stdout());
    set_stdlib(&mut vm);

    vm.interpret()
}

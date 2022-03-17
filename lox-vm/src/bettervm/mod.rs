mod memory;
pub mod vm;
mod interner;

use std::io::{Write, stdout};

use crate::bytecode::Module;

pub use vm::VmError;

use self::vm::VmOuter;

/// Add the lox standard library to a Vm instance.
/// Right now the stdlib consists of 'clock'.
pub fn set_stdlib<W>(outer: &mut VmOuter<W>)
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
    let mut vm = VmOuter::with_stdout(module, stdout());
    set_stdlib(&mut vm);

    vm.interpret()
}

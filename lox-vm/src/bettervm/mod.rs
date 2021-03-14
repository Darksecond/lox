mod memory;
pub mod vm;
use std::io::{Write};

use crate::bytecode::Module;
use vm::Vm;

pub use vm::VmError;

/// Add the lox standard library to a Vm instance.
/// Right now the stdlib consists of 'clock'.
pub fn set_stdlib<W>(vm: &mut Vm<W>)
where
    W: Write,
{
    vm.set_native_fn("clock", |_args| {
        use std::time::{SystemTime, UNIX_EPOCH};

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        memory::Value::Number(time)
    });
}

pub fn execute(module: &Module) -> Result<(), VmError> {
    let mut vm = Vm::new(module);
    set_stdlib(&mut vm);

    vm.interpret()
}

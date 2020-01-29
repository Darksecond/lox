mod memory;
mod vm;

use crate::bytecode::Module;
use vm::Vm;

pub use vm::VmError;

pub fn execute(module: &Module) -> Result<(), VmError>{
    let mut vm = Vm::new(module);

    //TODO Work on a way of initializing and executing the VM.
    //     It should be possible to define your own native functions.
    vm.set_native_fn("clock", |_args| {
        use std::time::{UNIX_EPOCH, SystemTime};

        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();
        memory::Value::Number(time)
    });

    vm.interpret()?;

    Ok(())
}
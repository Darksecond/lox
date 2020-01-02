mod memory;
mod vm;

use crate::bytecode::Module;
use vm::Vm;

pub use vm::VmError;

pub fn execute(module: &Module) -> Result<(), VmError>{
    let mut vm = Vm::new(module);

    //TODO Make it possible to supply these to the execute somehow...
    //     Either with a Vec of them, or by splitting this up in two parts or something
    vm.set_native_fn("test", |args| {
        println!("test {:?}", args);
        memory::Value::Nil
    });

    vm.set_native_fn("clock", |_args| {
        use std::time::{UNIX_EPOCH, SystemTime};

        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();
        memory::Value::Number(time)
    });

    vm.interpret()?;

    Ok(())
}
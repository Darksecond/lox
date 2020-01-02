mod memory;
mod vm;

use crate::bytecode::Module;
use vm::Vm;

pub fn execute(module: &Module) {
    let mut vm = Vm::new(module);
    vm.interpret();
}
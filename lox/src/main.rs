use std::env;

use lox_compiler::LineOffsets;
use lox_vm::VirtualMachine;
use lox_std::set_stdlib;
use lox_bytecode::bytecode::Module;

#[cfg(test)]
mod tests;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 {
        eprintln!("Usage: lox [path]");
        return;
    }

    let path = args.first().unwrap();
    let data = std::fs::read_to_string(path).unwrap();
    let offsets = LineOffsets::new(&data);

    let module = match lox_compiler::compile(&data) {
        Ok(module) => module,
        Err(diagnostics) => {
            for diag in diagnostics {
                let line = offsets.line(diag.span.start);
                let msg = diag.message;
                eprintln!("Error: {msg} at line {line}");
            }
            return;
        },
    };

    // Run virtual machine
    let mut vm = VirtualMachine::new();
    set_stdlib(&mut vm);
    vm.set_import(import);
    vm.interpret(module).unwrap();
}

fn import(path: &str) -> Option<Module> {
    let data = std::fs::read_to_string(format!("{}.lox", path)).unwrap();
    let offsets = LineOffsets::new(&data);

    match lox_compiler::compile(&data) {
        Ok(module) => Some(module),
        Err(diagnostics) => {
            for diag in diagnostics {
                let line = offsets.line(diag.span.start);
                let msg = diag.message;
                eprintln!("Error: {msg} at line {line}");
            }

            None
        },
    }
}

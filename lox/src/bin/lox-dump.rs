use std::env;
use lox_compiler::LineOffsets;

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

    lox_bytecode::disasm::disassemble_module(&module);
}

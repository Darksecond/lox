use std::env;

use lox_compiler::LineOffsets;

#[cfg(test)]
mod tests;

// fn main_old() {
//     let data = std::fs::read_to_string("test.lox").unwrap();

//     let module = lox_compiler::compile(&data).unwrap();

//     // Temporary to test serde out
//     let data = serde_json::to_string_pretty(&module).unwrap();
//     println!("{}", data);
//     let module: lox_bytecode::bytecode::Module = serde_json::from_str(&data).unwrap();

//     println!("constants: {:?}", module.constants());
//     for chunk in module.chunks() {
//         println!("chunk: {:?}", chunk.instructions());
//     }

//     println!();

//     lox_vm::bettervm::execute(&module).unwrap();
// }

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

    lox_vm::bettervm::execute(module, import).unwrap();
}

use lox_bytecode::bytecode::Module;
fn import(path: &str) -> Module {
    let data = std::fs::read_to_string(format!("{}.lox", path)).unwrap();
    let offsets = LineOffsets::new(&data);

    let module = match lox_compiler::compile(&data) {
        Ok(module) => module,
        Err(diagnostics) => {
            for diag in diagnostics {
                let line = offsets.line(diag.span.start);
                let msg = diag.message;
                eprintln!("Error: {msg} at line {line}");
            }
            panic!();
        },
    };

    module
}

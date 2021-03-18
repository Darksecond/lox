use std::env;

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

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 {
        Err("Usage: lox [path]".into())
    } else {
        let path = args.first().unwrap();

        let data = std::fs::read_to_string(path).unwrap();
        let module = lox_compiler::compile(&data).unwrap();
        lox_vm::bettervm::execute(module).unwrap();


        Ok(())
    }
}
fn main() {
    let data = std::fs::read_to_string("test.lox").unwrap();

    let module = lox_compiler::compile(&data).unwrap();

    // Temporary to test serde out
    let data = serde_json::to_string_pretty(&module).unwrap();
    println!("{}", data);
    let module: lox_bytecode::bytecode::Module = serde_json::from_str(&data).unwrap();

    println!("constants: {:?}", module.constants());
    for chunk in module.chunks() {
        println!("chunk: {:?}", chunk.instructions());
    }

    println!();

    lox_vm::bettervm::execute(&module).unwrap();
}

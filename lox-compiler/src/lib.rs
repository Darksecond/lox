mod bettercompiler;

use lox_bytecode::bytecode;
pub use lox_syntax::position::Diagnostic;
pub use lox_syntax::position::LineOffsets;

//TODO Better errors

use bytecode::Module;
pub fn compile(code: &str) -> Result<Module, Vec<Diagnostic>> {
    let ast = lox_syntax::parse(code)?;
    // println!("AST: {:?}", ast);
    let module = bettercompiler::compile(&ast)?;

    Ok(module)
}

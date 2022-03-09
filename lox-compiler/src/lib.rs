mod bettercompiler;

use lox_bytecode::bytecode;
use lox_syntax::position::Diagnostic;

//TODO Better errors

use bytecode::Module;
pub fn compile(code: &str) -> Result<Module, Vec<Diagnostic>> {
    let ast = lox_syntax::parse(code)?;
    // println!("AST: {:?}", ast);
    let module = bettercompiler::compile(&ast)?;

    Ok(module)
}

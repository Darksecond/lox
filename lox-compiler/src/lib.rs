mod bettercompiler;

use lox_bytecode::bytecode;
use lox_syntax::position::Diagnostic;

//TODO Better errors

pub use crate::bettercompiler::CompilerError;

#[derive(Debug)]
pub enum Error {
    CompileError(Vec<CompilerError>),
    ParseError(Vec<Diagnostic>),
}

use bytecode::Module;
pub fn compile(code: &str) -> Result<Module, Error> {
    let ast = lox_syntax::parse(code).map_err(|e| Error::ParseError(e))?;
    // println!("AST: {:?}", ast);
    let module = bettercompiler::compile(&ast).map_err(|e| Error::CompileError(e))?;

    Ok(module)
}

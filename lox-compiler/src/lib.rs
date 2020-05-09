mod bettercompiler;

use lox_bytecode::bytecode;

//TODO Better errors

pub use crate::{bettercompiler::CompilerError};
pub use lox_syntax::SyntaxError;

#[derive(Debug)]
pub enum Error {
    CompileError(CompilerError),
    ParseError(SyntaxError),
}

use bytecode::Module;
pub fn compile(code: &str) -> Result<Module, Error> {
    let ast = lox_syntax::parse(code).map_err(|e| Error::ParseError(e))?;
    let module = bettercompiler::compile(&ast).map_err(|e| Error::CompileError(e))?;
 
    Ok(module)
}
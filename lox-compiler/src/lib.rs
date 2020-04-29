mod bettercompiler;

use lox_bytecode::bytecode;

//TODO Better errors

pub use crate::{bettercompiler::CompilerError};
pub use lox_syntax::common::ParseError;

#[derive(Debug)]
pub enum Error {
    CompileError(CompilerError),
    ParseError(ParseError),
}

use bytecode::Module;
pub fn compile(code: &str) -> Result<Module, Error> {
    use lox_syntax::{tokenizer::tokenize_with_context, stmt_parser::parse};
    use crate::bettercompiler::compile;
    
    let tokens = tokenize_with_context(code);
    let mut it = tokens.as_slice().into_iter().peekable();

    let ast = parse(&mut it).map_err(|e| Error::ParseError(e))?;
    let module = compile(&ast).map_err(|e| Error::CompileError(e))?;
 
    Ok(module)
}
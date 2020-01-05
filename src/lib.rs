mod tokenizer;
#[macro_use]
mod common;
mod ast;
mod bytecode;
mod expr_parser;
mod stmt_parser;
mod token;
mod bettergc;
mod bettercompiler;
mod position;
pub mod bettervm;

//TODO Better errors

pub use crate::{bettercompiler::CompilerError, bettervm::VmError, common::ParseError};

#[derive(Debug)]
pub enum Error {
    CompileError(CompilerError),
    ParseError(ParseError),
    //TODO RuntimeError(VmError),
}

use bytecode::Module;
pub fn compile(code: &str) -> Result<Module, Error> {
    use crate::{tokenizer::tokenize_with_context, stmt_parser::parse, bettercompiler::compile};
    let tokens = tokenize_with_context(code);
    let mut it = tokens.as_slice().into_iter().peekable();

    let ast = parse(&mut it).map_err(|e| Error::ParseError(e))?;
    let module = compile(&ast).map_err(|e| Error::CompileError(e))?;
 
    Ok(module)
}
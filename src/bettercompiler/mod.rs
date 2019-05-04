mod compiler;
mod statements;

#[cfg(test)]
mod tests;

use crate::bytecode::*;
use crate::ast::*;
use compiler::{Compiler, ContextType};
use statements::compile_ast;

#[derive(Debug)]
pub enum CompilerError {
    //TODO
    UnpatchableInstruction(Instruction),
    NoContext,
    LocalAlreadyDefined(String),
    LocalNotInitialized(String),
    Multiple(Vec<CompilerError>),
}

pub fn compile(ast: &Ast) -> Result<Module, CompilerError> {
    let mut compiler = Compiler::new();

    compiler.with_context(ContextType::TopLevel, |compiler| {
        compile_ast(compiler, ast)
    })?;

    Ok(compiler.into_module())
}
mod compiler;
mod statements;
mod locals;

#[cfg(test)]
mod tests;

use crate::bytecode::*;
use crate::ast::*;
use compiler::{Compiler, ContextType};
use statements::compile_ast;
use crate::position::WithSpan;

#[derive(Debug)]
pub enum CompilerError {
    LocalAlreadyDefined,
    LocalNotInitialized,

    Multiple(Vec<CompilerError>),
    WithSpan(WithSpan<Box<CompilerError>>),
}

pub fn compile(ast: &Ast) -> Result<Module, CompilerError> {
    let mut compiler = Compiler::new();

    compiler.with_context(ContextType::TopLevel, |compiler| {
        compile_ast(compiler, ast)?;
        compiler.add_instruction(Instruction::Nil);
        compiler.add_instruction(Instruction::Return);
        Ok(())
    })?;

    Ok(compiler.into_module())
}
mod compiler;
mod locals;
mod statements;

#[cfg(test)]
mod tests;

use crate::bytecode::*;
use compiler::{Compiler, ContextType};
use lox_syntax::ast::*;
use lox_syntax::position::WithSpan;
use statements::compile_ast;

#[derive(Debug)]
pub enum CompilerError {
    LocalAlreadyDefined,
    LocalNotInitialized,
    InvalidThis,
    InvalidReturn,

    Multiple(Vec<CompilerError>),
    WithSpan(WithSpan<Box<CompilerError>>),
}

pub fn compile(ast: &Ast) -> Result<Module, Vec<CompilerError>> {
    let mut compiler = Compiler::new();

    let _ = compiler.with_context(ContextType::TopLevel, |compiler| {
        compile_ast(compiler, ast);
        compiler.add_instruction(Instruction::Nil);
        compiler.add_instruction(Instruction::Return);
    });

    if compiler.has_errors() {
        Err(compiler.into_errors())
    } else {
        Ok(compiler.into_module())
    }
}

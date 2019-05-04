use super::{CompilerError};
use crate::ast::*;
use crate::bytecode::*;
use super::compiler::Compiler;

pub fn compile_ast(compiler: &mut Compiler, ast: &Ast) -> Result<(), CompilerError> {
    let errors: Vec<_> = ast.iter()
        .map(|stmt| compile_stmt(compiler, stmt))
        .filter_map(Result::err)
        .collect();
    if errors.is_empty() { Ok(()) } else { Err(CompilerError::Multiple(errors)) }
}

fn compile_stmt(compiler: &mut Compiler, stmt: &Stmt) -> Result<(), CompilerError> {
    match stmt {
        Stmt::Print(ref expr) => compile_print(compiler, expr),
        _ => unimplemented!(),
    }
}

fn compile_print(compiler: &mut Compiler, expr: &Expr) -> Result<(), CompilerError> {
    compile_expr(compiler, expr)?;
    compiler.add_instruction(Instruction::Print)?;
    Ok(())
}

fn compile_expr(compiler: &mut Compiler, expr: &Expr) -> Result<(), CompilerError> {
    match *expr {
        Expr::Number(num) => compile_number(compiler, num),
        Expr::String(ref string) => compile_string(compiler, string),
        Expr::Binary(ref left, operator, ref right) => compile_binary(compiler, operator, left, right),
        _ => unimplemented!(),
    }
}

fn compile_number(compiler: &mut Compiler, num: f64) -> Result<(), CompilerError> {
    let constant = compiler.add_constant(num);
    compiler.add_instruction(Instruction::Constant(constant))?;
    Ok(())
}

fn compile_string(compiler: &mut Compiler, string: &str) -> Result<(), CompilerError> {
    let constant = compiler.add_constant(string);
    compiler.add_instruction(Instruction::Constant(constant))?;
    Ok(())
}

fn compile_binary(compiler: &mut Compiler, operator: BinaryOperator, left: &Expr, right: &Expr) -> Result<(), CompilerError> {
    compile_expr(compiler, left)?;
    compile_expr(compiler, right)?;
    match operator {
        BinaryOperator::Plus => compiler.add_instruction(Instruction::Add)?,
        BinaryOperator::Minus => compiler.add_instruction(Instruction::Subtract)?,
        _ => unimplemented!(),
    };
    Ok(())
}
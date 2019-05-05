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
        Stmt::Var(ref identifier, ref expr) => compile_var_declaration(compiler, identifier, expr.as_ref()),
        Stmt::Block(ref stmts) => compile_block(compiler, stmts),
        Stmt::Expression(ref expr) => compile_expression_statement(compiler, expr),
        _ => unimplemented!(),
    }
}

fn compile_expression_statement(compiler: &mut Compiler, expr: &Expr) -> Result<(), CompilerError> {
    compile_expr(compiler, expr)?;
    compiler.add_instruction(Instruction::Pop)?;
    Ok(())
}

fn compile_block(compiler: &mut Compiler, ast: &Ast) -> Result<(), CompilerError> {
    compiler.with_scope(|compiler| {
        compile_ast(compiler, ast)
    })
}

fn compile_var_declaration<T: AsRef<Expr>>(compiler: &mut Compiler, identifier: &str, expr: Option<T>) -> Result<(), CompilerError> {
    //declare
    if compiler.is_scoped() {
        compiler.add_local(identifier)?;
    }
    
    //expr
    if let Some(expr) = expr {
        compile_expr(compiler, expr.as_ref())?;
    } else {
        compile_nil(compiler)?;
    }

    //define
    if compiler.is_scoped() {
        compiler.mark_local_initialized()?;
    } else {
        let constant = compiler.add_constant(identifier);
        compiler.add_instruction(Instruction::DefineGlobal(constant))?;
    }

    Ok(())
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
        Expr::Variable(ref identifier) => compile_variable(compiler, identifier),
        Expr::Nil => compile_nil(compiler),
        Expr::Assign(ref identifier, ref expr) => compile_assign(compiler, identifier, expr),
        _ => unimplemented!(),
    }
}

fn compile_assign(compiler: &mut Compiler, identifier: &str, expr: &Expr) -> Result<(), CompilerError> {
    compile_expr(compiler, expr)?;
    if let Some(local) = compiler.resolve_local(identifier)? {
        // Local
        compiler.add_instruction(Instruction::SetLocal(local))?;
    } else {
        // Global
        let constant = compiler.add_constant(identifier);
        compiler.add_instruction(Instruction::SetGlobal(constant))?;
    }
    Ok(())
}

fn compile_variable(compiler: &mut Compiler, identifier: &str) -> Result<(), CompilerError> {
    if let Some(local) = compiler.resolve_local(identifier)? {
        // Local
        compiler.add_instruction(Instruction::GetLocal(local))?;
    } else {
        // Global
        let constant = compiler.add_constant(identifier);
        compiler.add_instruction(Instruction::GetGlobal(constant))?;
    }
    Ok(())
}

fn compile_nil(compiler: &mut Compiler) -> Result<(), CompilerError> {
    compiler.add_instruction(Instruction::Nil)?;
    Ok(())
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
use super::compiler::Compiler;
use super::compiler::ContextType;
use crate::bytecode::*;
use lox_syntax::ast::*;
use lox_syntax::position::WithSpan;
use lox_bytecode::opcode;

pub fn compile_ast(compiler: &mut Compiler, ast: &Ast) {
    for stmt in ast {
        compile_stmt(compiler, stmt);
    }
}

fn compile_stmt(compiler: &mut Compiler, stmt: &WithSpan<Stmt>) {
    match &stmt.value {
        Stmt::Print(ref expr) => compile_print(compiler, expr),
        Stmt::Var(ref identifier, ref expr) => {
            compile_var_declaration(compiler, identifier.as_ref(), expr.as_ref())
        }
        Stmt::Block(ref stmts) => compile_block(compiler, stmts),
        Stmt::Expression(ref expr) => compile_expression_statement(compiler, expr),
        Stmt::If(ref condition, ref then_stmt, ref else_stmt) => {
            compile_if(compiler, condition, then_stmt, else_stmt.as_ref())
        }
        Stmt::While(ref expr, ref stmt) => compile_while(compiler, expr, stmt),
        Stmt::Function(ref identifier, ref args, ref stmts) => {
            compile_function(compiler, &identifier.as_ref(), args, stmts)
        }
        Stmt::Return(ref expr) => {
            if compiler.context_type() == ContextType::Initializer && expr.is_some() { // Do Allow for early returns from init
                compiler.add_error("Invalid return", stmt.span);
                return;
            }
            if compiler.context_type() == ContextType::TopLevel && expr.is_some() {
                compiler.add_error("Invalid return", stmt.span);
                return;
            }
            compile_return(compiler, expr.as_ref())
        },
        Stmt::Class(ref identifier, ref extends, ref stmts) => {
            compile_class(compiler, identifier.as_ref(), extends.as_ref(), stmts)
        }
        Stmt::Import(path, identifiers) => compile_import(compiler, path, identifiers.as_ref()),
    }
}

fn declare_variable<I: AsRef<str>>(compiler: &mut Compiler, identifier: WithSpan<I>) {
    if compiler.is_scoped() {
        if compiler.has_local_in_current_scope(identifier.value.as_ref()) {
            compiler.add_error("Local already defined", identifier.span);
            return;
        }

        compiler.add_local(identifier.value.as_ref());
    }
}

fn define_variable(compiler: &mut Compiler, identifier: &str) {
    if compiler.is_scoped() {
        compiler.mark_local_initialized();
    } else {
        let constant = compiler.add_identifier(identifier);
        compiler.add_u8(opcode::DEFINE_GLOBAL);
        compiler.add_u32(constant as _);
    }
}

fn compile_import(compiler: &mut Compiler, path: &WithSpan<String>, identifiers: Option<&Vec<WithSpan<String>>>) {
    let constant = compiler.add_constant(path.value.as_str());
    compiler.add_u8(opcode::IMPORT);
    compiler.add_u32(constant as _);

    if let Some(identifiers) = identifiers {
        for identifier in identifiers {
            declare_variable(compiler, identifier.as_ref());

            let constant = compiler.add_identifier(identifier.value.as_str());
            compiler.add_u8(opcode::IMPORT_GLOBAL);
            compiler.add_u32(constant as _);

            define_variable(compiler, &identifier.value);
        }
    }

    compiler.add_u8(opcode::POP);
}

fn compile_class(
    compiler: &mut Compiler,
    identifier: WithSpan<&String>,
    _extends: Option<&WithSpan<String>>,
    stmts: &[WithSpan<Stmt>],
) {
    declare_variable(compiler, identifier.as_ref());
    let constant = compiler.add_class(Class{
        name: identifier.value.to_string()
    });
    compiler.add_u8(opcode::CLASS);
    compiler.add_u8(constant as _);
    define_variable(compiler, identifier.value);

    compile_variable(compiler, identifier);

    //TODO Extends

    // Methods
    for stmt in stmts {
        match &stmt.value {
            Stmt::Function(identifier, args, block) => {
                compile_method(compiler, identifier.as_ref(), args, block);
            },
            _ => unimplemented!(), //TODO
        }
    }

    compiler.add_u8(opcode::POP);
}

fn compile_method(
    compiler: &mut Compiler,
    identifier: WithSpan<&String>,
    args: &Vec<WithSpan<Identifier>>,
    block: &Vec<WithSpan<Stmt>>,
) {

    let context_type = if identifier.value == "init" { ContextType::Initializer } else { ContextType::Method };

    compile_closure(compiler, &identifier, args, block, context_type);

    let constant = compiler.add_identifier(identifier.value.as_str());
    compiler.add_u8(opcode::METHOD);
    compiler.add_u32(constant as _);
}

fn compile_return<E: AsRef<WithSpan<Expr>>>(
    compiler: &mut Compiler,
    expr: Option<E>,
) {
    if let Some(expr) = expr {
        compile_expr(compiler, expr.as_ref());
    } else if compiler.context_type() == ContextType::Initializer {
        compiler.add_u8(opcode::GET_LOCAL);
        compiler.add_u32(0);
    } else {
        compile_nil(compiler);
    }
    compiler.add_u8(opcode::RETURN);
}

fn compile_closure(
    compiler: &mut Compiler, 
    identifier: &WithSpan<&String>, 
    args: &Vec<WithSpan<Identifier>>, 
    block: &Vec<WithSpan<Stmt>>,
    context_type: ContextType
) {
    let (chunk_index, upvalues) =
    compiler.with_scoped_context(context_type, |compiler| {
        for arg in args {
            declare_variable(compiler, arg.as_ref());
            define_variable(compiler, &arg.value);
        }

        compile_ast(compiler, block);

        {
            let expr: Option<Box<WithSpan<Expr>>> = None;
            compile_return(compiler, expr.as_ref());
        }
    });

    let function = Function {
        name: identifier.value.into(),
        chunk_index,
        arity: args.len(),
    };

    let closure = Closure {
        function,
        upvalues,
    };

    let constant = compiler.add_closure(closure);
    compiler.add_u8(opcode::CLOSURE);
    compiler.add_u32(constant as _);
}

fn compile_function(
    compiler: &mut Compiler,
    identifier: &WithSpan<&String>,
    args: &Vec<WithSpan<Identifier>>,
    block: &Vec<WithSpan<Stmt>>,
) {
    declare_variable(compiler, identifier.as_ref());
    if compiler.is_scoped() {
        compiler.mark_local_initialized();
    }

    compile_closure(compiler, identifier, args, block, ContextType::Function);

    define_variable(compiler, identifier.value);
}

fn compile_while(
    compiler: &mut Compiler,
    condition: &WithSpan<Expr>,
    body: &WithSpan<Stmt>,
) {
    let loop_start = compiler.instruction_index();
    compile_expr(compiler, condition);
    compiler.add_u8(opcode::JUMP_IF_FALSE);
    let end_jump = compiler.add_i16(0);
    compiler.add_u8(opcode::POP);
    compile_stmt(compiler, body);
    compiler.add_u8(opcode::JUMP);
    let loop_jump = compiler.add_i16(0);
    compiler.patch_instruction_to(loop_jump, loop_start);
    compiler.patch_instruction(end_jump);
    compiler.add_u8(opcode::POP);
}

fn compile_if<S: AsRef<WithSpan<Stmt>>>(
    compiler: &mut Compiler,
    condition: &WithSpan<Expr>,
    then_stmt: &WithSpan<Stmt>,
    else_stmt: Option<S>,
) {
    compile_expr(compiler, condition);

    compiler.add_u8(opcode::JUMP_IF_FALSE);
    let then_index = compiler.add_i16(0);
    compiler.add_u8(opcode::POP);
    compile_stmt(compiler, then_stmt);

    if let Some(else_stmt) = else_stmt {
        compiler.add_u8(opcode::JUMP);
        let else_index = compiler.add_i16(0);
        compiler.patch_instruction(then_index);
        compiler.add_u8(opcode::POP);
        compile_stmt(compiler, else_stmt.as_ref());
        compiler.patch_instruction(else_index);
    } else {
        compiler.patch_instruction(then_index);
    }
}

fn compile_expression_statement(compiler: &mut Compiler, expr: &WithSpan<Expr>) {
    compile_expr(compiler, expr);
    compiler.add_u8(opcode::POP);
}

fn compile_block(compiler: &mut Compiler, ast: &Ast) {
    compiler.with_scope(|compiler| compile_ast(compiler, ast))
}

fn compile_var_declaration<T: AsRef<WithSpan<Expr>>, I: AsRef<str>>(
    compiler: &mut Compiler,
    identifier: WithSpan<I>,
    expr: Option<T>,
) {
    declare_variable(compiler, identifier.as_ref());

    //expr
    if let Some(expr) = expr {
        compile_expr(compiler, expr.as_ref());
    } else {
        compile_nil(compiler);
    }

    define_variable(compiler, identifier.value.as_ref());
}

fn compile_print(compiler: &mut Compiler, expr: &WithSpan<Expr>) {
    compile_expr(compiler, expr);
    compiler.add_u8(opcode::PRINT);
}

fn compile_expr(compiler: &mut Compiler, expr: &WithSpan<Expr>) {
    match expr.value {
        Expr::Number(num) => compile_number(compiler, num),
        Expr::String(ref string) => compile_string(compiler, string),
        Expr::Binary(ref left, ref operator, ref right) => {
            compile_binary(compiler, operator, left, right)
        }
        Expr::Variable(ref identifier) => compile_variable(compiler, identifier.as_ref()),
        Expr::Nil => compile_nil(compiler),
        Expr::Boolean(boolean) => compile_boolean(compiler, boolean),
        Expr::Assign(ref identifier, ref expr) => {
            compile_assign(compiler, identifier.as_ref(), expr)
        }
        Expr::Logical(ref left, ref operator, ref right) => {
            compile_logical(compiler, operator, left, right)
        }
        Expr::Call(ref identifier, ref args) => compile_call(compiler, identifier, args),
        Expr::Grouping(ref expr) => compile_expr(compiler, expr),
        Expr::Unary(ref operator, ref expr) => compile_unary(compiler, operator.clone(), expr),
        Expr::Set(ref expr, ref identifier, ref value) => {
            compile_set(compiler, expr, identifier.as_ref(), value)
        }
        Expr::Get(ref expr, ref identifier) => compile_get(compiler, expr, identifier.as_ref()),
        Expr::This => compile_this(compiler, expr),
        Expr::List(ref expr) => compile_list(compiler, expr),
        Expr::ListGet(ref list, ref expr) => compile_list_get(compiler, list, expr),
        Expr::ListSet(ref list, ref index, ref value) => compile_list_set(compiler, list, index, value),
        ref expr => unimplemented!("{:?}", expr),
    }
}

fn compile_list_set(compiler: &mut Compiler, list: &WithSpan<Expr>, index: &WithSpan<Expr>, value: &WithSpan<Expr>) {
        compile_expr(compiler, list);
        compile_expr(compiler, index);
        compile_expr(compiler, value);
        compiler.add_u8(opcode::SET_INDEX);
}

fn compile_list_get(compiler: &mut Compiler, list: &WithSpan<Expr>, expr: &WithSpan<Expr>) {
        compile_expr(compiler, list);
        compile_expr(compiler, expr);
        compiler.add_u8(opcode::GET_INDEX);
}

fn compile_list(compiler: &mut Compiler, expr: &Vec<WithSpan<Expr>>) {
    for expr in expr {
        compile_expr(compiler, expr);
    }

    compiler.add_u8(opcode::LIST);
    compiler.add_u8(expr.len() as _);
}

fn compile_this(compiler: &mut Compiler, expr: &WithSpan<Expr>) {
    if !compiler.in_method_or_initializer_nested() {
        compiler.add_error("Invalid 'this'", expr.span);
        return;
    }

    compile_variable(compiler, WithSpan::new(&"this".to_string(), expr.span))
}

fn compile_get(
    compiler: &mut Compiler,
    expr: &WithSpan<Expr>,
    identifier: WithSpan<&String>,
) {
    compile_expr(compiler, expr);
    let constant = compiler.add_identifier(identifier.value.as_str());
    compiler.add_u8(opcode::GET_PROPERTY);
    compiler.add_u32(constant as _);
}

fn compile_set(
    compiler: &mut Compiler,
    expr: &WithSpan<Expr>,
    identifier: WithSpan<&String>,
    value: &WithSpan<Expr>,
) {
    compile_expr(compiler, expr);
    compile_expr(compiler, value);
    let constant = compiler.add_identifier(identifier.value.as_str());
    compiler.add_u8(opcode::SET_PROPERTY);
    compiler.add_u32(constant as _);
}

fn compile_unary(
    compiler: &mut Compiler,
    operator: WithSpan<UnaryOperator>,
    expr: &WithSpan<Expr>,
) {
    compile_expr(compiler, expr);
    match operator.value {
        UnaryOperator::Minus => compiler.add_u8(opcode::NEGATE),
        UnaryOperator::Bang => compiler.add_u8(opcode::NOT),
    };
}

fn compile_call(
    compiler: &mut Compiler,
    identifier: &WithSpan<Expr>,
    args: &Vec<WithSpan<Expr>>,
) {
    if let Expr::Get(expr, ident) = &identifier.value {
        compile_expr(compiler, expr);

        for arg in args {
            compile_expr(compiler, arg);
        }

        let constant = compiler.add_identifier(ident.value.as_str());
        compiler.add_u8(opcode::INVOKE);
        compiler.add_u8(args.len() as _);
        compiler.add_u32(constant as _);
    } else {
        compile_expr(compiler, identifier);

        for arg in args {
            compile_expr(compiler, arg);
        }

        compiler.add_u8(opcode::CALL);
        compiler.add_u8(args.len() as _);
    }
}

fn compile_logical(
    compiler: &mut Compiler,
    operator: &WithSpan<LogicalOperator>,
    left: &WithSpan<Expr>,
    right: &WithSpan<Expr>,
) {
    match operator.value {
        LogicalOperator::And => compile_logical_and(compiler, left, right),
        LogicalOperator::Or => compile_logical_or(compiler, left, right),
    }
}

//TODO Implement this better, using one less jump, we can easily introduce a JumpIfTrue instruction.
fn compile_logical_or(
    compiler: &mut Compiler,
    left: &WithSpan<Expr>,
    right: &WithSpan<Expr>,
) {
    compile_expr(compiler, left);
    compiler.add_u8(opcode::JUMP_IF_FALSE);
    let else_jump = compiler.add_i16(0);
    compiler.add_u8(opcode::JUMP);
    let end_jump = compiler.add_i16(0);
    compiler.patch_instruction(else_jump);
    compiler.add_u8(opcode::POP);
    compile_expr(compiler, right);
    compiler.patch_instruction(end_jump);
}

fn compile_logical_and(
    compiler: &mut Compiler,
    left: &WithSpan<Expr>,
    right: &WithSpan<Expr>,
) {
    compile_expr(compiler, left);
    compiler.add_u8(opcode::JUMP_IF_FALSE);
    let end_jump = compiler.add_i16(0);
    compiler.add_u8(opcode::POP);
    compile_expr(compiler, right);
    compiler.patch_instruction(end_jump);
}

fn compile_boolean(compiler: &mut Compiler, boolean: bool) {
    if boolean {
        compiler.add_u8(opcode::TRUE);
    } else {
        compiler.add_u8(opcode::FALSE);
    }
}

fn compile_assign(
    compiler: &mut Compiler,
    identifier: WithSpan<&String>,
    expr: &WithSpan<Expr>,
) {
    compile_expr(compiler, expr);
    if let Some(local) = compiler.resolve_local(identifier.value) {
        // Local
        compiler.add_u8(opcode::SET_LOCAL);
        compiler.add_u32(local as _);
    } else if let Some(upvalue) = compiler.resolve_upvalue(identifier.value) {
        // Upvalue
        compiler.add_u8(opcode::SET_UPVALUE);
        compiler.add_u32(upvalue as _);
    } else {
        // Global
        let constant = compiler.add_identifier(identifier.value.as_str());
        compiler.add_u8(opcode::SET_GLOBAL);
        compiler.add_u32(constant as _);
    }
}

fn compile_variable(
    compiler: &mut Compiler,
    identifier: WithSpan<&String>,
) {
    if let Some(local) = compiler.resolve_local(identifier.value) {
        // Local
        compiler.add_u8(opcode::GET_LOCAL);
        compiler.add_u32(local as _);
    } else if let Some(upvalue) = compiler.resolve_upvalue(identifier.value) {
        // Upvalue
        compiler.add_u8(opcode::GET_UPVALUE);
        compiler.add_u32(upvalue as _);
    } else {
        // Global
        let constant = compiler.add_identifier(identifier.value.as_str());
        compiler.add_u8(opcode::GET_GLOBAL);
        compiler.add_u32(constant as _);
    }
}

fn compile_nil(compiler: &mut Compiler) {
    compiler.add_u8(opcode::NIL);
}

fn compile_number(compiler: &mut Compiler, num: f64) {
    let constant = compiler.add_constant(num);
    compiler.add_u8(opcode::CONSTANT);
    compiler.add_u32(constant as _);
}

fn compile_string(compiler: &mut Compiler, string: &str) {
    let constant = compiler.add_constant(string);
    compiler.add_u8(opcode::CONSTANT);
    compiler.add_u32(constant as _);
}

fn compile_binary(
    compiler: &mut Compiler,
    operator: &WithSpan<BinaryOperator>,
    left: &WithSpan<Expr>,
    right: &WithSpan<Expr>,
) {
    compile_expr(compiler, left);
    compile_expr(compiler, right);
    match operator.value {
        BinaryOperator::Plus => compiler.add_u8(opcode::ADD),
        BinaryOperator::Minus => compiler.add_u8(opcode::SUBTRACT),
        BinaryOperator::Less => compiler.add_u8(opcode::LESS),
        BinaryOperator::LessEqual => {
            compiler.add_u8(opcode::GREATER);
            compiler.add_u8(opcode::NOT)
        }
        BinaryOperator::Star => compiler.add_u8(opcode::MULTIPLY),
        BinaryOperator::EqualEqual => compiler.add_u8(opcode::EQUAL),
        BinaryOperator::BangEqual => {
            compiler.add_u8(opcode::EQUAL);
            compiler.add_u8(opcode::NOT)
        }
        BinaryOperator::Greater => compiler.add_u8(opcode::GREATER),
        BinaryOperator::GreaterEqual => {
            compiler.add_u8(opcode::LESS);
            compiler.add_u8(opcode::NOT)
        }
        BinaryOperator::Slash => compiler.add_u8(opcode::DIVIDE),
    };
}

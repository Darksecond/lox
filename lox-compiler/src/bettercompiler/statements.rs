use super::compiler::Compiler;
use super::compiler::ContextType;
use crate::bytecode::*;
use lox_syntax::ast::*;
use lox_syntax::position::WithSpan;

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
        let constant = compiler.add_constant(identifier);
        compiler.add_instruction(Instruction::DefineGlobal(constant));
    }
}

fn compile_import(compiler: &mut Compiler, path: &WithSpan<String>, identifiers: Option<&Vec<WithSpan<String>>>) {
    let constant = compiler.add_constant(path.value.as_str());
    compiler.add_instruction(Instruction::Import(constant));

    if let Some(identifiers) = identifiers {
        for identifier in identifiers {
            declare_variable(compiler, identifier.as_ref());
            
            let constant = compiler.add_constant(identifier.value.as_str());
            compiler.add_instruction(Instruction::ImportGlobal(constant));

            define_variable(compiler, &identifier.value);
        }
    }

    compiler.add_instruction(Instruction::Pop);
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
    compiler.add_instruction(Instruction::Class(constant));
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

    compiler.add_instruction(Instruction::Pop);
}

fn compile_method(
    compiler: &mut Compiler,
    identifier: WithSpan<&String>,
    args: &Vec<WithSpan<Identifier>>,
    block: &Vec<WithSpan<Stmt>>,
) {

    let context_type = if identifier.value == "init" { ContextType::Initializer } else { ContextType::Method };

    compile_closure(compiler, &identifier, args, block, context_type);

    let constant = compiler.add_constant(identifier.value.as_str());
    compiler.add_instruction(Instruction::Method(constant));
}

fn compile_return<E: AsRef<WithSpan<Expr>>>(
    compiler: &mut Compiler,
    expr: Option<E>,
) {
    if let Some(expr) = expr {
        compile_expr(compiler, expr.as_ref());
    } else if compiler.context_type() == ContextType::Initializer {
        compiler.add_instruction(Instruction::GetLocal(0)); //TODO Replace with proper 'variable' (compile_variable function)
    } else {
        compile_nil(compiler);
    }
    compiler.add_instruction(Instruction::Return);
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
    compiler.add_instruction(Instruction::Closure(constant));
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
    let end_jump = compiler.add_instruction(Instruction::JumpIfFalse(0));
    compiler.add_instruction(Instruction::Pop);
    compile_stmt(compiler, body);
    let loop_jump = compiler.add_instruction(Instruction::Jump(0));
    compiler.patch_instruction_to(loop_jump, loop_start);
    compiler.patch_instruction(end_jump);
    compiler.add_instruction(Instruction::Pop);
}

fn compile_if<S: AsRef<WithSpan<Stmt>>>(
    compiler: &mut Compiler,
    condition: &WithSpan<Expr>,
    then_stmt: &WithSpan<Stmt>,
    else_stmt: Option<S>,
) {
    compile_expr(compiler, condition);

    let then_index = compiler.add_instruction(Instruction::JumpIfFalse(0));
    compiler.add_instruction(Instruction::Pop);
    compile_stmt(compiler, then_stmt);

    if let Some(else_stmt) = else_stmt {
        let else_index = compiler.add_instruction(Instruction::Jump(0));
        compiler.patch_instruction(then_index);
        compiler.add_instruction(Instruction::Pop);
        compile_stmt(compiler, else_stmt.as_ref());
        compiler.patch_instruction(else_index);
    } else {
        compiler.patch_instruction(then_index);
    }
}

fn compile_expression_statement(compiler: &mut Compiler, expr: &WithSpan<Expr>) {
    compile_expr(compiler, expr);
    compiler.add_instruction(Instruction::Pop);
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
    compiler.add_instruction(Instruction::Print);
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
        ref expr => unimplemented!("{:?}", expr),
    }
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
    let constant = compiler.add_constant(identifier.value.as_str());
    compiler.add_instruction(Instruction::GetProperty(constant));
}

fn compile_set(
    compiler: &mut Compiler,
    expr: &WithSpan<Expr>,
    identifier: WithSpan<&String>,
    value: &WithSpan<Expr>,
) {
    compile_expr(compiler, expr);
    compile_expr(compiler, value);
    let constant = compiler.add_constant(identifier.value.as_str());
    compiler.add_instruction(Instruction::SetProperty(constant));
}

fn compile_unary(
    compiler: &mut Compiler,
    operator: WithSpan<UnaryOperator>,
    expr: &WithSpan<Expr>,
) {
    compile_expr(compiler, expr);
    match operator.value {
        UnaryOperator::Minus => compiler.add_instruction(Instruction::Negate),
        UnaryOperator::Bang => compiler.add_instruction(Instruction::Not),
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
        
        let constant = compiler.add_constant(ident.value.as_str());
        compiler.add_instruction(Instruction::Invoke(constant, args.len()));
    } else {
        compile_expr(compiler, identifier);

        for arg in args {
            compile_expr(compiler, arg);
        }

        compiler.add_instruction(Instruction::Call(args.len()));
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
    let else_jump = compiler.add_instruction(Instruction::JumpIfFalse(0));
    let end_jump = compiler.add_instruction(Instruction::Jump(0));
    compiler.patch_instruction(else_jump);
    compiler.add_instruction(Instruction::Pop);
    compile_expr(compiler, right);
    compiler.patch_instruction(end_jump);
}

fn compile_logical_and(
    compiler: &mut Compiler,
    left: &WithSpan<Expr>,
    right: &WithSpan<Expr>,
) {
    compile_expr(compiler, left);
    let end_jump = compiler.add_instruction(Instruction::JumpIfFalse(0));
    compiler.add_instruction(Instruction::Pop);
    compile_expr(compiler, right);
    compiler.patch_instruction(end_jump);
}

fn compile_boolean(compiler: &mut Compiler, boolean: bool) {
    if boolean {
        compiler.add_instruction(Instruction::True);
    } else {
        compiler.add_instruction(Instruction::False);
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
        compiler.add_instruction(Instruction::SetLocal(local));
    } else if let Some(upvalue) = compiler.resolve_upvalue(identifier.value) {
        // Upvalue
        compiler.add_instruction(Instruction::SetUpvalue(upvalue));
    } else {
        // Global
        let constant = compiler.add_constant(identifier.value.as_str());
        compiler.add_instruction(Instruction::SetGlobal(constant));
    }
}

fn compile_variable(
    compiler: &mut Compiler,
    identifier: WithSpan<&String>,
) {
    if let Some(local) = compiler.resolve_local(identifier.value) {
        // Local
        compiler.add_instruction(Instruction::GetLocal(local));
    } else if let Some(upvalue) = compiler.resolve_upvalue(identifier.value) {
        // Upvalue
        compiler.add_instruction(Instruction::GetUpvalue(upvalue));
    } else {
        // Global
        let constant = compiler.add_constant(identifier.value.as_str());
        compiler.add_instruction(Instruction::GetGlobal(constant));
    }
}

fn compile_nil(compiler: &mut Compiler) {
    compiler.add_instruction(Instruction::Nil);
}

fn compile_number(compiler: &mut Compiler, num: f64) {
    let constant = compiler.add_constant(num);
    compiler.add_instruction(Instruction::Constant(constant));
}

fn compile_string(compiler: &mut Compiler, string: &str) {
    let constant = compiler.add_constant(string);
    compiler.add_instruction(Instruction::Constant(constant));
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
        BinaryOperator::Plus => compiler.add_instruction(Instruction::Add),
        BinaryOperator::Minus => compiler.add_instruction(Instruction::Subtract),
        BinaryOperator::Less => compiler.add_instruction(Instruction::Less),
        BinaryOperator::LessEqual => {
            compiler.add_instruction(Instruction::Greater);
            compiler.add_instruction(Instruction::Not)
        }
        BinaryOperator::Star => compiler.add_instruction(Instruction::Multiply),
        BinaryOperator::EqualEqual => compiler.add_instruction(Instruction::Equal),
        BinaryOperator::BangEqual => {
            compiler.add_instruction(Instruction::Equal);
            compiler.add_instruction(Instruction::Not)
        }
        BinaryOperator::Greater => compiler.add_instruction(Instruction::Greater),
        BinaryOperator::GreaterEqual => {
            compiler.add_instruction(Instruction::Less);
            compiler.add_instruction(Instruction::Not)
        }
        BinaryOperator::Slash => compiler.add_instruction(Instruction::Divide),
    };
}

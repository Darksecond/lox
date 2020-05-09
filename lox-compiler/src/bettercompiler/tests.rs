use crate::bytecode::*;
use lox_syntax::ast::*;
use lox_syntax::SyntaxError;

fn parse_stmt(data: &str) -> Result<Vec<Stmt>, SyntaxError> {
    lox_syntax::parse(data)
}

fn assert_first_chunk(data: &str, constants: Vec<Constant>, instructions: Vec<Instruction>) {
    use super::compile;
    let ast = parse_stmt(data).unwrap();
    let module = compile(&ast).unwrap();
    let chunk = module.chunk(0);
    assert_eq!(instructions, chunk.instructions());
    assert_eq!(constants, module.constants());
}

fn compile_code(data: &str) -> Module {
    use super::compile;
    let ast = parse_stmt(data).unwrap();
    compile(&ast).unwrap()
}

fn assert_instructions(chunk: &Chunk, instructions: Vec<Instruction>) {
    assert_eq!(instructions, chunk.instructions());
}

fn assert_constants(module: &Module, constants: Vec<Constant>) {
    assert_eq!(constants, module.constants());
}

#[test]
fn test_stmt_print_numbers() {
    assert_first_chunk(
        "print 3;", 
        vec![3.0.into()],
        vec![Instruction::Constant(0), Instruction::Print, Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "print 1+2;", 
        vec![1.0.into(), 2.0.into()],
        vec![Instruction::Constant(0), Instruction::Constant(1), Instruction::Add, Instruction::Print, Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "print 1-2;", 
        vec![1.0.into(), 2.0.into()],
        vec![Instruction::Constant(0), Instruction::Constant(1), Instruction::Subtract, Instruction::Print, Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "print nil;", 
        vec![],
        vec![Instruction::Nil, Instruction::Print, Instruction::Nil, Instruction::Return]
    );
}

#[test]
fn test_stmt_print_strings() {
    assert_first_chunk(
        "print \"Hello, World!\";", 
        vec!["Hello, World!".into()],
        vec![Instruction::Constant(0), Instruction::Print, Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "print \"Hello, \" + \"World!\";", 
        vec!["Hello, ".into(), "World!".into()],
        vec![Instruction::Constant(0), Instruction::Constant(1), Instruction::Add, Instruction::Print, Instruction::Nil, Instruction::Return]
    );
}

#[test]
fn test_global_variables() {
    use crate::bytecode::Instruction::*;
    assert_first_chunk(
        "var x=3;", 
        vec![3.0.into(), "x".into()],
        vec![Instruction::Constant(0), Instruction::DefineGlobal(1), Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "var x;", 
        vec!["x".into()],
        vec![Instruction::Nil, Instruction::DefineGlobal(0), Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "var x=3; print x;", 
        vec![3.0.into(), "x".into(), "x".into()],
        vec![Instruction::Constant(0), Instruction::DefineGlobal(1), Instruction::GetGlobal(2), Instruction::Print, Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "var x=3;x=2;", 
        vec![3.0.into(), "x".into(), 2.0.into(), "x".into()],
        vec![Constant(0), DefineGlobal(1), Constant(2), SetGlobal(3), Pop, Instruction::Nil, Instruction::Return]
    );
}

#[test]
fn test_local_variables() {
    use crate::bytecode::Instruction::*;
    assert_first_chunk(
        "{var x=3;}", 
        vec![3.0.into()],
        vec![Instruction::Constant(0), Instruction::Pop, Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "{var x=3; print x;}", 
        vec![3.0.into()],
        vec![Instruction::Constant(0), Instruction::GetLocal(1), Instruction::Print, Instruction::Pop, Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "var x=2; {var x=3; { var x=4; print x; } print x;} print x;", 
        vec![2.0.into(), "x".into(), 3.0.into(), 4.0.into(), "x".into()],
        vec![Constant(0), DefineGlobal(1), Constant(2), Constant(3), GetLocal(2), Print, Pop, GetLocal(1), Print, Pop, GetGlobal(4), Print, Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "{var x;}", 
        vec![],
        vec![Instruction::Nil, Instruction::Pop, Instruction::Nil, Instruction::Return]
    );
    assert_first_chunk(
        "{var x;x=2;}", 
        vec![2.0.into()],
        vec![Nil, Constant(0), SetLocal(1), Pop, Pop, Instruction::Nil, Instruction::Return]
    );
}

#[test]
fn test_expression() {
    use crate::bytecode::Instruction::*;
    assert_first_chunk(
        "3;", 
        vec![3.0.into()],
        vec![Constant(0), Pop, Nil, Return]
    );

    assert_first_chunk(
        "true;", 
        vec![],
        vec![True, Pop, Nil, Return]
    );

    assert_first_chunk(
        "false;", 
        vec![],
        vec![False, Pop, Nil, Return]
    );

    assert_first_chunk(
        "nil;", 
        vec![],
        vec![Nil, Pop, Nil, Return]
    );
}

#[test]
fn test_if() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "if(false) 3;4;", 
        vec![3.0.into(), 4.0.into()],
        vec![False, JumpIfFalse(5), Pop, Constant(0), Pop, Constant(1), Pop, Nil, Return],
    );

    assert_first_chunk(
        "if(false) 3; else 4;5;", 
        vec![3.0.into(), 4.0.into(), 5.0.into()],
        vec![False, JumpIfFalse(6), Pop, Constant(0), Pop, Jump(9), Pop, Constant(1), Pop, Constant(2), Pop, Nil, Return],
    );
}

#[test]
fn test_logical_operators() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "3 and 4;", 
        vec![3.0.into(), 4.0.into()],
        vec![Constant(0), JumpIfFalse(4), Pop, Constant(1), Pop, Nil, Return],
    );

    assert_first_chunk(
        "3 or 4;", 
        vec![3.0.into(), 4.0.into()],
        vec![Constant(0), JumpIfFalse(3), Jump(5), Pop, Constant(1), Pop, Nil, Return],
    );
}

#[test]
fn test_equality() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "3 < 4;", 
        vec![3.0.into(), 4.0.into()],
        vec![Constant(0), Constant(1), Less, Pop, Nil, Return],
    );
}

#[test]
fn test_while() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "while(true) print 3;", 
        vec![3.0.into()],
        vec![True, JumpIfFalse(6), Pop, Constant(0), Print, Jump(0), Pop, Nil, Return],
    );
}

#[test]
fn test_for() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "for(var i = 0; i < 10; i = i + 1) print i;", 
        vec![0.0.into(), 10.0.into(), 1.0.into()],
        vec![Constant(0), GetLocal(1), Constant(1), Less, JumpIfFalse(14), Pop, GetLocal(1), Print, GetLocal(1), Constant(2), Add, SetLocal(1), Pop, Jump(1), Pop, Pop, Instruction::Nil, Instruction::Return],
    );
}

#[test]
fn test_simple_function() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first() { print 3; } first();");

    assert_instructions(module.chunk(0), vec![Closure(1), DefineGlobal(2), GetGlobal(3), Call(0), Pop, Instruction::Nil, Instruction::Return]);
    assert_instructions(module.chunk(1), vec![Constant(0), Print, Nil, Return]);

    assert_constants(&module, vec![
        3.0.into(),
        make_fun("first", 1, 0),
        "first".into(), 
        "first".into()
    ]);
}

#[test]
fn test_function_with_one_argument() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first(a) { print a; } first(3);");

    assert_instructions(module.chunk(0), vec![Closure(0), DefineGlobal(1), GetGlobal(2), Constant(3), Call(1), Pop, Instruction::Nil, Instruction::Return]);
    assert_instructions(module.chunk(1), vec![GetLocal(1), Print, Nil, Return]);

    assert_constants(&module, vec![
        make_fun("first", 1, 1),
        "first".into(), 
        "first".into(), 
        3.0.into()
    ]);
}

#[test]
fn test_recursive_function_with_one_argument() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first(a) { print first(a+1); } first(3);");

    assert_instructions(module.chunk(0), vec![Closure(2), DefineGlobal(3), GetGlobal(4), Constant(5), Call(1), Pop, Nil, Return]);
    assert_instructions(module.chunk(1), vec![GetGlobal(0), GetLocal(1), Constant(1), Add, Call(1), Print, Nil, Return]);

    assert_constants(&module, vec![
        "first".into(), 
        1.0.into(),  
        make_fun("first", 1, 1),
        "first".into(), 
        "first".into(), 
        3.0.into()
    ]);
}

#[test]
fn test_functions_calling_functions() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first() { second(); } fun second() { print 3; } first();");

    assert_instructions(module.chunk(0), vec![Closure(1), DefineGlobal(2), Closure(4), DefineGlobal(5), GetGlobal(6), Call(0), Pop, Instruction::Nil, Instruction::Return]);
    assert_instructions(module.chunk(1), vec![GetGlobal(0), Call(0), Pop, Nil, Return]);
    assert_instructions(module.chunk(2), vec![Constant(3), Print, Nil, Return]);

    assert_constants(&module, vec![
        "second".into(), 
        make_fun("first", 1, 0),
        "first".into(),
        3.0.into(),
        make_fun("second", 2, 0),
        "second".into(), 
        "first".into(),
    ]);
}

#[test]
fn test_simple_scoped_function() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{ fun first() { print 3; } first(); }");

    assert_instructions(module.chunk(0), vec![Closure(1), GetLocal(1), Call(0), Pop, Pop, Instruction::Nil, Instruction::Return]);
    assert_instructions(module.chunk(1), vec![Constant(0), Print, Nil, Return]);

    assert_constants(&module, vec![
        3.0.into(), 
        make_fun("first", 1, 0),
    ]);
}

#[test]
fn test_simple_scoped_recursive_function() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{ fun first() { print first(); } first(); }");

    assert_instructions(module.chunk(0), vec![Closure(0), GetLocal(1), Call(0), Pop, CloseUpvalue, Nil, Return]);
    assert_instructions(module.chunk(1), vec![GetUpvalue(0), Call(0), Print, Nil, Return]);

    assert_constants(&module, vec![
        make_closure("first", 1, 0, vec![Upvalue::Local(1)])
    ]);
}

#[test]
fn test_function_with_return() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first() { return 3; }");

    assert_instructions(module.chunk(0), vec![Closure(1), DefineGlobal(2), Instruction::Nil, Instruction::Return]);
    assert_instructions(module.chunk(1), vec![Constant(0), Return, Nil, Return]);

    assert_constants(&module, vec![
        3.0.into(),
        make_fun("first", 1, 0),
        "first".into()
    ]);
}

#[test]
fn test_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{var a = 3; fun f() { print a; }}");

    assert_instructions(module.chunk(0), vec![Constant(0), Closure(1), Pop, CloseUpvalue, Nil, Return]);
    assert_instructions(module.chunk(1), vec![GetUpvalue(0), Print, Nil, Return]);

    assert_constants(&module, vec![
        3.0.into(),
        make_closure("f", 1, 0, vec![Upvalue::Local(1)]),
    ]);
}

#[test]
fn test_double_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{var a = 3; fun f() { fun g() { print a; } }}");

    assert_instructions(module.chunk(0), vec![Constant(0), Closure(2), Pop, CloseUpvalue, Nil, Return]);
    assert_instructions(module.chunk(1), vec![Closure(1), Pop, Nil, Return]);
    assert_instructions(module.chunk(2), vec![GetUpvalue(0), Print, Nil, Return]);

    assert_constants(&module, vec![
        3.0.into(),
        make_closure("g", 2, 0, vec![Upvalue::Upvalue(0)]),
        make_closure("f", 1, 0, vec![Upvalue::Local(1)]),
    ]);
}

#[test]
fn test_multiple_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{var a = 3; var b = 4; fun f() {print b; print a; }}");

    assert_instructions(module.chunk(0), vec![Constant(0), Constant(1), Closure(2), Pop, CloseUpvalue, CloseUpvalue, Nil, Return]);
    assert_instructions(module.chunk(1), vec![GetUpvalue(0), Print, GetUpvalue(1), Print, Nil, Return]);

    assert_constants(&module, vec![
        3.0.into(),
        4.0.into(),
        make_closure("f", 1, 0, vec![Upvalue::Local(2), Upvalue::Local(1)]),
    ]);
}

#[test]
fn test_multiple_double_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{var a = 3; var b = 4; fun f() { fun g() { print a; print b; }}}");

    assert_instructions(module.chunk(0), vec![Constant(0), Constant(1), Closure(3), Pop, CloseUpvalue, CloseUpvalue, Nil, Return]);
    assert_instructions(module.chunk(1), vec![Closure(2), Pop, Nil, Return]);
    assert_instructions(module.chunk(2), vec![GetUpvalue(0), Print, GetUpvalue(1), Print, Nil, Return]);

    assert_constants(&module, vec![
        3.0.into(),
        4.0.into(),
        make_closure("g", 2, 0, vec![Upvalue::Upvalue(0), Upvalue::Upvalue(1)]),
        make_closure("f", 1, 0, vec![Upvalue::Local(1), Upvalue::Local(2)]),
    ]);
}

#[test]
fn test_scoped_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("var global; fun main() { { var a = 3; fun one() { print a; } global = one; } } main();");

    assert_instructions(module.chunk(0), vec![Nil, DefineGlobal(0), Closure(4), DefineGlobal(5), GetGlobal(6), Call(0), Pop, Instruction::Nil, Instruction::Return]);
    assert_instructions(module.chunk(1), vec![Constant(1), Closure(2), GetLocal(2), SetGlobal(3), Pop, Pop, CloseUpvalue, Nil, Return]);
    assert_instructions(module.chunk(2), vec![GetUpvalue(0), Print, Nil, Return]);

    assert_constants(&module, vec![
        "global".into(),
        3.0.into(),
        make_closure("one",2, 0, vec![Upvalue::Local(1)]),
        "global".into(),
        make_closure("main",1, 0, vec![]),
        "main".into(),
        "main".into(),
    ]);
}

#[test]
fn test_empty_class_global() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("class Foo {}");

    assert_instructions(module.chunk(0), vec![Class(0), DefineGlobal(1), Nil, Return]);

    assert_constants(&module, vec![
        make_class("Foo"),
        "Foo".into(),
    ]);
}

#[test]
fn test_empty_class_local() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{class Foo {}}");

    assert_instructions(module.chunk(0), vec![Class(0), Pop, Nil, Return]);

    assert_constants(&module, vec![
        make_class("Foo"),
    ]);
}

#[test]
fn test_set_property() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("x.test = 3;");

    assert_instructions(module.chunk(0), vec![GetGlobal(0), Constant(1), SetProperty(2), Pop, Nil, Return]);

    assert_constants(&module, vec![
        "x".into(),
        3.0.into(),
        "test".into(),
    ]);
}

#[test]
fn test_get_property() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("x.test;");

    assert_instructions(module.chunk(0), vec![GetGlobal(0), GetProperty(1), Pop, Nil, Return]);

    assert_constants(&module, vec![
        "x".into(),
        "test".into(),
    ]);
}

fn make_fun(name: &str, index: usize, arity: usize) -> Constant {
    crate::bytecode::Function{name: name.into(), chunk_index: index, arity: arity}.into()
}

fn make_closure(name: &str, index: usize, arity: usize, upvalues: Vec<Upvalue>) -> Constant {
    let function = crate::bytecode::Function{name: name.into(), chunk_index: index, arity: arity};
    let closure = crate::bytecode::Closure{function, upvalues};
    Constant::Closure(closure)
}

fn make_class(name: &str) -> Constant {
    Constant::Class(Class{ name: name.to_string() })
}
use crate::bytecode::*;
use crate::ast::*;

fn parse_stmt(data: &str) -> Result<Vec<Stmt>, String> {
    use crate::tokenizer::tokenize_with_context;
    let tokens = tokenize_with_context(data);
    println!("Tokens: {:?}", tokens);
    let mut it = tokens.as_slice().into_iter().map(|tc| &tc.token).peekable();
    crate::stmt_parser::parse(&mut it)
}

fn assert_first_chunk(data: &str, constants: Vec<Constant>, instructions: Vec<Instruction>) {
    use super::compile;
    let ast = parse_stmt(data).unwrap();
    let module = compile(&ast).unwrap();
    let chunk = module.chunk(0);
    assert_eq!(instructions, chunk.instructions());
    assert_eq!(constants, chunk.constants()); //TODO should be module
}

#[test]
fn test_stmt_print_numbers() {
    assert_first_chunk(
        "print 3;", 
        vec![3.0.into()],
        vec![Instruction::Constant(0), Instruction::Print]
    );
    assert_first_chunk(
        "print 1+2;", 
        vec![1.0.into(), 2.0.into()],
        vec![Instruction::Constant(0), Instruction::Constant(1), Instruction::Add, Instruction::Print]
    );
    assert_first_chunk(
        "print 1-2;", 
        vec![1.0.into(), 2.0.into()],
        vec![Instruction::Constant(0), Instruction::Constant(1), Instruction::Subtract, Instruction::Print]
    );
    assert_first_chunk(
        "print nil;", 
        vec![],
        vec![Instruction::Nil, Instruction::Print]
    );
}

#[test]
fn test_stmt_print_strings() {
    assert_first_chunk(
        "print \"Hello, World!\";", 
        vec!["Hello, World!".into()],
        vec![Instruction::Constant(0), Instruction::Print]
    );
    assert_first_chunk(
        "print \"Hello, \" + \"World!\";", 
        vec!["Hello, ".into(), "World!".into()],
        vec![Instruction::Constant(0), Instruction::Constant(1), Instruction::Add, Instruction::Print]
    );
}

#[test]
fn test_global_variables() {
    use crate::bytecode::Instruction::*;
    assert_first_chunk(
        "var x=3;", 
        vec![3.0.into(), "x".into()],
        vec![Instruction::Constant(0), Instruction::DefineGlobal(1)]
    );
    assert_first_chunk(
        "var x;", 
        vec!["x".into()],
        vec![Instruction::Nil, Instruction::DefineGlobal(0)]
    );
    assert_first_chunk(
        "var x=3; print x;", 
        vec![3.0.into(), "x".into(), "x".into()],
        vec![Instruction::Constant(0), Instruction::DefineGlobal(1), Instruction::GetGlobal(2), Instruction::Print]
    );
    assert_first_chunk(
        "var x=3;x=2;", 
        vec![3.0.into(), "x".into(), 2.0.into(), "x".into()],
        vec![Constant(0), DefineGlobal(1), Constant(2), SetGlobal(3), Pop]
    );
}

#[test]
fn test_local_variables() {
    use crate::bytecode::Instruction::*;
    assert_first_chunk(
        "{var x=3;}", 
        vec![3.0.into()],
        vec![Instruction::Constant(0), Instruction::Pop]
    );
    assert_first_chunk(
        "{var x=3; print x;}", 
        vec![3.0.into()],
        vec![Instruction::Constant(0), Instruction::GetLocal(0), Instruction::Print, Instruction::Pop]
    );
    assert_first_chunk(
        "var x=2; {var x=3; { var x=4; print x; } print x;} print x;", 
        vec![2.0.into(), "x".into(), 3.0.into(), 4.0.into(), "x".into()],
        vec![Constant(0), DefineGlobal(1), Constant(2), Constant(3), GetLocal(1), Print, Pop, GetLocal(0), Print, Pop, GetGlobal(4), Print]
    );
    assert_first_chunk(
        "{var x;}", 
        vec![],
        vec![Instruction::Nil, Instruction::Pop]
    );
    assert_first_chunk(
        "{var x;x=2;}", 
        vec![2.0.into()],
        vec![Nil, Constant(0), SetLocal(0), Pop, Pop]
    );
}

#[test]
fn test_expression() {
    use crate::bytecode::Instruction::*;
    assert_first_chunk(
        "3;", 
        vec![3.0.into()],
        vec![Constant(0), Pop]
    );

    assert_first_chunk(
        "true;", 
        vec![],
        vec![True, Pop]
    );

    assert_first_chunk(
        "false;", 
        vec![],
        vec![False, Pop]
    );

    assert_first_chunk(
        "nil;", 
        vec![],
        vec![Nil, Pop]
    );
}

#[test]
fn test_if() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "if(false) 3;4;", 
        vec![3.0.into(), 4.0.into()],
        vec![False, JumpIfFalse(5), Pop, Constant(0), Pop, Constant(1), Pop],
    );

    assert_first_chunk(
        "if(false) 3; else 4;5;", 
        vec![3.0.into(), 4.0.into(), 5.0.into()],
        vec![False, JumpIfFalse(6), Pop, Constant(0), Pop, Jump(9), Pop, Constant(1), Pop, Constant(2), Pop],
    );
}

#[test]
fn test_logical_operators() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "3 and 4;", 
        vec![3.0.into(), 4.0.into()],
        vec![Constant(0), JumpIfFalse(4), Pop, Constant(1), Pop],
    );

    assert_first_chunk(
        "3 or 4;", 
        vec![3.0.into(), 4.0.into()],
        vec![Constant(0), JumpIfFalse(3), Jump(5), Pop, Constant(1), Pop],
    );
}
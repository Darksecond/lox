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
use lox;
use lox::bytecode::*;
use lox::tokenizer::*;
use lox::ast::*;
use lox::compiler::*;
use lox::vm::*;

fn compile(ast: &Vec<Stmt>) -> Chunk {
    let mut chunk = Chunk::new();
    for stmt in ast {
        compile_stmt(&mut chunk, stmt);
    }
    chunk
}

fn parse_stmt(data: &str) -> Result<Vec<Stmt>, String> {
    let tokens = tokenize_with_context(data);
    println!("Tokens: {:?}", tokens);
    let mut it = tokens.as_slice().into_iter().map(|tc| &tc.token).peekable();
    lox::stmt_parser::parse(&mut it)
}

fn main() {
    // let data = "print 1 + 2;1+2;print 3;";
    let data = "print 1+2*5+12;print 3;print 2+3;";
    let ast = parse_stmt(data).unwrap();
    println!();
    let mut chunk = compile(&ast);
    chunk.add_instruction(Instruction::Return);
    println!("{:?}", chunk.constants());
    println!("{:?}", chunk.instructions());

    println!();
    

    let mut state = VmState::new();
    let mut vm = Vm::new(&mut state, &chunk);
    while vm.interpret_next() {
    }
}
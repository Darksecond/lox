use lox;
use lox::bytecode::*;
use lox::tokenizer::*;
use lox::ast::*;
use lox::vm::*;

fn parse_stmt(data: &str) -> Result<Vec<Stmt>, String> {
    let tokens = tokenize_with_context(data);
    println!("Tokens: {:?}", tokens);
    let mut it = tokens.as_slice().into_iter().map(|tc| &tc.token).peekable();
    lox::stmt_parser::parse(&mut it)
}

fn main() {
    // let data = "print 1 + 2;1+2;print 3;";
    // let data = "print 1+2*5+12;print 3;print 2+3;";
    // let data = "var x=1+3;x=x+2;print x;";
    // let data = "var x = \"Hi!\";print \"Hello, World!\";x=3;";
    // let data = "var a = \"He\"; var b = \"llo\";print a+b;";
    // let data = "print \"He\"+\"llo\";";
    // let data = "var x=93;{var x=123; {var y=3; var x=4; {print x;x=6;}print x+3;} print x;}print x;";
    // let data ="print 12+8-3;print 12.3;";
    // let data = "var x = 3;";
    // let data = "var x=2; {var x=3; { var x=4; print x; } print x;} print x;";
    // let data = "{var x=2;{ x=3;} print x;}";
    // let data ="{var x=2; x=3;print x;}";
    // let data = "{var x;} {var x; var y;} {var w; {var x; var y;} var z;}";
    // let data ="{var x=1;{}var y=2;{var z=3;}var a=4;}";
    // let data = "if(true) print 3; else print 4; print 5;";
    // let data = "var i =0; while(i < 1000) { print i; i = i + 1; }";
    let data = "for(var i =0; i < 1000; i = i + 1) print i;";
    let ast = parse_stmt(data).unwrap();
    println!();

    let mut module = lox::bettercompiler::compile(&ast).unwrap();
    let chunk = module.chunk_mut(0);

    //HACK
    chunk.add_instruction(Instruction::Return);
    println!("{:?}", chunk.constants());
    println!("{:?}", chunk.instructions());

    println!();
    

    let mut state = VmState::new();
    let mut vm = Vm::new(&mut state, &chunk);
    while vm.interpret_next() {
        vm.collect();
    }
    vm.collect();
}
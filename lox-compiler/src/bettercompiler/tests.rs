use crate::bytecode::*;
use lox_bytecode::opcode;
use lox_syntax::ast::*;
use lox_syntax::position::Diagnostic;
use lox_syntax::position::WithSpan;

fn parse_stmt(data: &str) -> Result<Vec<WithSpan<Stmt>>, Vec<Diagnostic>> {
    lox_syntax::parse(data)
}

fn assert_first_chunk(data: &str, numbers: Vec<f64>, strings: Vec<String>, identifiers: Vec<&str>, instructions: Vec<u8>) {
    use super::compile;
    let ast = parse_stmt(data).unwrap();
    let module = compile(&ast).unwrap();
    let chunk = module.chunk(0);
    assert_eq!(instructions, chunk.as_slice());
    assert_eq!(numbers, module.numbers);
    assert_eq!(strings, module.strings);
    assert_eq!(identifiers, module.identifiers());
}

fn compile_code(data: &str) -> Module {
    use super::compile;
    let ast = parse_stmt(data).unwrap();
    compile(&ast).unwrap()
}

fn assert_chunk0(module: &Module, instructions: Vec<u8>) {
    assert_instructions(module.chunk(0), instructions);
}

fn assert_instructions(chunk: &Chunk, instructions: Vec<u8>) {
    assert_eq!(instructions, chunk.as_slice());
}

fn assert_strings(module: &Module, constants: Vec<String>) {
    assert_eq!(constants, module.strings);
}

fn assert_numbers(module: &Module, constants: Vec<f64>) {
    assert_eq!(constants, module.numbers);
}

fn assert_closures(module: &Module, closures: Vec<Closure>) {
    assert_eq!(closures, module.closures());
}

fn assert_classes(module: &Module, classes: Vec<Class>) {
    assert_eq!(classes, module.classes());
}

fn assert_identifiers(module: &Module, identifiers: Vec<&str>) {
    assert_eq!(identifiers, module.identifiers());
}

#[test]
fn test_stmt_print_numbers() {
    {
        let module = compile_code("print 3;");
        assert_chunk0(&module, vec![
                      opcode::NUMBER,
                      0, 0,
                      opcode::PRINT,
                      opcode::RETURN_TOP,
        ]);

        assert_numbers(&module, vec![3.0]);
        assert_strings(&module, vec![]);
        assert_identifiers(&module, vec![]);
    }


    assert_first_chunk(
        "print 1+2;",
        vec![1.0.into(), 2.0.into()],
        vec![],
        vec![],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::NUMBER,
            1, 0,
            opcode::ADD,
            opcode::PRINT,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "print 1-2;",
        vec![1.0.into(), 2.0.into()],
        vec![],
        vec![],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::NUMBER,
            1, 0,
            opcode::SUBTRACT,
            opcode::PRINT,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "print nil;",
        vec![],
        vec![],
        vec![],
        vec![
            opcode::NIL,
            opcode::PRINT,
            opcode::RETURN_TOP,
        ],
    );
}

#[test]
fn test_stmt_print_strings() {
    assert_first_chunk(
        "print \"Hello, World!\";",
        vec![],
        vec!["Hello, World!".into()],
        vec![],
        vec![
            opcode::STRING,
            0, 0,
            opcode::PRINT,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "print \"Hello, \" + \"World!\";",
        vec![],
        vec!["Hello, ".into(), "World!".into()],
        vec![],
        vec![
            opcode::STRING,
            0, 0,
            opcode::STRING,
            1, 0,
            opcode::ADD,
            opcode::PRINT,
            opcode::RETURN_TOP,
        ],
    );
}

#[test]
fn test_global_variables() {
    assert_first_chunk(
        "var x=3;",
        vec![3.0.into()],
        vec![],
        vec!["x"],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "var x;",
        vec![],
        vec![],
        vec!["x"],
        vec![
            opcode::NIL,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "var x=3; print x;",
        vec![3.0.into()],
        vec![],
        vec!["x"],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::GET_GLOBAL,
            0, 0, 0, 0,
            opcode::PRINT,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "var x=3;x=2;",
        vec![3.0.into(), 2.0.into()],
        vec![],
        vec!["x"],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::NUMBER,
            1, 0,
            opcode::SET_GLOBAL,
            0, 0, 0, 0,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
}

#[test]
fn test_local_variables() {
    assert_first_chunk(
        "{var x=3;}",
        vec![3.0.into()],
        vec![],
        vec![],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "{var x=3; print x;}",
        vec![3.0.into()],
        vec![],
        vec![],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::GET_LOCAL,
            1, 0, 0, 0,
            opcode::PRINT,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "var x=2; {var x=3; { var x=4; print x; } print x;} print x;",
        vec![2.0.into(), 3.0.into(), 4.0.into()],
        vec![],
        vec!["x"],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::NUMBER,
            1, 0,
            opcode::NUMBER,
            2, 0,
            opcode::GET_LOCAL,
            2, 0, 0, 0,
            opcode::PRINT,
            opcode::POP,
            opcode::GET_LOCAL,
            1, 0, 0, 0,
            opcode::PRINT,
            opcode::POP,
            opcode::GET_GLOBAL,
            0, 0, 0, 0,
            opcode::PRINT,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "{var x;}",
        vec![],
        vec![],
        vec![],
        vec![
            opcode::NIL,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
    assert_first_chunk(
        "{var x;x=2;}",
        vec![2.0.into()],
        vec![],
        vec![],
        vec![
            opcode::NIL,
            opcode::NUMBER,
            0, 0,
            opcode::SET_LOCAL,
            1, 0, 0, 0,
            opcode::POP,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
}

#[test]
fn test_expression() {
    assert_first_chunk("3;", vec![3.0.into()], vec![], vec![], vec![
                       opcode::NUMBER,
                       0, 0, 
                       opcode::POP,
                       opcode::RETURN_TOP,
    ]);

    assert_first_chunk("true;", vec![], vec![], vec![],vec![opcode::TRUE, opcode::POP, opcode::RETURN_TOP]);

    assert_first_chunk("false;", vec![], vec![], vec![],vec![opcode::FALSE, opcode::POP, opcode::RETURN_TOP]);

    assert_first_chunk("nil;", vec![], vec![], vec![],vec![opcode::NIL, opcode::POP, opcode::RETURN_TOP]);
}

#[test]
fn test_if() {
    assert_first_chunk(
        "if(false) 3;4;",
        vec![3.0.into(), 4.0.into()],
        vec![],
        vec![],
        vec![
            opcode::FALSE,
            opcode::JUMP_IF_FALSE,
            5, 0,
            opcode::POP,
            opcode::NUMBER,
            0, 0,
            opcode::POP,
            opcode::NUMBER,
            1, 0,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );

    assert_first_chunk(
        "if(false) 3; else 4;5;",
        vec![3.0.into(), 4.0.into(), 5.0.into()],
        vec![],
        vec![],
        vec![
            opcode::FALSE,
            opcode::JUMP_IF_FALSE,
            8, 0,
            opcode::POP,
            opcode::NUMBER,
            0, 0,
            opcode::POP,
            opcode::JUMP,
            5, 0,
            opcode::POP,
            opcode::NUMBER,
            1, 0,
            opcode::POP,
            opcode::NUMBER,
            2, 0,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
}

#[test]
fn test_logical_operators() {
    assert_first_chunk(
        "3 and 4;",
        vec![3.0.into(), 4.0.into()],
        vec![],
        vec![],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::JUMP_IF_FALSE,
            4, 0,
            opcode::POP,
            opcode::NUMBER,
            1, 0,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );

    assert_first_chunk(
        "3 or 4;",
        vec![3.0.into(), 4.0.into()],
        vec![],
        vec![],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::JUMP_IF_FALSE,
            3, 0,
            opcode::JUMP,
            4, 0,
            opcode::POP,
            opcode::NUMBER,
            1, 0,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
}

#[test]
fn test_equality() {
    assert_first_chunk(
        "3 < 4;",
        vec![3.0.into(), 4.0.into()],
        vec![],
        vec![],
        vec![
            opcode::NUMBER, 
            0, 0,
            opcode::NUMBER, 
            1, 0,
            opcode::LESS,
            opcode::POP,
            opcode::RETURN_TOP
        ]);
}

#[test]
fn test_while() {
    assert_first_chunk(
        "while(true) print 3;",
        vec![3.0.into()],
        vec![],
        vec![],
        vec![
            opcode::TRUE,
            opcode::JUMP_IF_FALSE,
            8, 0,
            opcode::POP,
            opcode::NUMBER,
            0, 0,
            opcode::PRINT,
            opcode::JUMP,
            244, 255,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
}

#[test]
fn test_for() {
    assert_first_chunk(
        "for(var i = 0; i < 10; i = i + 1) print i;",
        vec![0.0.into(), 10.0.into(), 1.0.into()],
        vec![],
        vec![],
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::GET_LOCAL,
            1, 0, 0, 0,
            opcode::NUMBER,
            1, 0,
            opcode::LESS,
            opcode::JUMP_IF_FALSE,
            25, 0,
            opcode::POP,
            opcode::GET_LOCAL,
            1, 0, 0, 0,
            opcode::PRINT,
            opcode::GET_LOCAL,
            1, 0, 0, 0,
            opcode::NUMBER,
            2, 0,
            opcode::ADD,
            opcode::SET_LOCAL,
            1, 0, 0, 0,
            opcode::POP,
            opcode::JUMP,
            219, 255,
            opcode::POP,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
}

#[test]
fn test_simple_function() {
    let module = compile_code("fun first() { print 3; } first();");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::CLOSURE,
            0, 0, 0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::GET_GLOBAL,
            0, 0, 0, 0,
            opcode::CALL,
            0,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(module.chunk(1), vec![opcode::NUMBER, 0, 0, opcode::PRINT, opcode::NIL, opcode::RETURN]);

    assert_numbers(&module, vec![3.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_fun("first", 1, 0),
    ]);

    assert_identifiers(&module, vec![
        "first",
    ]);
}

#[test]
fn test_function_with_one_argument() {
    let module = compile_code("fun first(a) { print a; } first(3);");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::CLOSURE,
            0, 0, 0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::GET_GLOBAL,
            0, 0, 0, 0,
            opcode::NUMBER,
            0, 0,
            opcode::CALL,
            1,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(module.chunk(1), vec![
                        opcode::GET_LOCAL,
                        1, 0, 0, 0,
                        opcode::PRINT, 
                        opcode::NIL, 
                        opcode::RETURN,
    ]);

    assert_numbers(&module, vec![3.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_fun("first", 1, 1),
    ]);

    assert_identifiers(&module, vec![
        "first"
    ]);
}

#[test]
fn test_recursive_function_with_one_argument() {
    let module = compile_code("fun first(a) { print first(a+1); } first(3);");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::CLOSURE,
            0, 0, 0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::GET_GLOBAL,
            0, 0, 0, 0,
            opcode::NUMBER,
            1, 0,
            opcode::CALL,
            1,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(
        module.chunk(1),
        vec![
            opcode::GET_GLOBAL,
            0, 0, 0, 0,
            opcode::GET_LOCAL,
            1, 0, 0, 0,
            opcode::NUMBER,
            0, 0,
            opcode::ADD,
            opcode::CALL,
            1,
            opcode::PRINT,
            opcode::NIL,
            opcode::RETURN,
        ],
    );

    assert_numbers(&module, vec![1.0, 3.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_fun("first", 1, 1),
    ]);

    assert_identifiers(&module, vec![
        "first"
    ]);
}

#[test]
fn test_functions_calling_functions() {
    let module = compile_code("fun first() { second(); } fun second() { print 3; } first();");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::CLOSURE,
            0, 0, 0, 0,
            opcode::DEFINE_GLOBAL,
            1, 0, 0, 0,
            opcode::CLOSURE,
            1, 0, 0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::GET_GLOBAL,
            1, 0, 0, 0,
            opcode::CALL,
            0,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(
        module.chunk(1), vec![
        opcode::GET_GLOBAL,
        0, 0, 0, 0,
        opcode::CALL,
        0,
        opcode::POP,
        opcode::NIL,
        opcode::RETURN,
    ]);
    assert_instructions(module.chunk(2), vec![
                        opcode::NUMBER,
                        0, 0,
                        opcode::PRINT,
                        opcode::NIL,
                        opcode::RETURN,
    ]);

    assert_numbers(&module, vec![3.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_fun("first", 1, 0),
        make_fun("second", 2, 0),
    ]);

    assert_identifiers(&module, vec![
        "second",
        "first",
    ]);
}

#[test]
fn test_simple_scoped_function() {
    let module = compile_code("{ fun first() { print 3; } first(); }");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::CLOSURE,
            0, 0, 0, 0,
            opcode::GET_LOCAL,
            1, 0, 0, 0,
            opcode::CALL,
            0,
            opcode::POP,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(module.chunk(1), vec![
                        opcode::NUMBER,
                        0, 0,
                        opcode::PRINT,
                        opcode::NIL,
                        opcode::RETURN,
    ]);

    assert_numbers(&module, vec![3.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_fun("first", 1, 0),
    ]);
}

#[test]
fn test_simple_scoped_recursive_function() {
    let module = compile_code("{ fun first() { print first(); } first(); }");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::CLOSURE,
            0, 0, 0, 0,
            opcode::GET_LOCAL,
            1, 0, 0, 0,
            opcode::CALL,
            0,
            opcode::POP,
            opcode::CLOSE_UPVALUE,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(
        module.chunk(1),
        vec![
            opcode::GET_UPVALUE,
            0, 0, 0, 0,
            opcode::CALL,
            0,
            opcode::PRINT,
            opcode::NIL,
            opcode::RETURN
        ]);

    assert_closures(&module, vec![
        make_closure("first", 1, 0, vec![Upvalue::Local(1)]),
    ]);
}

#[test]
fn test_function_with_return() {
    let module = compile_code("fun first() { return 3; }");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::CLOSURE,
            0, 0, 0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(module.chunk(1), vec![
                        opcode::NUMBER,
                        0, 0,
                        opcode::RETURN,
                        opcode::NIL,
                        opcode::RETURN
    ]);

    assert_numbers(&module, vec![3.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_fun("first", 1, 0),
    ]);

    assert_identifiers(&module, vec![
        "first"
    ]);
}

#[test]
fn test_upvalue() {
    let module = compile_code("{var a = 3; fun f() { print a; }}");

    assert_instructions(
        module.chunk(0),
        vec![
        opcode::NUMBER,
        0, 0,
        opcode::CLOSURE,
        0, 0, 0, 0,
        opcode::POP,
        opcode::CLOSE_UPVALUE,
        opcode::RETURN_TOP,
        ]);
    assert_instructions(module.chunk(1), vec![
                        opcode::GET_UPVALUE,
                        0, 0, 0, 0,
                        opcode::PRINT,
                        opcode::NIL,
                        opcode::RETURN,
    ]);

    assert_numbers(&module, vec![3.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_closure("f", 1, 0, vec![Upvalue::Local(1)])
    ]);
}

#[test]
fn test_double_upvalue() {
    let module = compile_code("{var a = 3; fun f() { fun g() { print a; } }}");

    assert_instructions(
        module.chunk(0),
        vec![
        opcode::NUMBER,
        0, 0,
        opcode::CLOSURE,
        1, 0, 0, 0,
        opcode::POP,
        opcode::CLOSE_UPVALUE,
        opcode::RETURN_TOP,
        ]);
    assert_instructions(module.chunk(1), vec![
                        opcode::CLOSURE,
                        0, 0, 0, 0,
                        opcode::NIL,
                        opcode::RETURN
    ]);
    assert_instructions(module.chunk(2), vec![
                        opcode::GET_UPVALUE,
                        0, 0, 0, 0,
                        opcode::PRINT,
                        opcode::NIL,
                        opcode::RETURN
    ]);

    assert_numbers(&module, vec![3.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_closure("g", 2, 0, vec![Upvalue::Upvalue(0)]),
        make_closure("f", 1, 0, vec![Upvalue::Local(1)]),
    ]);
}

#[test]
fn test_multiple_upvalue() {
    let module = compile_code("{var a = 3; var b = 4; fun f() {print b; print a; }}");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::NUMBER,
            1, 0,
            opcode::CLOSURE,
            0, 0, 0, 0,
            opcode::POP,
            opcode::CLOSE_UPVALUE,
            opcode::CLOSE_UPVALUE,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(
        module.chunk(1),
        vec![
            opcode::GET_UPVALUE,
            0, 0, 0, 0,
            opcode::PRINT,
            opcode::GET_UPVALUE,
            1, 0, 0, 0,
            opcode::PRINT,
            opcode::NIL,
            opcode::RETURN
        ]);

    assert_numbers(&module, vec![3.0, 4.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_closure("f", 1, 0, vec![Upvalue::Local(2), Upvalue::Local(1)]),
    ]);
}

#[test]
fn test_multiple_double_upvalue() {
    let module = compile_code("{var a = 3; var b = 4; fun f() { fun g() { print a; print b; }}}");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::NUMBER,
            1, 0,
            opcode::CLOSURE,
            1, 0, 0, 0,
            opcode::POP,
            opcode::CLOSE_UPVALUE,
            opcode::CLOSE_UPVALUE,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(module.chunk(1), vec![
                        opcode::CLOSURE,
                        0, 0, 0, 0,
                        opcode::NIL,
                        opcode::RETURN
    ]);
    assert_instructions(
        module.chunk(2),
        vec![
            opcode::GET_UPVALUE,
            0, 0, 0, 0,
            opcode::PRINT,
            opcode::GET_UPVALUE,
            1, 0, 0, 0,
            opcode::PRINT,
            opcode::NIL,
            opcode::RETURN
        ]);

    assert_numbers(&module, vec![3.0, 4.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_closure("g", 2, 0, vec![Upvalue::Upvalue(0), Upvalue::Upvalue(1)]),
        make_closure("f", 1, 0, vec![Upvalue::Local(1), Upvalue::Local(2)]),
    ]);
}

#[test]
fn test_scoped_upvalue() {
    let module = compile_code(
        "var global; fun main() { { var a = 3; fun one() { print a; } global = one; } } main();",
    );

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::NIL,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::CLOSURE,
            1, 0, 0, 0,
            opcode::DEFINE_GLOBAL,
            1, 0, 0, 0,
            opcode::GET_GLOBAL,
            1, 0, 0, 0,
            opcode::CALL,
            0,
            opcode::POP,
            opcode::RETURN_TOP,
        ],
    );
    assert_instructions(
        module.chunk(1),
        vec![
            opcode::NUMBER,
            0, 0,
            opcode::CLOSURE,
            0, 0, 0, 0,
            opcode::GET_LOCAL,
            2, 0, 0, 0,
            opcode::SET_GLOBAL,
            0, 0, 0, 0,
            opcode::POP,
            opcode::POP,
            opcode::CLOSE_UPVALUE,
            opcode::NIL,
            opcode::RETURN,
        ],
    );
    assert_instructions(module.chunk(2), vec![
                        opcode::GET_UPVALUE,
                        0, 0, 0, 0,
                        opcode::PRINT,
                        opcode::NIL,
                        opcode::RETURN
    ]);

    assert_numbers(&module, vec![3.0]);
    assert_strings(&module, vec![]);

    assert_closures(&module, vec![
        make_closure("one", 2, 0, vec![Upvalue::Local(1)]),
        make_closure("main", 1, 0, vec![]),
    ]);

    assert_identifiers(&module, vec![
        "global",
        "main",
    ]);
}

#[test]
fn test_simple_import() {
    let module = compile_code("import \"foo\";");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::IMPORT,
            0, 0, 0, 0,
            opcode::POP,
            opcode::RETURN_TOP
        ]);

    assert_numbers(&module, vec![]);
    assert_strings(&module, vec!["foo".into()]);
}

#[test]
fn test_complex_import() {
    let module = compile_code("import \"foo\" for x;");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::IMPORT,
            0, 0, 0, 0,
            opcode::IMPORT_GLOBAL,
            0, 0, 0, 0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::POP,
            opcode::RETURN_TOP
        ]);

    assert_numbers(&module, vec![]);
    assert_strings(&module, vec!["foo".into()]);

    assert_identifiers(&module, vec![
        "x"
    ]);
}

#[test]
fn test_complex_local_import() {
    let module = compile_code("{import \"foo\" for x; print x;}");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::IMPORT,
            0, 0, 0, 0,
            opcode::IMPORT_GLOBAL,
            0, 0, 0, 0,
            opcode::POP,
            opcode::GET_LOCAL,
            1, 0, 0, 0,
            opcode::PRINT,
            opcode::POP,
            opcode::RETURN_TOP
    ]);

    assert_numbers(&module, vec![]);
    assert_strings(&module, vec!["foo".into()]);

    assert_identifiers(&module, vec![
        "x"
    ]);
}

#[test]
fn test_empty_class_global() {
    let module = compile_code("class Foo {}");

    assert_instructions(
        module.chunk(0),
        vec![
            opcode::CLASS,
            0,
            opcode::DEFINE_GLOBAL,
            0, 0, 0, 0,
            opcode::GET_GLOBAL,
            0, 0, 0, 0,
            opcode::POP,
            opcode::RETURN_TOP
        ]);

    assert_numbers(&module, vec![]);
    assert_strings(&module, vec![]);

    assert_classes(&module, vec![
        make_class("Foo")
    ]);

    assert_identifiers(&module, vec![
        "Foo"
    ]);
}

#[test]
fn test_empty_class_local() {
    let module = compile_code("{class Foo {}}");

    assert_instructions(module.chunk(0), vec![
                        opcode::CLASS,
                        0,
                        opcode::GET_LOCAL,
                        1, 0, 0, 0,
                        opcode::POP,
                        opcode::POP,
                        opcode::RETURN_TOP,
    ]);

    assert_classes(&module, vec![
        make_class("Foo")
    ]);
}

#[test]
fn test_set_property() {
    use lox_bytecode::opcode::*;

    let module = compile_code("x.test = 3;");

    assert_instructions(
        module.chunk(0),
        vec![
            GET_GLOBAL,
            0, 0, 0, 0,
            NUMBER,
            0, 0,
            SET_PROPERTY,
            1, 0, 0, 0,
            POP,
            RETURN_TOP
        ]);

    assert_numbers(&module, vec![3.0]);
    assert_strings(&module, vec![]);

    assert_identifiers(&module, vec![
        "x",
        "test",
    ]);
}

#[test]
fn test_get_property() {
    use lox_bytecode::opcode::*;
    let module = compile_code("x.test;");

    assert_instructions(
        module.chunk(0),
        vec![
            GET_GLOBAL, 0, 0, 0, 0, 
            GET_PROPERTY, 1, 0, 0, 0, 
            POP, 
            RETURN_TOP
        ],
    );

    assert_numbers(&module, vec![]);
    assert_strings(&module, vec![]);

    assert_identifiers(&module, vec![
        "x",
        "test",
    ]);
}

fn make_fun(name: &str, index: usize, arity: usize) -> Closure {
    crate::bytecode::Function {
        name: name.into(),
        chunk_index: index,
        arity,
    }
    .into()
}

fn make_closure(name: &str, index: usize, arity: usize, upvalues: Vec<Upvalue>) -> Closure {
    let function = crate::bytecode::Function {
        name: name.into(),
        chunk_index: index,
        arity,
    };
    crate::bytecode::Closure { function, upvalues }
}

fn make_class(name: &str) -> Class {
    Class {
        name: name.to_string(),
    }
}

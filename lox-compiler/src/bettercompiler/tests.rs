use crate::bytecode::*;
use lox_syntax::ast::*;
use lox_syntax::position::Diagnostic;
use lox_syntax::position::WithSpan;

fn parse_stmt(data: &str) -> Result<Vec<WithSpan<Stmt>>, Vec<Diagnostic>> {
    lox_syntax::parse(data)
}

fn assert_first_chunk(data: &str, constants: Vec<Constant>, identifiers: Vec<&str>, instructions: Vec<Instruction>) {
    use super::compile;
    let ast = parse_stmt(data).unwrap();
    let module = compile(&ast).unwrap();
    let chunk = module.chunk(0);
    assert_eq!(instructions, chunk.instructions());
    assert_eq!(constants, module.constants());
    assert_eq!(identifiers, module.identifiers());
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
    assert_first_chunk(
        "print 3;",
        vec![3.0.into()],
        vec![],
        vec![
            Instruction::Constant(0),
            Instruction::Print,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "print 1+2;",
        vec![1.0.into(), 2.0.into()],
        vec![],
        vec![
            Instruction::Constant(0),
            Instruction::Constant(1),
            Instruction::Add,
            Instruction::Print,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "print 1-2;",
        vec![1.0.into(), 2.0.into()],
        vec![],
        vec![
            Instruction::Constant(0),
            Instruction::Constant(1),
            Instruction::Subtract,
            Instruction::Print,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "print nil;",
        vec![],
        vec![],
        vec![
            Instruction::Nil,
            Instruction::Print,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
}

#[test]
fn test_stmt_print_strings() {
    assert_first_chunk(
        "print \"Hello, World!\";",
        vec!["Hello, World!".into()],
        vec![],
        vec![
            Instruction::Constant(0),
            Instruction::Print,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "print \"Hello, \" + \"World!\";",
        vec!["Hello, ".into(), "World!".into()],
        vec![],
        vec![
            Instruction::Constant(0),
            Instruction::Constant(1),
            Instruction::Add,
            Instruction::Print,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
}

#[test]
fn test_global_variables() {
    use crate::bytecode::Instruction::*;
    assert_first_chunk(
        "var x=3;",
        vec![3.0.into()],
        vec!["x"],
        vec![
            Instruction::Constant(0),
            Instruction::DefineGlobal(0),
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "var x;",
        vec![],
        vec!["x"],
        vec![
            Instruction::Nil,
            Instruction::DefineGlobal(0),
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "var x=3; print x;",
        vec![3.0.into()],
        vec!["x"],
        vec![
            Instruction::Constant(0),
            Instruction::DefineGlobal(0),
            Instruction::GetGlobal(0),
            Instruction::Print,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "var x=3;x=2;",
        vec![3.0.into(), 2.0.into()],
        vec!["x"],
        vec![
            Constant(0),
            DefineGlobal(0),
            Constant(1),
            SetGlobal(0),
            Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
}

#[test]
fn test_local_variables() {
    use crate::bytecode::Instruction::*;
    assert_first_chunk(
        "{var x=3;}",
        vec![3.0.into()],
        vec![],
        vec![
            Instruction::Constant(0),
            Instruction::Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "{var x=3; print x;}",
        vec![3.0.into()],
        vec![],
        vec![
            Instruction::Constant(0),
            Instruction::GetLocal(1),
            Instruction::Print,
            Instruction::Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "var x=2; {var x=3; { var x=4; print x; } print x;} print x;",
        vec![2.0.into(), 3.0.into(), 4.0.into()],
        vec!["x"],
        vec![
            Constant(0),
            DefineGlobal(0),
            Constant(1),
            Constant(2),
            GetLocal(2),
            Print,
            Pop,
            GetLocal(1),
            Print,
            Pop,
            GetGlobal(0),
            Print,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "{var x;}",
        vec![],
        vec![],
        vec![
            Instruction::Nil,
            Instruction::Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_first_chunk(
        "{var x;x=2;}",
        vec![2.0.into()],
        vec![],
        vec![
            Nil,
            Constant(0),
            SetLocal(1),
            Pop,
            Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
}

#[test]
fn test_expression() {
    use crate::bytecode::Instruction::*;
    assert_first_chunk("3;", vec![3.0.into()], vec![],vec![Constant(0), Pop, Nil, Return]);

    assert_first_chunk("true;", vec![],  vec![],vec![True, Pop, Nil, Return]);

    assert_first_chunk("false;", vec![],  vec![],vec![False, Pop, Nil, Return]);

    assert_first_chunk("nil;", vec![],  vec![],vec![Nil, Pop, Nil, Return]);
}

#[test]
fn test_if() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "if(false) 3;4;",
        vec![3.0.into(), 4.0.into()],
        vec![],
        vec![
            False,
            JumpIfFalse(5),
            Pop,
            Constant(0),
            Pop,
            Constant(1),
            Pop,
            Nil,
            Return,
        ],
    );

    assert_first_chunk(
        "if(false) 3; else 4;5;",
        vec![3.0.into(), 4.0.into(), 5.0.into()],
        vec![],
        vec![
            False,
            JumpIfFalse(6),
            Pop,
            Constant(0),
            Pop,
            Jump(9),
            Pop,
            Constant(1),
            Pop,
            Constant(2),
            Pop,
            Nil,
            Return,
        ],
    );
}

#[test]
fn test_logical_operators() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "3 and 4;",
        vec![3.0.into(), 4.0.into()],
        vec![],
        vec![
            Constant(0),
            JumpIfFalse(4),
            Pop,
            Constant(1),
            Pop,
            Nil,
            Return,
        ],
    );

    assert_first_chunk(
        "3 or 4;",
        vec![3.0.into(), 4.0.into()],
        vec![],
        vec![
            Constant(0),
            JumpIfFalse(3),
            Jump(5),
            Pop,
            Constant(1),
            Pop,
            Nil,
            Return,
        ],
    );
}

#[test]
fn test_equality() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "3 < 4;",
        vec![3.0.into(), 4.0.into()],
        vec![],
        vec![Constant(0), Constant(1), Less, Pop, Nil, Return],
    );
}

#[test]
fn test_while() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "while(true) print 3;",
        vec![3.0.into()],
        vec![],
        vec![
            True,
            JumpIfFalse(6),
            Pop,
            Constant(0),
            Print,
            Jump(0),
            Pop,
            Nil,
            Return,
        ],
    );
}

#[test]
fn test_for() {
    use crate::bytecode::Instruction::*;

    assert_first_chunk(
        "for(var i = 0; i < 10; i = i + 1) print i;",
        vec![0.0.into(), 10.0.into(), 1.0.into()],
        vec![],
        vec![
            Constant(0),
            GetLocal(1),
            Constant(1),
            Less,
            JumpIfFalse(14),
            Pop,
            GetLocal(1),
            Print,
            GetLocal(1),
            Constant(2),
            Add,
            SetLocal(1),
            Pop,
            Jump(1),
            Pop,
            Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
}

#[test]
fn test_simple_function() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first() { print 3; } first();");

    assert_instructions(
        module.chunk(0),
        vec![
            Closure(0),
            DefineGlobal(0),
            GetGlobal(0),
            Call(0),
            Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_instructions(module.chunk(1), vec![Constant(0), Print, Nil, Return]);

    assert_constants(
        &module,
        vec![
            3.0.into(),
        ],
    );

    assert_closures(&module, vec![
        make_fun("first", 1, 0),
    ]);

    assert_identifiers(&module, vec![
        "first",
    ]);
}

#[test]
fn test_function_with_one_argument() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first(a) { print a; } first(3);");

    assert_instructions(
        module.chunk(0),
        vec![
            Closure(0),
            DefineGlobal(0),
            GetGlobal(0),
            Constant(0),
            Call(1),
            Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_instructions(module.chunk(1), vec![GetLocal(1), Print, Nil, Return]);

    assert_constants(
        &module,
        vec![
            3.0.into(),
        ],
    );

    assert_closures(&module, vec![
        make_fun("first", 1, 1),
    ]);

    assert_identifiers(&module, vec![
        "first"
    ]);
}

#[test]
fn test_recursive_function_with_one_argument() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first(a) { print first(a+1); } first(3);");

    assert_instructions(
        module.chunk(0),
        vec![
            Closure(0),
            DefineGlobal(0),
            GetGlobal(0),
            Constant(1),
            Call(1),
            Pop,
            Nil,
            Return,
        ],
    );
    assert_instructions(
        module.chunk(1),
        vec![
            GetGlobal(0),
            GetLocal(1),
            Constant(0),
            Add,
            Call(1),
            Print,
            Nil,
            Return,
        ],
    );

    assert_constants(
        &module,
        vec![
            1.0.into(),
            3.0.into(),
        ],
    );

    assert_closures(&module, vec![
        make_fun("first", 1, 1),
    ]);

    assert_identifiers(&module, vec![
        "first"
    ]);
}

#[test]
fn test_functions_calling_functions() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first() { second(); } fun second() { print 3; } first();");

    assert_instructions(
        module.chunk(0),
        vec![
            Closure(0),
            DefineGlobal(1),
            Closure(1),
            DefineGlobal(0),
            GetGlobal(1),
            Call(0),
            Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_instructions(
        module.chunk(1),
        vec![GetGlobal(0), Call(0), Pop, Nil, Return],
    );
    assert_instructions(module.chunk(2), vec![Constant(0), Print, Nil, Return]);

    assert_constants(
        &module,
        vec![
            3.0.into(),
        ],
    );

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
    use crate::bytecode::Instruction::*;

    let module = compile_code("{ fun first() { print 3; } first(); }");

    assert_instructions(
        module.chunk(0),
        vec![
            Closure(0),
            GetLocal(1),
            Call(0),
            Pop,
            Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_instructions(module.chunk(1), vec![Constant(0), Print, Nil, Return]);

    assert_constants(&module, vec![3.0.into()]);

    assert_closures(&module, vec![
        make_fun("first", 1, 0),
    ]);
}

#[test]
fn test_simple_scoped_recursive_function() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{ fun first() { print first(); } first(); }");

    assert_instructions(
        module.chunk(0),
        vec![
            Closure(0),
            GetLocal(1),
            Call(0),
            Pop,
            CloseUpvalue,
            Nil,
            Return,
        ],
    );
    assert_instructions(
        module.chunk(1),
        vec![GetUpvalue(0), Call(0), Print, Nil, Return],
    );

    assert_closures(&module, vec![
        make_closure("first", 1, 0, vec![Upvalue::Local(1)]),
    ]);
}

#[test]
fn test_function_with_return() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("fun first() { return 3; }");

    assert_instructions(
        module.chunk(0),
        vec![
            Closure(0),
            DefineGlobal(0),
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_instructions(module.chunk(1), vec![Constant(0), Return, Nil, Return]);

    assert_constants(
        &module,
        vec![3.0.into()],
    );

    assert_closures(&module, vec![
        make_fun("first", 1, 0),
    ]);

    assert_identifiers(&module, vec![
        "first"
    ]);
}

#[test]
fn test_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{var a = 3; fun f() { print a; }}");

    assert_instructions(
        module.chunk(0),
        vec![Constant(0), Closure(0), Pop, CloseUpvalue, Nil, Return],
    );
    assert_instructions(module.chunk(1), vec![GetUpvalue(0), Print, Nil, Return]);

    assert_constants(
        &module,
        vec![3.0.into()],
    );

    assert_closures(&module, vec![
        make_closure("f", 1, 0, vec![Upvalue::Local(1)])
    ]);
}

#[test]
fn test_double_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{var a = 3; fun f() { fun g() { print a; } }}");

    assert_instructions(
        module.chunk(0),
        vec![Constant(0), Closure(1), Pop, CloseUpvalue, Nil, Return],
    );
    assert_instructions(module.chunk(1), vec![Closure(0), Nil, Return]);
    assert_instructions(module.chunk(2), vec![GetUpvalue(0), Print, Nil, Return]);

    assert_constants(
        &module,
        vec![
            3.0.into(),
        ],
    );

    assert_closures(&module, vec![
        make_closure("g", 2, 0, vec![Upvalue::Upvalue(0)]),
        make_closure("f", 1, 0, vec![Upvalue::Local(1)]),
    ]);
}

#[test]
fn test_multiple_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{var a = 3; var b = 4; fun f() {print b; print a; }}");

    assert_instructions(
        module.chunk(0),
        vec![
            Constant(0),
            Constant(1),
            Closure(0),
            Pop,
            CloseUpvalue,
            CloseUpvalue,
            Nil,
            Return,
        ],
    );
    assert_instructions(
        module.chunk(1),
        vec![GetUpvalue(0), Print, GetUpvalue(1), Print, Nil, Return],
    );

    assert_constants(
        &module,
        vec![
            3.0.into(),
            4.0.into(),
        ],
    );

    assert_closures(&module, vec![
        make_closure("f", 1, 0, vec![Upvalue::Local(2), Upvalue::Local(1)]),
    ]);
}

#[test]
fn test_multiple_double_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{var a = 3; var b = 4; fun f() { fun g() { print a; print b; }}}");

    assert_instructions(
        module.chunk(0),
        vec![
            Constant(0),
            Constant(1),
            Closure(1),
            Pop,
            CloseUpvalue,
            CloseUpvalue,
            Nil,
            Return,
        ],
    );
    assert_instructions(module.chunk(1), vec![Closure(0), Nil, Return]);
    assert_instructions(
        module.chunk(2),
        vec![GetUpvalue(0), Print, GetUpvalue(1), Print, Nil, Return],
    );

    assert_constants(
        &module,
        vec![
            3.0.into(),
            4.0.into(),
        ],
    );

    assert_closures(&module, vec![
        make_closure("g", 2, 0, vec![Upvalue::Upvalue(0), Upvalue::Upvalue(1)]),
        make_closure("f", 1, 0, vec![Upvalue::Local(1), Upvalue::Local(2)]),
    ]);
}

#[test]
fn test_scoped_upvalue() {
    use crate::bytecode::Instruction::*;

    let module = compile_code(
        "var global; fun main() { { var a = 3; fun one() { print a; } global = one; } } main();",
    );

    assert_instructions(
        module.chunk(0),
        vec![
            Nil,
            DefineGlobal(0),
            Closure(1),
            DefineGlobal(1),
            GetGlobal(1),
            Call(0),
            Pop,
            Instruction::Nil,
            Instruction::Return,
        ],
    );
    assert_instructions(
        module.chunk(1),
        vec![
            Constant(0),
            Closure(0),
            GetLocal(2),
            SetGlobal(0),
            Pop,
            Pop,
            CloseUpvalue,
            Nil,
            Return,
        ],
    );
    assert_instructions(module.chunk(2), vec![GetUpvalue(0), Print, Nil, Return]);

    assert_constants(
        &module,
        vec![
            3.0.into(),
        ],
    );

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
    use crate::bytecode::Instruction::*;

    let module = compile_code("import \"foo\";");

    assert_instructions(
        module.chunk(0),
        vec![Import(0), Pop, Nil, Return],
    );

    assert_constants(&module, vec!["foo".into()]);
}

#[test]
fn test_complex_import() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("import \"foo\" for x;");

    assert_instructions(
        module.chunk(0),
        vec![Import(0), ImportGlobal(0), DefineGlobal(0), Pop, Nil, Return],
    );

    assert_constants(&module, vec!["foo".into()]);

    assert_identifiers(&module, vec![
        "x"
    ]);
}

#[test]
fn test_complex_local_import() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{import \"foo\" for x; print x;}");

    assert_instructions(
        module.chunk(0),
        vec![Import(0), ImportGlobal(0), Pop, GetLocal(1), Print, Pop, Nil, Return],
    );

    assert_constants(&module, vec!["foo".into()]);

    assert_identifiers(&module, vec![
        "x"
    ]);
}

#[test]
fn test_empty_class_global() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("class Foo {}");

    assert_instructions(
        module.chunk(0),
        vec![Class(0), DefineGlobal(0), GetGlobal(0), Pop, Nil, Return],
    );

    assert_constants(&module, vec![]);

    assert_classes(&module, vec![
        make_class("Foo")
    ]);

    assert_identifiers(&module, vec![
        "Foo"
    ]);
}

#[test]
fn test_empty_class_local() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("{class Foo {}}");

    assert_instructions(module.chunk(0), vec![Class(0), GetLocal(1), Pop, Pop, Nil, Return]);

    assert_classes(&module, vec![
        make_class("Foo")
    ]);
}

#[test]
fn test_set_property() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("x.test = 3;");

    assert_instructions(
        module.chunk(0),
        vec![GetGlobal(0), Constant(0), SetProperty(1), Pop, Nil, Return],
    );

    assert_constants(&module, vec![3.0.into()]);

    assert_identifiers(&module, vec![
        "x",
        "test",
    ]);
}

#[test]
fn test_get_property() {
    use crate::bytecode::Instruction::*;

    let module = compile_code("x.test;");

    assert_instructions(
        module.chunk(0),
        vec![GetGlobal(0), GetProperty(1), Pop, Nil, Return],
    );

    assert_constants(&module, vec![]);

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

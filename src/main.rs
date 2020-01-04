use lox;

fn main() {
    // let data = "print 3;";
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
    // let data = "if(false) print 3; else print 4; print 5;";
    // let data = "var i =0; while(i < 1000) { print i; i = i + 1; }";
    // let data = "for(var i =0; i < 1000; i = i + 1) print i;";
    // let data = "fun first() { print 3; } print first;";
    // let data = "fun first() { print 3; } first();";
    // let data = "fun first(a) { print a; } first(3);";
    // let data = "fun first(a) { print a; if(a < 1000) first(a+1); } first(3);";
    // let data = "fun first() { return 3;} print first();";
    // let data = "fun first(a) { return a + 3; } print first(5); print first(1);";
    // let data = "print -1;";
    // let data = "fun first(a) { print a + 3; } first(-1);";
    // let data = "print !false";
    // let data = "fun first(a) { if(!a) print \"is falsey\"; else print a; } first(3); first(false); first(nil); first(true);";
    // let data = "fun first() {} first(1);";
    // let data = "test(1); test(2,3);";
    // let data = "var a = clock(); for(var i =0; i < 10000; i = i + 1) {} print clock()-a;";

    // closures
    // let data = "{ var a = 3; fun first() { print a; } first(); }";
    // let data = "fun outer() {var x = 3; fun inner() { print x; } return inner; } var closure = outer(); closure();";
    // let data = "var global; fun main() { { var a = 3; fun one() { print a; } global = one; } } main(); global();";
    // let data = "{ var a = 3; fun first() { print a; } fun second() { print a; } }";
    // let data = "
    // var globalSet;
    // var globalGet;
    
    // fun main() {
    //   var a = \"initial\";
    
    //   fun set() { a = \"updated\"; }
    //   fun get() { print a; }
    
    //   globalSet = set;
    //   globalGet = get;
    // }
    
    // main();
    // globalSet();
    // globalGet();
    // ";

    let data = std::fs::read_to_string("test.lox").unwrap();

    let module = lox::compile(&data).unwrap();

    println!("constants: {:?}", module.constants());
    for chunk in module.chunks() {
        println!("chunk: {:?}", chunk.instructions());
    }

    println!();

    lox::bettervm::execute(&module).unwrap();
}
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

#[derive(PartialEq, Debug)]
enum TestResult {
    Ok,
    CompileError,
    RuntimeError,
}

fn execute(source: &str) -> TestResult {
    let module = match lox_compiler::compile(source) {
        Ok(module) => module,
        Err(_) => return TestResult::CompileError,
    };

    let mut vm = lox_vm::VirtualMachine::new();
    lox_std::set_stdlib(&mut vm);
    let result = match vm.interpret(module) {
        Ok(_) => TestResult::Ok,
        Err(err) => {
            println!("Runtime error: {:?}", err);
            TestResult::RuntimeError
        },
    };

    result
}

fn criterion_benchmark(c: &mut Criterion) {
    let zoo = include_str!("zoo.lox");
    let trees = include_str!("trees.lox");
    let fib = include_str!("fib.lox");

    c.bench_with_input(
        BenchmarkId::new("run", "zoo"),
        &zoo,
        |b, s| { b.iter(|| { execute(s); }); }
    );

    c.bench_with_input(
        BenchmarkId::new("run", "trees"),
        &trees,
        |b, s| { b.iter(|| { execute(s); }); }
    );

    c.bench_with_input(
        BenchmarkId::new("run", "fib"),
        &fib,
        |b, s| { b.iter(|| { execute(s); }); }
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);


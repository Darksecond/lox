pub mod memory;
pub mod interner;
pub mod value;

mod runtime;
mod stack;
mod ops;
mod fiber;
mod table;

//TODO Move to lox-gc
mod array;
pub mod string;

use lox_bytecode::bytecode::Module;
use runtime::{Runtime, Mode};
use interner::{Symbol, intern};
use memory::{Import, NativeFunction, Class};
use value::Value;
use lox_gc::{Gc, Trace};

pub use runtime::VmError;

pub struct VirtualMachine {
    runtime: Runtime,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    pub fn set_stdout(&mut self, print: for<'r> fn(&'r str)) {
        self.runtime.print = print;
    }

    pub fn set_import(&mut self, import: for<'r> fn(&'r str) -> Option<Module>) {
        self.runtime.import = import;
    }

    pub fn interpret(&mut self, module: Module) -> Result<(), VmError> {
        self.runtime.with_module(module);
        self.runtime.interpret(Mode::Continuous)
    }

    pub fn native(&mut self) -> Native {
        Native {
            runtime: &mut self.runtime,
        }
    }
}

pub struct Native<'a> {
    runtime: &'a mut Runtime,
}

impl Native<'_> {
    pub fn intern(&mut self, value: &str) -> Symbol {
        intern(value)
    }

    pub fn manage<T: 'static + Trace>(&self, value: T) -> Gc<T> {
        lox_gc::manage(value)
    }

    pub fn build_fn(&self, identifier: &str, code: fn(Native, Value, &[Value]) -> Value) -> Gc<NativeFunction> {
        lox_gc::manage((NativeFunction {
            name: identifier.into(),
            code,
        }).into())
    }

    pub fn set_fn(&mut self, import: Gc<Import>, identifier: &str, code: fn(Native, Value, &[Value]) -> Value) {
        let root = self.build_fn(identifier, code);
        let identifier = intern(identifier);
        import.set_global(identifier, Value::from_object(root.erase()))
    }

    pub fn set_method(&mut self, class: Gc<Class>, identifier: &str, code: fn(Native, Value, &[Value]) -> Value) {
        let root = self.build_fn(identifier, code);
        let identifier = intern(identifier);
        class.set_method(identifier, Value::from_object(root.erase()));
    }

    pub fn set_global_fn(&mut self, identifier: &str, code: fn(Native, Value, &[Value]) -> Value) {
        self.set_fn(self.global_import(), identifier, code)
    }

    pub fn global_import(&self) -> Gc<Import> {
        self.runtime.globals_import()
    }

    pub fn list_class(&self) -> Gc<Class> {
        self.runtime.builtins.list_class
    }

    pub fn string_class(&self) -> Gc<Class> {
        self.runtime.builtins.string_class
    }

    pub fn add_import(&mut self, import: Gc<Import>) {
        self.runtime.imports.insert(import.name.clone(), import);
    }

    pub fn call(&mut self, callee: Gc<()>, args: &[Value]) -> Value {
        let callee = Value::from_object(callee);

        self.runtime.fiber.with_stack(|stack| {
            stack.push(callee);

            //TODO maybe reverse?
            for arg in args {
                stack.push(*arg);
            }
        });

        let depth = self.runtime.fiber.frame_depth();

        match self.runtime.call(args.len(), callee) {
            runtime::Signal::More => {
                //TODO Handle runtime errors
                self.runtime.interpret(Mode::Function(depth)).expect("Runtime error");
            },
            runtime::Signal::Return => (),
            runtime::Signal::RuntimeError => panic!("runtime error!"),
            _ => panic!("unexpected signal!"),
        }

        self.runtime.fiber.with_stack(|stack| {
            stack.pop()
        })
    }

    pub fn get_global(&self, identifier: Symbol) -> Option<Value> {
        let current_import = self.runtime.fiber.current_import();
        let value = current_import.global(identifier);
        value
    }
}

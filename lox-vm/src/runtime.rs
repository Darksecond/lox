mod builtins;

use lox_bytecode::bytecode::Module;
use crate::value::Value;
use builtins::Builtins;

use super::memory::*;
use super::interner::{Symbol, Interner};
use lox_gc::{Gc, Trace, Tracer};
use crate::fiber::Fiber;
use crate::string::LoxString;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Signal {
    Done,
    More,
    RuntimeError,
}

//TODO thiserror
//TODO RuntimeError
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VmError {
    Unknown,
    StackEmpty,
    FrameEmpty,
    StringConstantExpected,
    GlobalNotDefined,
    InvalidCallee,
    IncorrectArity,
    UnexpectedConstant,
    ClosureConstantExpected,
    UnexpectedValue,
    UndefinedProperty,
    Unimplemented,
    UnknownImport,
    IndexOutOfRange,
}

pub struct Runtime {
    pub fiber: Fiber,
    init_symbol: Symbol, //TODO Move to builtins
    pub interner: Interner,
    pub imports: HashMap<LoxString, Gc<Import>>,

    pub builtins: Builtins,

    // Env
    pub print: for<'r> fn(&'r str),
    pub import: for<'r> fn(&'r str) -> Option<Module>,

    ip: *const u8,
}

unsafe impl Trace for Runtime {
    #[inline]
    fn trace(&self, tracer: &mut Tracer) {
        self.fiber.trace(tracer);
        self.imports.trace(tracer);
        self.builtins.trace(tracer);
    }
}

pub fn default_print(value: &str) {
    println!("{}", value);
}

pub fn default_import(_value: &str) -> Option<Module> {
    None
}

impl Runtime {
    pub fn new() -> Self {
        let mut interner = Interner::new();
        let builtins = Builtins::new();

        Self {
            fiber: Fiber::new(),
            init_symbol: interner.intern("init"),
            interner,
            imports: HashMap::new(),
            print: default_print,
            import: default_import,

            builtins,

            ip: std::ptr::null(),
        }
    }

    #[cold]
    pub fn concat(&mut self, a: Value, b: Value) -> Signal {
        if a.is_object_of_type::<LoxString>() && b.is_object_of_type::<LoxString>() {
            let a = a.as_object().cast::<LoxString>();
            let b = b.as_object().cast::<LoxString>();
            self.push_string(format!("{}{}", a.as_str(), b.as_str()));
            return Signal::More;
        }

        self.fiber.runtime_error(VmError::UnexpectedValue)
    }

    pub fn print(&self, value: &str) {
        (self.print)(value);
    }

    pub fn with_module(&mut self, module: Module) {
        let closure = self.prepare_interpret(module);
        self.fiber.stack.push(Value::from_object(closure.erase()));
        self.fiber.begin_frame(closure);
        self.load_ip();
    }

    #[cold]
    pub fn manage<T: Trace + 'static>(&self, data: T) -> Gc<T> {
        lox_gc::collect(&[self, &data]);
        lox_gc::manage(data)
    }

    //TODO Use name given instead of _root
    fn prepare_interpret(&mut self, module: Module) -> Gc<Closure> {
        let import = Import::with_module("_root", module, &mut self.interner);
        let import: Gc<Import> = self.manage(import.into());
        lox_gc::finalize(import);
        self.imports.insert(import.name.clone(), import);
        self.globals_import().copy_to(&import);

        self.manage(Closure::with_import(import).into())
    }

    pub fn import(&mut self, path: &str) -> Option<Gc<Import>> {
        if let Some(import) = self.imports.get(path) {
            Some(*import)
        } else {
            None
        }
    }

    pub fn load_import(&mut self, path: &str) -> Result<Gc<Import>, VmError> {
        let module = (self.import)(path);
        if let Some(module) = module {
            let import = Import::with_module(path, module, &mut self.interner);
            let import = self.manage(import.into());
            lox_gc::finalize(import);
            self.imports.insert(path.into(), import);
            self.globals_import().copy_to(&import);

            Ok(import)
        } else {
            Err(VmError::UnknownImport)
        }
    }

    pub fn globals_import(&self) -> Gc<Import> {
        self.builtins.globals_import
    }

    pub fn call(&mut self, arity: usize, callee: Value) -> Signal {
        if !callee.is_object() {
            return self.fiber.runtime_error(VmError::InvalidCallee);
        }

        let callee = callee.as_object();

        if let Some(callee) = callee.try_cast::<Closure>() {
            return self.call_closure(arity, callee);
        } else if let Some(callee) = callee.try_cast::<NativeFunction>() {
            return self.call_native_function(arity, callee);
        } else if let Some(class) = callee.try_cast::<Class>() {
            return self.call_class(arity, class);
        } else if let Some(bind) = callee.try_cast::<BoundMethod>() {
            return self.call_bound_method(arity, bind);
        } else {
            return self.fiber.runtime_error(VmError::InvalidCallee)
        }
    }

    pub fn call_closure(&mut self, arity: usize, callee: Gc<Closure>) -> Signal {
        self.store_ip();

        if callee.function.arity != arity {
            return self.fiber.runtime_error(VmError::IncorrectArity);
        }
        self.fiber.begin_frame(callee);

        self.load_ip();

        Signal::More
    }

    pub fn call_native_function(&mut self, arity: usize, callee: Gc<NativeFunction>) -> Signal {
        self.store_ip();

        let args = self.fiber.stack.pop_n(arity);
        let this = self.fiber.stack.pop(); // discard callee
        let result = (callee.code)(this, &args);
        self.fiber.stack.push(result);

        self.load_ip();

        Signal::More
    }

    pub fn call_class(&mut self, arity: usize, class: Gc<Class>) -> Signal {
        self.store_ip();

        let instance: Gc<Instance> = self.manage(Instance::new(class).into());
        self.fiber.stack.rset(arity, Value::from_object(instance.erase()));

        if let Some(initializer) = class.method(self.init_symbol) {
            if !initializer.is_object() {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            }

            let initializer = initializer.as_object();

            if let Some(initializer) = initializer.try_cast::<Closure>() {
                if initializer.function.arity != arity {
                    return self.fiber.runtime_error(VmError::IncorrectArity);
                }
                self.fiber.begin_frame(initializer);
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            }
        } else if arity != 0 {
            // Arity must be 0 without initializer
            return self.fiber.runtime_error(VmError::IncorrectArity);
        }

        self.load_ip();

        Signal::More
    }

    pub fn call_bound_method(&mut self, arity: usize, bind: Gc<BoundMethod>) -> Signal {
        self.store_ip();

            self.fiber.stack.rset(arity, Value::from_object(bind.receiver));

        return self.call(arity, bind.method);
    }

    #[cold]
    pub fn push_string(&mut self, string: impl Into<LoxString>) {
        let root: Gc<LoxString> = self.manage(string.into());
        self.fiber.stack.push(Value::from_object(root.erase()));
    }


    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        unsafe {
            let slice = &*std::ptr::slice_from_raw_parts(self.ip, 4);
            let value = u32::from_le_bytes(slice.try_into().unwrap());
            self.ip = self.ip.add(4);
            value
        }
    }

    #[inline]
    pub fn next_i16(&mut self) -> i16 {
        unsafe {
            let slice = &*std::ptr::slice_from_raw_parts(self.ip, 2);
            let value = i16::from_le_bytes(slice.try_into().unwrap());
            self.ip = self.ip.add(2);
            value
        }
    }

    #[inline]
    pub fn next_u16(&mut self) -> u16 {
        unsafe {
            let slice = &*std::ptr::slice_from_raw_parts(self.ip, 2);
            let value = u16::from_le_bytes(slice.try_into().unwrap());
            self.ip = self.ip.add(2);
            value
        }
    }

    #[inline]
    pub fn next_u8(&mut self) -> u8 {
        unsafe {
            //let value = std::ptr::read(self.ip);
            let value = *self.ip;
            self.ip = self.ip.add(1);
            value
        }
    }

    #[inline]
    pub fn store_ip(&self) {
        let ip = self.ip;
        self.fiber.current_frame().store_ip(ip);
    }

    #[inline]
    pub fn load_ip(&mut self) {
        self.ip = self.fiber.current_frame().load_ip();
    }

    #[inline]
    pub fn set_ip(&mut self, to: i16) {
        unsafe {
            self.ip = self.ip.offset(to as isize);
        }
    }
}

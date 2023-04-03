mod builtins;

use lox_bytecode::bytecode::Module;
use crate::value::Value;
use super::memory::*;
use super::interner::{Symbol, Interner};
use super::gc::{Gc, Trace, Heap};
use crate::fiber::Fiber;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Signal {
    Done,
    More,
    RuntimeError,
    ContextSwitch,
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
    pub fiber: Gc<Fiber>,
    next_fiber: Option<Gc<Fiber>>,
    init_symbol: Symbol,
    pub interner: Interner,
    imports: HashMap<String, Gc<Object<Import>>>,
    pub heap: Heap,

    // Env
    pub print: for<'r> fn(&'r str),
    pub import: for<'r> fn(&'r str) -> Option<Module>,

    ip: *const u8,
}

impl Trace for Runtime {
    #[inline]
    fn trace(&self) {
        self.fiber.trace();
        self.next_fiber.trace();
        self.imports.trace();
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
        let heap = Heap::new();
        let fiber = heap.manage(Fiber::new(None));
        let mut imports = HashMap::new();

        let globals = heap.manage(Import::new().into());
        imports.insert("_globals".to_string(), globals);

        Self {
            fiber,
            next_fiber: None,
            init_symbol: interner.intern("init"),
            interner,
            heap,
            imports,
            print: default_print,
            import: default_import,

            ip: std::ptr::null(),
        }
    }

    #[inline(never)]
    pub fn context_switch(&mut self) {
        if let Some(next_fiber) = self.next_fiber {
            if self.fiber.has_current_frame() {
                self.store_ip();
            }
            self.fiber = next_fiber;
            self.load_ip();
        }

        self.next_fiber = None;
    }

    pub fn switch_to(&mut self, fiber: Gc<Fiber>) -> Signal {
        self.next_fiber = Some(fiber);
        Signal::ContextSwitch
    }

    pub fn concat(&self, a: Value, b: Value) -> Signal {
        if a.is_object() && b.is_object() && a.as_object().tag == ObjectTag::String && b.as_object().tag == ObjectTag::String {
            let a = a.as_object().as_string();
            let b = b.as_object().as_string();
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
        self.fiber.with_stack(|stack| {
            stack.push(Value::from_object(closure));
        });
        self.fiber.begin_frame(closure);
        self.load_ip();
    }

    pub fn adjust_size<T: 'static + Trace>(&mut self, object: Gc<T>) {
        self.heap.adjust_size(object);
    }

    pub fn manage<T: Trace>(&self, data: T) -> Gc<T> {
        self.heap.collect(&[self]);
        self.heap.manage(data)
    }

    //TODO Use name given instead of _root
    fn prepare_interpret(&mut self, module: Module) -> Gc<Object<Closure>> {
        let import = Import::with_module(module, &mut self.interner, &self.heap);
        let import = self.manage(import.into());
        self.imports.insert("_root".into(), import);
        self.globals_import().copy_to(&import);

        let function = Function {
            arity: 0,
            chunk_index: 0,
            name: "top".into(),
            import,
        };

        self.manage(Closure {
            upvalues: vec![],
            function,
        }.into())
    }

    pub fn import(&mut self, path: &str) -> Option<Gc<Object<Import>>> {
        if let Some(import) = self.imports.get(path) {
            Some(*import)
        } else {
            None
        }
    }

    pub fn load_import(&mut self, path: &str) -> Result<Gc<Object<Import>>, VmError> {
        let module = (self.import)(path);
        if let Some(module) = module {
            let import = Import::with_module(module, &mut self.interner, &self.heap);
            let import = self.manage(import.into());
            self.imports.insert(path.into(), import);
            self.globals_import().copy_to(&import);

            Ok(import)
        } else {
            Err(VmError::UnknownImport)
        }
    }

    #[inline]
    pub fn current_import(&self) -> Gc<Object<Import>> {
        self.fiber.current_frame().closure.function.import
    }

    pub fn globals_import(&self) -> Gc<Object<Import>> {
        *self.imports.get("_globals").expect("Could not find globals import")
    }

    //TODO Reduce duplicate code paths
    #[inline(never)]
    pub fn call(&mut self, arity: usize, callee: Value) -> Signal {
        self.store_ip();

        if !callee.is_object() {
            return self.fiber.runtime_error(VmError::InvalidCallee);
        }

        let callee = callee.as_object();

        match callee.tag {
            ObjectTag::Closure => {
                let callee = callee.as_closure();
                if callee.function.arity != arity {
                    return self.fiber.runtime_error(VmError::IncorrectArity);
                }
                self.fiber.begin_frame(callee);
            }
            ObjectTag::NativeFunction => {
                let callee = callee.as_native_function();
                self.fiber.with_stack(|stack| {
                    let args = stack.pop_n(arity);
                    let _this = stack.pop(); // discard callee
                    let result = (callee.code)(&args);
                    stack.push(result);
                });
            }
            ObjectTag::Class => {
                let class = callee.as_class();
                let instance: Gc<Object<Instance>> = self.manage(Instance::new(class).into());
                self.fiber.with_stack(|stack| {
                    stack.rset(arity, Value::from_object(instance));
                });

                if let Some(initializer) = class.method(self.init_symbol) {
                    if !initializer.is_object() {
                        return self.fiber.runtime_error(VmError::UnexpectedValue);
                    }

                    let initializer = initializer.as_object();

                    if initializer.tag == ObjectTag::Closure {
                        let initializer = initializer.as_closure();
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
            }
            ObjectTag::BoundMethod => {
                let bind = callee.as_bound_method();
                self.fiber.with_stack(|stack| {
                    stack.rset(arity, Value::from_object(bind.receiver));
                });
                return self.call(arity, bind.method);

            },
            _ => return self.fiber.runtime_error(VmError::InvalidCallee),
        }

        self.load_ip();

        Signal::More
    }

    pub fn push_string(&self, string: impl Into<String>) {
        let string = string.into();
        let root: Gc<Object<String>> = self.manage(string.into());
        self.fiber.with_stack(|stack| {
            stack.push(Value::from_object(root));
        });
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

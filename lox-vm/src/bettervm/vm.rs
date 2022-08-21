use lox_bytecode::bytecode::Module;
use lox_bytecode::opcode;
use super::memory::*;
use super::interner::{Symbol, Interner};
use crate::bettergc::{Gc, Trace, Heap};
use crate::bettervm::fiber::Thread;
use std::collections::HashMap;
use std::cell::RefCell;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Signal {
    Done,
    More,
    RuntimeError,
}

//TODO thiserror
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VmError {
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
}

pub struct Fiber {
    pub fiber: Thread,
    init_symbol: Symbol,
    pub print: for<'r> fn(&'r str),
    interner: Interner,
    imports: HashMap<String, Gc<Import>>,
    heap: RefCell<Heap>,

    ip: *const u8,
}

impl Trace for Fiber {
    #[inline]
    fn trace(&self) {
        self.fiber.trace();
        self.imports.trace();
    }
}

impl Fiber {
    pub fn new(print: for<'r> fn(&'r str)) -> Self {
        let mut interner = Interner::new();
        Self {
            fiber: Thread::new(),
            init_symbol: interner.intern("init"),
            interner,
            heap: RefCell::new(Heap::new()),
            imports: HashMap::new(),
            print,

            ip: std::ptr::null(),
        }
    }

    pub fn with_module(&mut self, module: Module) {
        let closure = self.prepare_interpret(module);
        self.fiber.stack.push(Value::Closure(closure));
        self.fiber.begin_frame(closure);
        self.load_ip();
    }

    pub fn manage<T: Trace>(&mut self, data: T) -> Gc<T> {
        let mut heap = self.heap.borrow_mut();
        heap.collect(&[self]);
        heap.manage(data)
    }

    pub fn intern(&mut self, string: &str) -> Symbol {
        self.interner.intern(string)
    }

    //TODO Use name given instead of _root
    fn prepare_interpret(&mut self, module: Module) -> Gc<Closure> {
        let import = Import::new(module, &mut self.interner);
        let import = self.manage(import);
        self.imports.insert("_root".into(), import);

        let function = Function {
            arity: 0,
            chunk_index: 0,
            name: "top".into(),
            import,
        };

        self.manage(Closure {
            upvalues: vec![],
            function,
        })
    }


    pub fn current_import(&self) -> Gc<Import> {
        self.fiber.current_frame().closure.function.import
    }

    pub fn set_native_fn(&mut self, identifier: &str, code: fn(&[Value]) -> Value) {
        let native_function = NativeFunction {
            name: identifier.to_string(),
            code,
        };

        let identifier = self.interner.intern(identifier);
        let root = self.manage(native_function);
        self.current_import().set_global(identifier, Value::NativeFunction(root))
    }

    pub fn interpret(&mut self) -> Result<(), VmError> {
        loop {
            let result = match self.next_u8() {
                opcode::CONSTANT      => self.op_constant(),
                opcode::IMPORT        => self.op_import(),
                opcode::IMPORT_GLOBAL => self.op_import_global(),
                opcode::CLOSURE       => self.op_closure(),
                opcode::CLASS         => self.op_class(),
                opcode::METHOD        => self.op_method(),
                opcode::SET_PROPERTY  => self.op_set_property(),
                opcode::GET_PROPERTY  => self.op_get_property(),
                opcode::PRINT         => self.op_print(),
                opcode::NIL           => self.op_nil(),
                opcode::RETURN        => self.op_return(),
                opcode::ADD           => self.op_add(),
                opcode::SUBTRACT      => self.op_subtract(),
                opcode::MULTIPLY      => self.op_multiply(),
                opcode::DIVIDE        => self.op_divide(),
                opcode::POP           => self.op_pop(),
                opcode::DEFINE_GLOBAL => self.op_define_global(),
                opcode::GET_GLOBAL    => self.op_get_global(),
                opcode::SET_GLOBAL    => self.op_set_global(),
                opcode::GET_LOCAL     => self.op_get_local(),
                opcode::SET_LOCAL     => self.op_set_local(),
                opcode::TRUE          => self.op_bool(true),
                opcode::FALSE         => self.op_bool(false),
                opcode::JUMP_IF_FALSE => self.op_jump_if_false(),
                opcode::JUMP          => self.op_jump(),
                opcode::LESS          => self.op_less(),
                opcode::GREATER       => self.op_greater(),
                opcode::EQUAL         => self.op_equal(),
                opcode::CALL          => self.op_call(),
                opcode::NEGATE        => self.op_negate(),
                opcode::NOT           => self.op_not(),
                opcode::GET_UPVALUE   => self.op_get_upvalue(),
                opcode::SET_UPVALUE   => self.op_set_upvalue(),
                opcode::CLOSE_UPVALUE => self.op_close_upvalue(),
                opcode::INVOKE        => self.op_invoke(),
                _ => unreachable!(),
            };

            match result {
                Signal::Done => return Ok(()),
                Signal::More => (),
                Signal::RuntimeError => {
                    return Err(self.fiber.error.unwrap());
                },
            }
        }
    }

    //TODO Reduce duplicate code paths
    pub fn call(&mut self, arity: usize, callee: Value) -> Signal {
        self.store_ip();

        match callee {
            Value::Closure(callee) => {
                if callee.function.arity != arity {
                    return self.fiber.runtime_error(VmError::IncorrectArity);
                }
                self.fiber.begin_frame(callee);
            }
            Value::NativeFunction(callee) => {
                let args = self.fiber.stack.pop_n(arity);
                self.fiber.stack.pop(); // discard callee
                let result = (callee.code)(&args);
                self.fiber.stack.push(result);
            }
            Value::Class(class) => {
                let instance = self.manage(Instance::new(class));
                self.fiber.stack.rset(arity, Value::Instance(instance));

                if let Some(initializer) = class.method(self.init_symbol) {
                    if initializer.function.arity != arity {
                        return self.fiber.runtime_error(VmError::IncorrectArity);
                    }
                    self.fiber.begin_frame(initializer);
                } else if arity != 0 {
                    // Arity must be 0 without initializer
                    return self.fiber.runtime_error(VmError::IncorrectArity);
                }
            }
            Value::BoundMethod(bind) => {
                let callee = bind.method;
                if callee.function.arity != arity {
                    return self.fiber.runtime_error(VmError::IncorrectArity);
                }
                self.fiber.stack.rset(arity, Value::Instance(bind.receiver));
                self.fiber.begin_frame(callee);

            },
            _ => return self.fiber.runtime_error(VmError::InvalidCallee),
        }

        self.load_ip();

        Signal::More
    }

    pub fn push_string(&mut self, string: impl Into<String>) {
        let root = self.manage(string.into());
        self.fiber.stack.push(Value::String(root));
    }


    pub fn next_u32(&mut self) -> u32 {
        unsafe {
            let slice = &*std::ptr::slice_from_raw_parts(self.ip, 4);
            let value = u32::from_le_bytes(slice.try_into().unwrap());
            self.ip = self.ip.add(4);
            value
        }
    }

    pub fn next_u8(&mut self) -> u8 {
        unsafe {
            let value = std::ptr::read(self.ip);
            self.ip = self.ip.add(1);
            value
        }
    }

    pub fn store_ip(&mut self) {
        self.fiber.current_frame_mut().store_ip(self.ip);
    }

    pub fn load_ip(&mut self) {
        self.ip = self.fiber.current_frame().load_ip();
    }

    pub fn set_ip(&mut self, to: usize) {
        self.fiber.current_frame_mut().set_pc(to);
        self.load_ip();
    }
}

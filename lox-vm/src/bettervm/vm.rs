use lox_bytecode::bytecode::{Chunk, Instruction};

use super::memory::*;
use crate::bettergc::{Gc, Trace, UniqueRoot, Heap};
use crate::bytecode::Module;
use std::{cell::RefCell, io::{Stdout, Write, stdout}};
use std::collections::HashMap;
use fxhash::FxHashMap;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Symbol(u32);

impl Symbol {
    pub const fn invalid() -> Self {
        Self(0)
    }
}

pub struct Interner {
    next: u32,
    map: HashMap<String, Symbol>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            next: 1,
            map: HashMap::new(),
        }
    }

    pub fn intern(&mut self, string: &str) -> Symbol {
        if let Some(symbol) = self.map.get(string) {
            *symbol
        } else {
            let symbol = Symbol(self.next);
            self.next += 1;
            self.map.insert(string.to_string(), symbol);
            symbol
        }
    }
}

#[derive(PartialEq)]
enum InterpretResult {
    Done,
    More,
}

#[derive(Debug)]
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
}

struct CallFrame {
    program_counter: usize,
    base_counter: usize,
    closure: Gc<Closure>,

    chunk: *const Chunk,
}

impl Trace for CallFrame {
    fn trace(&self) {
        self.closure.trace();
    }
}

impl CallFrame {
    pub fn new(closure: Gc<Closure>, base_counter: usize) -> Self {
        let chunk: *const Chunk = closure.function.import.chunk(closure.function.chunk_index);
        Self {
            program_counter: 0,
            base_counter,
            closure,
            chunk,
        }
    }

    #[inline]
    fn chunk(&self) -> &Chunk {
        // We use unsafe here because it's way faster
        // This is safe, because we have `Root<Closure>` which eventually has a `Gc<Import>`.
        unsafe { &*self.chunk }
    }

    #[inline]
    pub fn next_instruction(&mut self) -> Instruction {
        let instr = self.chunk().instruction(self.program_counter);
        self.program_counter += 1;
        instr
    }

    #[inline]
    pub fn set_pc(&mut self, value: usize) {
        self.program_counter = value;
    }
}

pub struct Vm<W> where W: Write {
    imports: UniqueRoot<HashMap<String, Gc<Import>>>,
    frames: UniqueRoot<Vec<CallFrame>>,
    stack: UniqueRoot<Vec<Value>>,
    upvalues: UniqueRoot<Vec<Gc<RefCell<Upvalue>>>>,
    stdout: W,
    interner: Interner,
    heap: Heap,
}

impl Vm<Stdout> {
    pub fn new(module: Module) -> Self {
        Vm::with_stdout(module, stdout())
    }
}

impl<W> Vm<W> where W: Write {
    pub fn with_stdout(module: Module, stdout: W) -> Self {
        let mut heap = Heap::new();
        let mut vm = Vm {
            frames: heap.unique(Vec::with_capacity(8192)),
            stack: heap.unique(Vec::with_capacity(8192)),
            upvalues: heap.unique(Vec::with_capacity(8192)),
            stdout,
            imports: heap.unique(HashMap::new()),
            interner: Interner::new(),
            heap,
        };

        //TODO reserve frames/upvalues/stack

        vm.prepare_interpret(module);

        vm
    }

    fn prepare_interpret(&mut self, module: Module) {
        let import = self.heap.manage(Import::new(module, &mut self.interner));
        self.imports.insert("_root".into(), import.as_gc());

        let function = Function {
            arity: 0,
            chunk_index: 0,
            name: "top".into(),
            import: import.as_gc(),
        };
        let closure = self.heap.manage(Closure {
            upvalues: vec![],
            function,
        });
        self.push(Value::Closure(closure.as_gc()));
        self.frames.push(CallFrame::new(closure.as_gc(), 0));
    }

    #[inline]
    fn current_import(&self) -> Gc<Import> {
        self.current_frame().closure.function.import
    }

    pub fn interpret(&mut self) -> Result<(), VmError> {
        while self.interpret_next()? == InterpretResult::More {
            self.heap.collect();
        }

        Ok(())
    }

    pub fn set_native_fn(&mut self, identifier: &str, code: fn(&[Value]) -> Value) {
        let native_function = NativeFunction {
            name: identifier.to_string(),
            code,
        };
        
        let identifier = self.interner.intern(identifier);
        let root = self.heap.manage(native_function);
        self.current_import().set_global(identifier, Value::NativeFunction(root.as_gc()))
    }

    fn interpret_next(&mut self) -> Result<InterpretResult, VmError> {
        use crate::bytecode::Constant;

        let current_import = self.current_import();

        let instr = {
            let frame = self.current_frame_mut();
            frame.next_instruction()
        };

        if false {
            // DEBUG
            println!("stack len: {}", self.stack.len());
            println!("stack: {:?}", self.stack);
            println!("");
            println!("stack len: {}", self.stack.len());
            println!("globals: {:?}", current_import.globals.borrow());
            println!("{:?}", instr);
            println!("");
        }

        match instr {
            Instruction::Constant(index) => match current_import.constant(index) {
                Constant::Number(n) => self.push(Value::Number(*n)),
                Constant::String(string) => self.push_string(string),
            },

            Instruction::Import(_) => unimplemented!(),
            Instruction::ImportGlobal(_) => unimplemented!(),
            
            Instruction::Closure(index) => {
                let closure = current_import.closure(index);
                let upvalues = closure
                    .upvalues
                    .iter()
                    .map(|u| {
                        match u {
                            crate::bytecode::Upvalue::Local(index) => {
                                let frame = &self.frames[self.frames.len() - 1]; //TODO Result // Get the enclosing frame
                                let base = frame.base_counter;
                                let index = base + *index;

                                if let Some(upvalue) = self.find_open_upvalue_with_index(index)
                                {
                                    upvalue
                                } else {
                                    let root = self.heap.manage(RefCell::new(Upvalue::Open(index)));
                                    self.upvalues.push(root.as_gc());
                                    root.as_gc()
                                }
                            }
                            crate::bytecode::Upvalue::Upvalue(u) => {
                                self.find_upvalue_by_index(*u)
                            }
                        }
                    })
                    .collect();

                let closure_root = self.heap.manage(Closure {
                    function: Function::new(&closure.function, current_import),
                    upvalues,
                });
                self.push(Value::Closure(closure_root.as_gc()));
            }
            Instruction::Class(index) => {
                let class = current_import.class(index);
                let class = self.heap.manage(RefCell::new(Class {
                    name: class.name.clone(),
                    methods: FxHashMap::default(),
                }));
                self.push(Value::Class(class.as_gc()));
            }
            //TODO Rewrite if's to improve error handling
            //TODO Pretty sure it leaves the stack clean, but double check
            Instruction::Method(index) => {
                let identifier = current_import.symbol(index);
                if let Value::Class(class) = self.peek_n(1) {
                    if let Value::Closure(closure) = self.peek_n(0) {
                        class.borrow_mut().methods.insert(identifier, *closure);
                    } else {
                        return Err(VmError::UnexpectedConstant);
                    }
                } else {
                    return Err(VmError::UnexpectedConstant);
                }

                self.pop();
            },
            Instruction::SetProperty(index) => {
                let property = current_import.symbol(index);
                if let Value::Instance(instance) = self.peek_n(1) {
                    instance
                        .borrow_mut()
                        .fields
                        .insert(property, *self.peek());

                    let value = self.pop();
                    self.pop();
                    self.push(value);
                } else {
                    return Err(VmError::UnexpectedValue);
                }
            }
            Instruction::GetProperty(index) => {
                let property = current_import.symbol(index);
                if let Value::Instance(instance) = self.pop() {
                    if let Some(value) = instance.borrow().fields.get(&property) {
                        self.push(*value);
                    } else if let Some(method) = instance.borrow().class.borrow().methods.get(&property) {
                        let bind = self.heap.manage(BoundMethod {
                            receiver: instance,
                            method: *method,
                        });
                        self.push(Value::BoundMethod(bind.as_gc()));
                    } else {
                        return Err(VmError::UndefinedProperty);
                    };
                } else {
                    return Err(VmError::UnexpectedValue);
                }
            }
            Instruction::Print => match self.pop() {
                Value::Number(n) => writeln!(self.stdout, "{}", n).expect("Could not write to stdout"),
                Value::Nil => writeln!(self.stdout, "nil").expect("Could not write to stdout"),
                Value::Boolean(boolean) => writeln!(self.stdout, "{}", boolean).expect("Could not write to stdout"),
                Value::String(string) => writeln!(self.stdout, "{}", string).expect("Could not write to stdout"),
                Value::NativeFunction(_function) => writeln!(self.stdout, "<native fn>").expect("Could not write to stdout"),
                Value::Closure(closure) => writeln!(self.stdout, "<fn {}>", closure.function.name).expect("Could not write to stdout"),
                Value::Class(class) => writeln!(self.stdout, "{}", class.borrow().name).expect("Could not write to stdout"),
                Value::Instance(instance) => {
                    writeln!(self.stdout, "{} instance", instance.borrow().class.borrow().name).expect("Could not write to stdout")
                },
                Value::BoundMethod(bind) => writeln!(self.stdout, "<fn {}>", bind.method.function.name).expect("Could not write to stdout"),
                Value::Import(_) => writeln!(self.stdout, "<import>").expect("Could not write to stdout"),
            },
            Instruction::Nil => self.push(Value::Nil),
            Instruction::Return => {
                let result = self.pop();
                let frame = self.frames.pop().expect("no frame");

                for i in frame.base_counter..self.stack.len() {
                    self.close_upvalues(i);
                }

                self.stack.truncate(frame.base_counter);

                if self.frames.len() == 0 {
                    // We are done interpreting, don't push a result as it'll be nil
                    return Ok(InterpretResult::Done);
                }

                self.push(result);
            }
            Instruction::Add => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a + b)),
                (Value::String(b), Value::String(a)) => self.push_string(&format!("{}{}", a, b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Subtract => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a - b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Multiply => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a * b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Divide => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a / b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Pop => {
                self.pop();
            }
            Instruction::DefineGlobal(index) => {
                let identifier = current_import.symbol(index);
                let value = self.pop();
                current_import.set_global(identifier, value);
            }
            Instruction::GetGlobal(index) => {
                let identifier = current_import.symbol(index);
                let value = current_import.global(identifier);
                if let Some(value) = value {
                    self.push(value);
                } else {
                    return Err(VmError::GlobalNotDefined);
                }
            }
            Instruction::SetGlobal(index) => {
                let identifier = current_import.symbol(index);
                let value = *self.peek();
                if current_import.has_global(identifier) {
                    current_import.set_global(identifier, value);
                } else {
                    return Err(VmError::GlobalNotDefined);
                }
            }
            Instruction::GetLocal(index) => {
                let index = self.current_frame().base_counter + index;
                self.push(self.stack[index]);
            }
            Instruction::SetLocal(index) => {
                let index = self.current_frame().base_counter + index;
                let value = *self.peek();
                self.stack[index] = value;
            }
            Instruction::True => self.push(Value::Boolean(true)),
            Instruction::False => self.push(Value::Boolean(false)),
            Instruction::JumpIfFalse(to) => {
                if self.peek().is_falsey() {
                    self.current_frame_mut().set_pc(to);
                }
            }
            Instruction::Jump(to) => {
                self.current_frame_mut().set_pc(to);
            }
            Instruction::Less => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push((a < b).into()),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Greater => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push((a > b).into()),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Equal => {
                let b = self.pop();
                let a = self.pop();

                if Value::is_same_type(&a, &b) {
                    match (b, a) {
                        (Value::Number(b), Value::Number(a)) => self.push((a == b).into()),
                        (Value::Boolean(b), Value::Boolean(a)) => self.push((a == b).into()),
                        (Value::String(b), Value::String(a)) => self.push((*a == *b).into()),
                        (Value::Closure(b), Value::Closure(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                        (Value::NativeFunction(b), Value::NativeFunction(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                        (Value::Nil, Value::Nil) => self.push(true.into()),
                        (Value::BoundMethod(b), Value::BoundMethod(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                        (Value::Class(b), Value::Class(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                        (Value::Instance(b), Value::Instance(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                        _ => unimplemented!(),
                    };
                } else {
                    self.push(false.into())
                }
            }
            Instruction::Call(arity) => {
                let callee = *self.peek_n(arity);
                self.call(arity, callee)?;
            }
            Instruction::Negate => match self.pop() {
                Value::Number(n) => self.push(Value::Number(-n)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Not => {
                let is_falsey = self.pop().is_falsey();
                self.push(is_falsey.into());
            }
            Instruction::GetUpvalue(index) => {
                let upvalue = self.current_frame().closure.upvalues[index];
                self.push(self.resolve_upvalue_into_value(&*upvalue.borrow()));
            }
            Instruction::SetUpvalue(index) => {
                let value = *self.peek();
                let upvalue = self.current_frame().closure.upvalues[index];
                self.set_upvalue(&mut *upvalue.borrow_mut(), value);
            }
            Instruction::CloseUpvalue => {
                let index = self.stack.len() - 1;
                self.close_upvalues(index);
                self.pop();
            }
            Instruction::Invoke(index, arity) => {
                let property = current_import.symbol(index);
                if let Value::Instance(instance) = *self.peek_n(arity) {
                    if let Some(value) = instance.borrow().fields.get(&property) {
                        self.rset(arity+1, *value);
                        self.call(arity, *value)?;
                    } else if let Some(method) = instance.borrow().class.borrow().methods.get(&property) {
                        if method.function.arity != arity {
                            return Err(VmError::IncorrectArity);
                        }
                        self.begin_frame(*method);
                    } else {
                        return Err(VmError::UndefinedProperty);
                    };
                } else {
                    return Err(VmError::UnexpectedValue);
                }
            }
        }

        Ok(InterpretResult::More)
    }

    fn close_upvalues(&mut self, index: usize) {
        for upvalue in self.upvalues.iter() {
            if upvalue.borrow().is_open_with_index(index) {
                let value = self.stack[index];
                upvalue.replace(Upvalue::Closed(value));
            }
        }

        self.upvalues.retain(|u| u.borrow().is_open());
    }

    fn find_upvalue_by_index(&self, index: usize) -> Gc<RefCell<Upvalue>> {
        let frame = &self.frames[self.frames.len() - 1]; //TODO Result
        frame.closure.upvalues[index]
    }

    fn find_open_upvalue_with_index(&self, index: usize) -> Option<Gc<RefCell<Upvalue>>> {
        for upvalue in self.upvalues.iter().rev() {
            if upvalue.borrow().is_open_with_index(index) {
                return Some(*upvalue);
            }
        }

        None
    }

    fn resolve_upvalue_into_value(&self, upvalue: &Upvalue) -> Value {
        match upvalue {
            Upvalue::Closed(value) => *value,
            Upvalue::Open(index) => self.stack[*index],
        }
    }

    fn set_upvalue(&mut self, upvalue: &mut Upvalue, new_value: Value) {
        match upvalue {
            Upvalue::Closed(value) => *value = new_value,
            Upvalue::Open(index) => self.stack[*index] = new_value,
        }
    }

    //TODO Reduce duplicate code paths
    fn call(&mut self, arity: usize, callee: Value) -> Result<(), VmError> {
        match callee {
            Value::Closure(callee) => {
                if callee.function.arity != arity {
                    return Err(VmError::IncorrectArity);
                }
                self.begin_frame(callee);
            }
            Value::NativeFunction(callee) => {
                let mut args = self.pop_n(arity);
                args.reverse();
                self.pop(); // discard callee
                let result = (callee.code)(&args);
                self.push(result);
            }
            Value::Class(class) => {
                let instance = self.heap.manage(RefCell::new(Instance {
                    class,
                    fields: FxHashMap::default(),
                }));
                self.rset(arity+1, Value::Instance(instance.as_gc()));

                let init_symbol = self.interner.intern("init"); //TODO move to constructor
                if let Some(initializer) = class.borrow().methods.get(&init_symbol) {
                    if initializer.function.arity != arity {
                        return Err(VmError::IncorrectArity);
                    }
                    self.begin_frame(*initializer);
                } else if arity != 0 {
                    return Err(VmError::IncorrectArity); // Arity must be 0 without initializer
                }
            }
            Value::BoundMethod(bind) => {
                let callee = bind.method;
                if callee.function.arity != arity {
                    return Err(VmError::IncorrectArity);
                }
                self.rset(arity+1, Value::Instance(bind.receiver));
                self.begin_frame(callee);
            },
            _ => return Err(VmError::InvalidCallee),
        }

        Ok(())
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("No frame")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("No frame")
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    //TODO match index with n of peek_n, this is 1 off from peek_n.
    fn rset(&mut self, index: usize, value: Value) {
        let index = self.stack.len() - index;
        self.stack[index] = value;
    }

    fn push_string(&mut self, string: &str) {
        let root = self.heap.manage(string.to_string());
        self.push(Value::String(root.as_gc()));
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Stack empty")
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        let mut result = Vec::with_capacity(n);
        while result.len() < n {
            result.push(self.pop());
        }

        result
    }

    fn peek(&self) -> &Value {
        self.stack.last().expect("Stack empty")
    }

    fn peek_n(&self, n: usize) -> &Value {
        self.stack.get(self.stack.len() - n - 1).expect("Stack not deep enough")
    }

    fn begin_frame(&mut self, closure: Gc<Closure>) {
        self.frames.push(CallFrame::new(closure, self.stack.len() - closure.function.arity - 1));
    }
}

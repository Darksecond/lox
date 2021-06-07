use lox_bytecode::bytecode::Chunk;

use super::memory::*;
use crate::bettergc::{gc, Gc, Root, UniqueRoot};
use crate::bytecode::{Module};
use std::{cell::RefCell, io::{Stdout, Write, stdout}};
use std::collections::HashMap;

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
    closure: Root<Closure>,
    chunk: *const Chunk,
}

impl CallFrame {
    #[inline]
    fn chunk(&self) -> &Chunk {
        // We use unsafe here because it's way faster
        // This is safe, because we have `Root<Closure>` which eventually has a `Gc<Import>`.
        unsafe { &*self.chunk }
        // self.closure.function.import.chunk(self.closure.function.chunk_index)
    }
}

pub struct Vm<W> where W: Write {
    imports: UniqueRoot<HashMap<String, Gc<Import>>>,
    frames: Vec<CallFrame>,
    stack: UniqueRoot<Vec<Value>>,
    upvalues: Vec<Root<RefCell<Upvalue>>>,
    stdout: W,
}

impl Vm<Stdout> {
    pub fn new(module: Module) -> Self {
        Vm::with_stdout(module, stdout())
    }
}

impl<W> Vm<W> where W: Write {
    pub fn with_stdout(module: Module, stdout: W) -> Self {
        let mut vm = Vm {
            frames: vec![],
            stack: gc::unique(vec![]),
            upvalues: vec![],
            stdout,
            imports: gc::unique(HashMap::new()),
        };

        vm.prepare_interpret(module);

        vm
    }

    fn prepare_interpret(&mut self, module: Module) {
        let import = gc::manage(Import::new(module));
        self.imports.insert("_root".into(), import.as_gc());

        let function = gc::manage(Function {
            arity: 0,
            chunk_index: 0,
            name: "top".into(),
            import: import.as_gc(),
        });
        let closure = gc::manage(Closure {
            upvalues: vec![],
            function: function.as_gc(),
        });
        self.push(Value::Closure(closure.as_gc()));

        let chunk: *const Chunk = closure.function.import.chunk(closure.function.chunk_index);
        self.frames.push(CallFrame {
            //TODO Use begin/end_frame because it needs to do cleanup of the stack
            program_counter: 0,
            base_counter: 0,
            closure,
            chunk,
        });
    }

    fn current_import(&self) -> Result<Gc<Import>, VmError> {
        Ok(self.current_frame()?.closure.function.import)
    }

    pub fn interpret(&mut self) -> Result<(), VmError> {
        while self.interpret_next()? == InterpretResult::More {
            gc::collect();
        }

        Ok(())
    }

    pub fn set_native_fn(&mut self, identifier: &str, code: fn(&[Value]) -> Value) {
        let native_function = NativeFunction {
            name: identifier.to_string(),
            code,
        };

        let root = gc::manage(native_function);
        self.current_import().unwrap().set_global(identifier, Value::NativeFunction(root.as_gc()))
    }

    fn interpret_next(&mut self) -> Result<InterpretResult, VmError> {
        use crate::bytecode::{Constant, Instruction};

        let current_import = self.current_import()?;

        self.current_frame_mut()?.program_counter += 1;

        let instr = {
            let frame = self.current_frame()?;
            frame.chunk().instructions()[frame.program_counter - 1]
        };

        if false {
            // DEBUG
            println!("stack: {:?}", self.stack);
            println!("");
            println!("globals: {:?}", current_import.globals.borrow());
            println!("{:?}", instr);
            println!("");
        }

        match instr {
            Instruction::Constant(index) => match current_import.constant(index) {
                Constant::Number(n) => self.push(Value::Number(*n)),
                Constant::String(string) => self.push_string(string),
                Constant::Class(_) => unimplemented!(),
                Constant::Closure(_) => unimplemented!(),
            },

            Instruction::Import(_) => unimplemented!(),
            Instruction::ImportGlobal(_) => unimplemented!(),
            
            Instruction::Closure(index) => {
                if let Constant::Closure(closure) = current_import.constant(index) {
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
                                        let root = gc::manage(RefCell::new(Upvalue::Open(index)));
                                        self.upvalues.push(root.clone());
                                        root.as_gc()
                                    }
                                }
                                crate::bytecode::Upvalue::Upvalue(u) => {
                                    self.find_upvalue_by_index(*u)
                                }
                            }
                        })
                        .collect();

                    let function_root = gc::manage(Function::new(&closure.function, current_import));
                    let closure_root = gc::manage(Closure {
                        function: function_root.as_gc(),
                        upvalues: upvalues,
                    });
                    self.push(Value::Closure(closure_root.as_gc()));
                } else {
                    return Err(VmError::ClosureConstantExpected);
                }
            }
            Instruction::Class(index) => {
                if let Constant::Class(class) = current_import.constant(index) {
                    let class = gc::manage(RefCell::new(Class {
                        name: class.name.clone(),
                        methods: HashMap::new(),
                    }));
                    self.push(Value::Class(class.as_gc()));
                } else {
                    return Err(VmError::UnexpectedConstant);
                }
            }
            //TODO Rewrite if's to improve error handling
            //TODO Pretty sure it leaves the stack clean, but double check
            Instruction::Method(index) => {
                if let Constant::String(identifier) = current_import.constant(index) {
                    if let Value::Class(class) = self.peek_n(1)? {
                        if let Value::Closure(closure) = self.peek_n(0)? {
                            class.borrow_mut().methods.insert(identifier.to_owned(), *closure);
                        } else {
                            return Err(VmError::UnexpectedConstant);
                        }
                    } else {
                        return Err(VmError::UnexpectedConstant);
                    }

                    self.pop()?;
                } else {
                    return Err(VmError::UnexpectedConstant);
                }
            },
            Instruction::SetProperty(index) => {
                if let Constant::String(property) = current_import.constant(index) {
                    if let Value::Instance(instance) = self.peek_n(1)? {
                        instance
                            .borrow_mut()
                            .fields
                            .insert(property.clone(), *self.peek()?);

                        let value = self.pop()?;
                        self.pop()?;
                        self.push(value);
                    } else {
                        return Err(VmError::UnexpectedValue);
                    }
                } else {
                    return Err(VmError::UnexpectedConstant);
                }
            }
            Instruction::GetProperty(index) => {
                if let Constant::String(property) = current_import.constant(index) {
                    if let Value::Instance(instance) = self.pop()? {
                        let instance = gc::root(instance);
                        if let Some(value) = instance.borrow().fields.get(property) {
                            self.push(*value);
                        } else if let Some(method) = instance.borrow().class.borrow().methods.get(property) {
                            let bind = gc::manage(BoundMethod {
                                receiver: instance.as_gc(),
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
            }
            Instruction::Print => match self.pop()? {
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
                let result = self.pop()?;
                let frame = self.frames.pop().ok_or(VmError::FrameEmpty)?;

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
            Instruction::Add => match (self.pop()?, self.pop()?) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a + b)),
                (Value::String(b), Value::String(a)) => self.push_string(&format!("{}{}", a, b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Subtract => match (self.pop()?, self.pop()?) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a - b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Multiply => match (self.pop()?, self.pop()?) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a * b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Divide => match (self.pop()?, self.pop()?) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a / b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Pop => {
                self.pop()?;
            }
            Instruction::DefineGlobal(index) => {
                if let Constant::String(identifier) = current_import.constant(index) {
                    let value = self.pop()?;
                    current_import.set_global(identifier, value);
                } else {
                    return Err(VmError::StringConstantExpected);
                }
            }
            Instruction::GetGlobal(index) => {
                if let Constant::String(identifier) = current_import.constant(index) {
                    let value = current_import.global(identifier);
                    if let Some(value) = value {
                        self.push(value);
                    } else {
                        return Err(VmError::GlobalNotDefined);
                    }
                } else {
                    return Err(VmError::StringConstantExpected);
                }
            }
            Instruction::SetGlobal(index) => {
                if let Constant::String(identifier) = current_import.constant(index) {
                    let value = *self.peek()?;
                    if current_import.has_global(identifier) {
                        current_import.set_global(identifier, value);
                    } else {
                        return Err(VmError::GlobalNotDefined);
                    }
                } else {
                    return Err(VmError::StringConstantExpected);
                }
            }
            Instruction::GetLocal(index) => {
                let index = self.current_frame()?.base_counter + index;
                self.push(self.stack[index]);
            }
            Instruction::SetLocal(index) => {
                let index = self.current_frame()?.base_counter + index;
                let value = *self.peek()?;
                self.stack[index] = value;
            }
            Instruction::True => self.push(Value::Boolean(true)),
            Instruction::False => self.push(Value::Boolean(false)),
            Instruction::JumpIfFalse(to) => {
                if self.peek()?.is_falsey() {
                    self.current_frame_mut()?.program_counter = to;
                }
            }
            Instruction::Jump(to) => {
                self.current_frame_mut()?.program_counter = to;
            }
            Instruction::Less => match (self.pop()?, self.pop()?) {
                (Value::Number(b), Value::Number(a)) => self.push((a < b).into()),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Greater => match (self.pop()?, self.pop()?) {
                (Value::Number(b), Value::Number(a)) => self.push((a > b).into()),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Equal => {
                let b = self.pop()?;
                let a = self.pop()?;

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
                self.call(arity)?;
            }
            Instruction::Negate => match self.pop()? {
                Value::Number(n) => self.push(Value::Number(-n)),
                _ => return Err(VmError::UnexpectedValue),
            },
            Instruction::Not => {
                let is_falsey = self.pop()?.is_falsey();
                self.push(is_falsey.into());
            }
            Instruction::GetUpvalue(index) => {
                let upvalue = self.current_frame()?.closure.upvalues[index];
                self.push(self.resolve_upvalue_into_value(&*upvalue.borrow()));
            }
            Instruction::SetUpvalue(index) => {
                let value = *self.peek()?;
                let upvalue = self.current_frame()?.closure.upvalues[index];
                self.set_upvalue(&mut *upvalue.borrow_mut(), value);
            }
            Instruction::CloseUpvalue => {
                let index = self.stack.len() - 1;
                self.close_upvalues(index);
                self.stack.pop().ok_or(VmError::StackEmpty)?;
            }
        }

        Ok(InterpretResult::More)
    }

    fn close_upvalues(&mut self, index: usize) {
        let value = self.stack[index]; //TODO Result
        for root in &self.upvalues {
            if root.borrow().is_open_with_index(index) {
                root.replace(Upvalue::Closed(value));
            }
        }

        self.upvalues.retain(|u| u.borrow().is_open());
    }

    fn find_upvalue_by_index(&self, index: usize) -> Gc<RefCell<Upvalue>> {
        let frame = &self.frames[self.frames.len() - 1]; //TODO Result
        frame.closure.upvalues[index]
    }

    fn find_open_upvalue_with_index(&self, index: usize) -> Option<Gc<RefCell<Upvalue>>> {
        for root in self.upvalues.iter().rev() {
            if root.borrow().is_open_with_index(index) {
                return Some(root.as_gc());
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
    fn call(&mut self, arity: usize) -> Result<(), VmError> {
        let callee = *self.peek_n(arity)?;
        match callee {
            Value::Closure(callee) => {
                if callee.function.arity != arity {
                    return Err(VmError::IncorrectArity);
                }
                self.begin_frame(callee);
            }
            Value::NativeFunction(callee) => {
                let mut args = self.pop_n(arity)?;
                args.reverse();
                self.pop()?; // discard callee
                let result = (callee.code)(&args);
                self.push(result);
            }
            Value::Class(class) => {
                let instance = gc::manage(RefCell::new(Instance {
                    class,
                    fields: HashMap::new(),
                }));
                self.rset(arity+1, Value::Instance(instance.as_gc()));

                if let Some(initializer) = class.borrow().methods.get("init") {
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

    fn current_frame(&self) -> Result<&CallFrame, VmError> {
        self.frames.last().ok_or(VmError::FrameEmpty)
    }

    fn current_frame_mut(&mut self) -> Result<&mut CallFrame, VmError> {
        self.frames.last_mut().ok_or(VmError::FrameEmpty)
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn rset(&mut self, index: usize, value: Value) {
        let index = self.stack.len() - index;
        self.stack[index] = value;
    }

    fn push_string(&mut self, string: &str) {
        let root = gc::manage(string.to_string());
        self.push(Value::String(root.as_gc()));
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackEmpty)
    }

    fn pop_n(&mut self, n: usize) -> Result<Vec<Value>, VmError> {
        let mut result = Vec::with_capacity(n);
        while result.len() < n {
            result.push(self.pop()?);
        }

        Ok(result)
    }

    fn peek(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or(VmError::StackEmpty)
    }

    fn peek_n(&self, n: usize) -> Result<&Value, VmError> {
        self.stack
            .get(self.stack.len() - n - 1)
            .ok_or(VmError::StackEmpty)
    }

    fn begin_frame(&mut self, closure: Gc<Closure>) {
        let chunk: *const Chunk = closure.function.import.chunk(closure.function.chunk_index);
        self.frames.push(CallFrame {
            program_counter: 0,
            base_counter: self.stack.len() - closure.function.arity - 1,
            closure: gc::root(closure),
            chunk,
        });
    }
}

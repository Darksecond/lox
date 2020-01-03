use super::memory::*;
use std::collections::HashMap;
use crate::bytecode::{Module, Chunk};
use crate::bettergc::{UniqueRoot, Gc, Weak, Root, gc};
use std::cell::RefCell;

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
    IncorrectArity
}

struct CallFrame<'a> {
    program_counter: usize,
    base_counter: usize,
    chunk: &'a Chunk,
    closure: Root<Closure>,
}

pub struct Vm<'a> {
    module: &'a Module,
    frames: Vec<CallFrame<'a>>,
    stack: UniqueRoot<Vec<Value>>,
    globals: UniqueRoot<HashMap<String, Value>>,
    upvalues: Vec<Weak<RefCell<Upvalue>>>, //TODO Replace this with a list of *just* open upvalues, then we don't care about it being weak.
}

impl<'a> Vm<'a> {
    pub fn new(module: &'a Module) -> Self {
        Vm {
            module,
            frames: vec![],
            stack: gc::unique(vec![]),
            globals: gc::unique(HashMap::new()),
            upvalues: vec![],
        }
    }

    pub fn interpret(&mut self) -> Result<(), VmError> {
        let function = gc::manage(Function{ arity: 0, chunk_index: 0, name: "top".into() });
        let closure = gc::manage(Closure { upvalues: vec![], function: function.as_gc() });
        self.push(Value::Object(Object::Closure(closure.as_gc())));

        self.frames.push(CallFrame { //TODO Use begin/end_frame because it needs to do cleanup of the stack
            program_counter: 0,
            base_counter: 0,
            chunk: self.module.chunk(0),
            closure: closure,
        });

        while self.interpret_next()? == InterpretResult::More {};

        //TODO properly end the frame, close upvalues and clean the stack
        //     this is required if we later want to be able to re-use a 'VmModule' in another vm.

        Ok(())
    }

    fn interpret_next(&mut self) -> Result<InterpretResult, VmError> {
        use crate::bytecode::{Instruction, Constant};

        
        self.current_frame_mut()?.program_counter += 1;

        let instr = {
            let frame = self.current_frame()?;
            frame.chunk.instructions()[frame.program_counter-1]
        };

        if false { // DEBUG
            println!("stack: {:?}", self.stack);
            println!("");
            println!("globals: {:?}", self.globals);
            println!("{:?}", instr);
            println!("");
        }

        match instr {
            Instruction::Constant(index) => {
                match self.module.constant(index) {
                    Constant::Number(n) => self.push(Value::Number(*n)),
                    Constant::String(string) => self.push_string(string),
                    Constant::Closure(closure) => {
                        let upvalues = closure.upvalues.iter().map(|u| {
                            match u {
                                crate::bytecode::Upvalue::Local(index) => {
                                    let frame = &self.frames[self.frames.len()-1]; //TODO Result // Get the enclosing frame
                                    let base = frame.base_counter;
                                    let index = base + *index;

                                    //TODO Write a test for this
                                    // Try to find an existing upvalue that matches ours
                                    for upvalue in self.upvalues.iter().rev() {
                                        if let Some(root) = gc::upgrade(upvalue) {
                                            if root.borrow().is_open_with_index(index) { return root; }
                                        }
                                    }

                                    gc::manage(RefCell::new(Upvalue::Open(index)))
                                },
                                crate::bytecode::Upvalue::Upvalue(u) => gc::root(self.find_upvalue_by_index(*u)),
                            }
                        });
                        let upvalue_roots: Vec<Root<RefCell<Upvalue>>> = upvalues.collect();

                        // Put all upvalues into a big list so we can iterate through them.
                        //TODO In the future only push open upvalues, and only if they are not in the list already
                        //     We should consider making it a Set
                        for upvalue in &upvalue_roots {
                            self.upvalues.push(gc::downgrade(upvalue.as_gc()));
                        }

                        let function_root = gc::manage(Function::from(&closure.function));
                        let closure_root = gc::manage(Closure {
                            function: function_root.as_gc(),
                            upvalues: upvalue_roots.iter().map(|r| r.as_gc()).collect(),
                        });
                        let object = Object::Closure(closure_root.as_gc());
                        self.push(Value::Object(object));
                    },
                }
            },
            Instruction::Print => {
                match self.pop()? {
                    Value::Number(n) => println!("{}", n),
                    Value::Nil => println!("nil"),
                    Value::Boolean(boolean) => println!("{}", boolean),
                    Value::Object(ref obj) => {
                        match obj {
                            Object::String(string) => println!("{}", string),
                            Object::NativeFunction(function) => println!("<native fun {}>", function.name),
                            Object::Closure(closure) => println!("<fun {}({}) @ {}>", closure.function.name, closure.function.arity, closure.function.chunk_index),
                        }
                    },
                }
            },
            Instruction::Nil => {
                self.push(Value::Nil)
            },
            Instruction::Return => {
                let result = self.pop()?;
                let frame = self.frames.pop().ok_or(VmError::FrameEmpty)?;
                if self.frames.len() == 0 { return Ok(InterpretResult::Done); } // We are done interpreting //TODO Move this down?

                for i in frame.base_counter..self.stack.len() {
                    self.close_upvalues(i);
                }
                
                self.stack.split_off(frame.base_counter);
                self.push(result);
            },
            Instruction::Add => {
                match (self.pop()?, self.pop()?) {
                    (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a+b)),
                    (Value::Object(ref b), Value::Object(ref a)) => {
                        match (b, a) {
                            (Object::String(b), Object::String(a)) => self.push_string(&format!("{}{}", a, b)),
                            (b, a) => unimplemented!("{:?} + {:?}", a, b),
                        }
                    },
                    (b, a) => unimplemented!("{:?} + {:?}", a, b),
                }
            },
            Instruction::Subtract => {
                match (self.pop()?, self.pop()?) {
                    (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a-b)),
                    (b, a) => unimplemented!("{:?} - {:?}", a, b),
                }
            },
            Instruction::Multiply => {
                match (self.pop()?, self.pop()?) {
                    (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a*b)),
                    (b, a) => unimplemented!("{:?} * {:?}", a, b),
                }
            },
            Instruction::Pop => {
                self.pop()?;
            },
            Instruction::DefineGlobal(index) => {
                if let Constant::String(identifier) = self.module.constant(index) {
                    let value = self.pop()?;
                    self.globals.insert(identifier.to_string(), value);
                } else {
                    return Err(VmError::StringConstantExpected);
                }
            },
            Instruction::GetGlobal(index) => {
                if let Constant::String(identifier) = self.module.constant(index) {
                    let value = self.globals.get(identifier).cloned();
                    if let Some(value) = value {
                        self.push(value);
                    } else {
                        return Err(VmError::GlobalNotDefined);
                    }
                } else {
                    return Err(VmError::StringConstantExpected);
                }
            },
            Instruction::SetGlobal(index) => {
                if let Constant::String(identifier) = self.module.constant(index) {
                    let value = *self.peek()?;
                    if self.globals.contains_key(identifier) { 
                        self.globals.insert(identifier.to_string(), value);
                    } else {
                        return Err(VmError::GlobalNotDefined); 
                    }
                } else {
                    return Err(VmError::StringConstantExpected);
                }
            },
            Instruction::GetLocal(index) => {
                let index = self.current_frame()?.base_counter+index;
                self.push(self.stack[index]);
            },
            Instruction::SetLocal(index) => {
                let index = self.current_frame()?.base_counter+index;
                let value = *self.peek()?;
                self.stack[index] = value;
            },
            Instruction::True => {
                self.push(Value::Boolean(true))
            },
            Instruction::False => {
                self.push(Value::Boolean(false))
            },
            Instruction::JumpIfFalse(to) => {
                if self.peek()?.is_falsey() {
                    self.current_frame_mut()?.program_counter = to;
                }
            },
            Instruction::Jump(to) => {
                self.current_frame_mut()?.program_counter = to;
            },
            Instruction::Less => {
                match (self.pop()?, self.pop()?) {
                    (Value::Number(b), Value::Number(a)) => self.push((a < b).into()),
                    (b, a) => unimplemented!("{:?} < {:?}", a, b),
                }
            },
            Instruction::Equal => {
                let b = self.pop()?;
                let a = self.pop()?;

                if Value::is_same_type(&a, &b) {
                    match (b,a) {
                        (Value::Number(b), Value::Number(a)) => self.push((a == b).into()),
                        (Value::Boolean(b), Value::Boolean(a)) => self.push((a == b).into()),
                        (Value::Object(Object::String(b)), Value::Object(Object::String(a))) => self.push((*a == *b).into()),
                        (Value::Nil, Value::Nil) => self.push(true.into()),
                        _ => (),
                    };
                } else {
                    self.push(false.into())
                }
            },
            Instruction::Call(arity) => {
                self.call(arity)?;
            },
            Instruction::Negate => {
                match self.pop()? {
                    Value::Number(n) => self.push(Value::Number(-n)),
                    x => unimplemented!("{:?}", x),
                }
            },
            Instruction::Not => {
                let is_falsey = self.pop()?.is_falsey();
                self.push(is_falsey.into());
            },
            Instruction::GetUpvalue(index) => {
                let upvalue = self.current_frame()?.closure.upvalues[index];
                self.push(self.resolve_upvalue_into_value(&*upvalue.borrow()));
            },
            Instruction::SetUpvalue(index) => {
                let value = *self.peek()?;
                let upvalue = self.current_frame()?.closure.upvalues[index];
                self.set_upvalue(&mut *upvalue.borrow_mut(), value);
            },
            Instruction::CloseUpvalue => {
                let index = self.stack.len() - 1;
                self.close_upvalues(index);
                self.stack.pop().ok_or(VmError::StackEmpty)?;
            },
            _ => unimplemented!("{:?}", instr),
        }

        Ok(InterpretResult::More)
    }

    fn close_upvalues(&mut self, index: usize) {
        let value = self.stack[index]; //TODO Result
        for upvalue in &self.upvalues {
            if let Some(root) = gc::upgrade(upvalue) {
                if root.borrow().is_open_with_index(index) {
                    root.replace(Upvalue::Closed(value));
                }
            }
        }
    }

    fn find_upvalue_by_index(&self, index: usize) -> Gc<RefCell<Upvalue>> {
        let frame = &self.frames[self.frames.len()-1]; //TODO Result
        frame.closure.upvalues[index]
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

    fn call(&mut self, arity: usize) -> Result<(), VmError> {
        let callee = *self.peek_n(arity)?;
        if let Value::Object(ref callee) = callee {
            match callee {
                Object::Closure(callee) => {
                    if callee.function.arity != arity { return Err(VmError::IncorrectArity); }
                    self.begin_frame(*callee);
                },
                Object::NativeFunction(callee) => {
                    let mut args = self.pop_n(arity)?;
                    args.reverse();
                    self.pop()?; // discard callee
                    let result = (callee.code)(&args);
                    self.push(result);
                },
                _ => return Err(VmError::InvalidCallee),
            }
        } else {
            return Err(VmError::InvalidCallee);
        }

        Ok(())
    }

    fn current_frame(&self) -> Result<&CallFrame, VmError> {
        self.frames.last().ok_or(VmError::FrameEmpty)
    }

    fn current_frame_mut(&mut self) -> Result<&mut CallFrame<'a>, VmError> {
        self.frames.last_mut().ok_or(VmError::FrameEmpty)
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn push_string(&mut self, string: &str) {
        let root = gc::manage(string.to_string());
        let object = Object::String(root.as_gc());
        self.push(Value::Object(object));
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackEmpty)
    }

    fn pop_n(&mut self, n: usize) -> Result<Vec<Value>, VmError> {
        let mut result = vec![];
        while result.len() < n {
            result.push(self.pop()?);
        }

        Ok(result)
    }

    fn peek(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or(VmError::StackEmpty)
    }

    fn peek_n(&self, n: usize) -> Result<&Value, VmError> {
        self.stack.get(self.stack.len() - n - 1).ok_or(VmError::StackEmpty)
    }

    fn begin_frame(&mut self, closure: Gc<Closure>) {
        self.frames.push(CallFrame {
            program_counter: 0,
            base_counter: self.stack.len() - closure.function.arity - 1,
            chunk: self.module.chunk(closure.function.chunk_index),
            closure: gc::root(closure),
        });
    }

    pub fn set_native_fn(&mut self, identifier: &str, code: fn(&[Value]) -> Value) {
        let native_function = NativeFunction {
            name: identifier.to_string(),
            code: code,
        };

        let root = gc::manage(native_function);
        let object = Object::NativeFunction(root.as_gc());
        self.globals.insert(identifier.to_string(), Value::Object(object));
    }
}
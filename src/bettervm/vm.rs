use super::memory::*;
use std::collections::HashMap;
use crate::bytecode::{Module, Chunk};
use crate::bettergc::{UniqueRoot, gc};

#[derive(PartialEq)]
enum InterpretResult {
    Done,
    More,
}

pub enum VmError {
    StackEmpty,
    FrameEmpty,
    StringConstantExpected,
    GlobalNotDefined,
    InvalidCallee
}

struct CallFrame<'a> {
    program_counter: usize,
    base_counter: usize,
    chunk: &'a Chunk,
}

pub struct Vm<'a> {
    module: &'a Module,
    frames: Vec<CallFrame<'a>>,
    stack: UniqueRoot<Vec<Value>>,
    globals: UniqueRoot<HashMap<String, Value>>,
}

impl<'a> Vm<'a> {
    pub fn new(module: &'a Module) -> Self {
        Vm {
            module,
            frames: vec![],
            stack: gc::unique(vec![]),
            globals: gc::unique(HashMap::new()),
        }
    }

    pub fn interpret(&mut self) -> Result<(), VmError> {
        self.frames.push(CallFrame { //Use begin/end_chunk because it needs to do cleanup of the stack
            program_counter: 0,
            base_counter: 0,
            chunk: self.module.chunk(0),
        });

        while self.interpret_next()? == InterpretResult::More {};

        Ok(())
    }

    fn interpret_next(&mut self) -> Result<InterpretResult, VmError> {
        use crate::bytecode::{Instruction, Constant};

        
        self.current_frame_mut()?.program_counter += 1;

        let frame = self.current_frame()?;
        let instr = &frame.chunk.instructions()[frame.program_counter-1];

        // {
        //     println!("stack: {:?}", self.stack); // DEBUG
        //     println!("globals: {:?}", self.globals); // DEBUG
        //     println!("{:?}", instr); // DEBUG
        //     println!("");
        // }

        match instr {
            Instruction::Constant(index) => {
                match self.module.constant(*index) {
                    Constant::Number(n) => self.push(Value::Number(*n)),
                    Constant::String(string) => self.push_string(string),
                    Constant::Function(function) => {
                        use std::cell::RefCell;

                        let object = Object::Function(function.into());
                        let root = gc::manage(RefCell::new(object));
                        self.push(Value::Object(root.as_gc()));
                    },
                }
            },
            Instruction::Print => {
                match self.pop()? {
                    Value::Number(n) => println!("{}", n),
                    Value::Nil => println!("nil"),
                    Value::True => println!("true"),
                    Value::False => println!("false"),
                    Value::Object(obj) => {
                        match *obj.borrow() {
                            Object::String(ref string) => println!("{}", string),
                            Object::Function(ref function) => println!("fn<{}({}) @ {}>", function.name, function.arity, function.chunk_index)
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
                if self.frames.len() == 0 { return Ok(InterpretResult::Done); } // We are done interpreting
                self.stack.split_off(frame.base_counter);
                self.push(result);
            },
            Instruction::Add => {
                match (self.pop()?, self.pop()?) {
                    (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a+b)),
                    (Value::Object(b), Value::Object(a)) => {
                        match (&*b.borrow(), &*a.borrow()) {
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
                if let Constant::String(identifier) = self.module.constant(*index) {
                    let value = self.pop()?;
                    self.globals.insert(identifier.to_string(), value);
                } else {
                    return Err(VmError::StringConstantExpected);
                }
            },
            Instruction::GetGlobal(index) => {
                if let Constant::String(identifier) = self.module.constant(*index) {
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
                if let Constant::String(identifier) = self.module.constant(*index) {
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
                let index = frame.base_counter+index;
                self.push(self.stack[index]);
            },
            Instruction::SetLocal(index) => {
                let index = frame.base_counter+index;
                let value = *self.peek()?;
                self.stack[index] = value;
            },
            Instruction::True => {
                self.push(Value::True)
            },
            Instruction::False => {
                self.push(Value::False)
            },
            Instruction::JumpIfFalse(to) => {
                if self.peek()?.is_falsey() {
                    self.current_frame_mut()?.program_counter = *to;
                }
            },
            Instruction::Jump(to) => {
                self.current_frame_mut()?.program_counter = *to;
            },
            Instruction::Less => {
                match (self.pop()?, self.pop()?) {
                    (Value::Number(b), Value::Number(a)) => self.push((a < b).into()),
                    (b, a) => unimplemented!("{:?} < {:?}", a, b),
                }
            },
            Instruction::Call(arity) => {
                let arity = *arity;
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
            }
            _ => unimplemented!("{:?}", instr),
        }

        Ok(InterpretResult::More)
    }

    fn call(&mut self, arity: usize) -> Result<(), VmError> {
        let callee = *self.peek_n(arity)?;
        if let Value::Object(callee) = callee {
            if let Object::Function(callee) = &*callee.borrow() {
                self.begin_frame(callee);
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
        use std::cell::RefCell;

        let object = Object::String(string.to_string());
        let root = gc::manage(RefCell::new(object));
        self.push(Value::Object(root.as_gc()));
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackEmpty)
    }

    fn peek(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or(VmError::StackEmpty)
    }

    fn peek_n(&self, n: usize) -> Result<&Value, VmError> {
        self.stack.get(self.stack.len() - n - 1).ok_or(VmError::StackEmpty)
    }

    fn begin_frame(&mut self, function: &Function) {
        self.frames.push(CallFrame {
            program_counter: 0,
            base_counter: self.stack.len() - function.arity - 1,
            chunk: self.module.chunk(function.chunk_index),
        });
    }
}
use crate::bytecode::*;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)] //TODO Double check we want Copy
pub enum Value {
    Number(f64),
    //Nil, Bool, Object
}

pub struct VmState {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

pub struct Vm<'a> {
    state: &'a mut VmState,
    chunk: &'a Chunk,
    program_counter: usize,
}

impl VmState {
    pub fn new() -> VmState {
        VmState {
            stack: vec![],
            globals: HashMap::new(),
        }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    pub fn push_number(&mut self, value: f64) {
        self.push(Value::Number(value))
    }

    pub fn pop(&mut self) -> Value { //TODO Result
        self.stack.pop().unwrap()
    }

    pub fn peek(&mut self) -> &Value { //TODO Result
        self.stack.last().unwrap()
    }
}

impl<'a> Vm<'a> {
    pub fn new(state: &'a mut VmState, chunk: &'a Chunk) -> Vm<'a> {
        Vm {
            state,
            chunk,
            program_counter: 0,
        }
    }

    pub fn interpret_next(&mut self) -> bool { //TODO Result
        self.program_counter += 1;
        let instr = &self.chunk.instructions()[self.program_counter-1];
        println!("Instr: {:?}", instr);
        match *instr {
            Instruction::Return => {return false;},
            Instruction::Constant(index) => {
                match self.chunk.constants()[index] {
                    Constant::Number(n) => self.state.push(Value::Number(n)),
                    Constant::String(_) => unimplemented!(),
                }
            },
            Instruction::Print => {
                let value = self.state.pop();
                match value {
                    Value::Number(n) => println!("{}", n),
                }
            },
            Instruction::Add => {
                match (self.state.pop(), self.state.pop()) {
                    (Value::Number(b), Value::Number(a)) => self.state.push_number(a+b),
                }
            },
            Instruction::Multiply => {
                match (self.state.pop(), self.state.pop()) {
                    (Value::Number(b), Value::Number(a)) => self.state.push_number(a*b),
                }
            },
            Instruction::DefineGlobal(index) => {
                if let Constant::String(identifier) = &self.chunk.constants()[index] {
                    let value = self.state.pop();
                    self.state.globals.insert(identifier.to_string(), value);
                } else { panic!("String constant expected") } //TODO else runtime error
            },
            Instruction::GetGlobal(index) => {
                if let Constant::String(identifier) = &self.chunk.constants()[index] {
                    let value = self.state.globals.get(identifier);
                    if let Some(value) = value {
                        self.state.push(*value);
                    } else {
                        panic!("Runtime error, global not defined"); //TODO else runtime error
                    }
                } else { panic!("String constant expected") } //TODO else runtime error
            },
            Instruction::SetGlobal(index) => {
                if let Constant::String(identifier) = &self.chunk.constants()[index] {
                    let value = *self.state.peek();
                    if !self.state.globals.contains_key(identifier) { panic!("Runtime error, global not defined"); } //TODO else runtime error
                    self.state.globals.insert(identifier.to_string(), value);
                } else { panic!("String constant expected") } //TODO else runtime error
            },
            Instruction::Pop => {
                self.state.pop();
            }
            _ => unimplemented!(),
        }
        true
    }
}
use crate::bytecode::*;

#[derive(Debug)]
pub enum Value {
    Number(f64),
    //Nil, Bool, Object
}

pub struct VmState {
    stack: Vec<Value>,
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
        // println!("Instr: {:?}", instr);
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
            _ => unimplemented!(),
        }
        true
    }
}
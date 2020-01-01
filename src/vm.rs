use crate::bytecode::*;
use std::collections::HashMap;
use std::cell::RefCell;
use crate::bettergc::*;

#[derive(Debug)]
pub enum Object {
    String(String),
}

impl Trace for Object {
    fn trace(&self) {
        match self {
            Object::String(_) => (),
        }
    }
}

#[derive(Debug, Copy, Clone)] //TODO Double check we want Copy
pub enum Value {
    Number(f64),
    Object(Gc<RefCell<Object>>),
    Nil,
    True,
    False,
}

impl Trace for Value {
    fn trace(&self) {
        match self {
            Value::Object(obj) => obj.trace(),
            _ => (),
        }
    }
}

pub struct VmState {
    stack: UniqueRoot<Vec<Value>>,
    globals: UniqueRoot<HashMap<String, Value>>,
    heap: Heap,
}

pub struct Vm<'a> {
    state: &'a mut VmState,
    chunk: &'a Chunk,
    program_counter: usize,
}

impl VmState {
    pub fn new() -> VmState {
        let mut heap = Heap::new();
        let stack = heap.unique(vec![]);
        let globals = heap.unique(HashMap::new());
        VmState {
            stack,
            globals,
            heap,
        }
    }

    pub fn manage(&mut self, object: Object) -> Gc<RefCell<Object>> {
        self.heap.manage(RefCell::new(object))
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

    pub fn collect(&mut self) {
        self.state.heap.collect();
    }

    pub fn interpret_next(&mut self) -> bool { //TODO Result
        self.program_counter += 1;
        let instr = &self.chunk.instructions()[self.program_counter-1];
        // println!("Instr: {:?} ({})", instr, self.state.stack.len());
        match *instr {
            Instruction::Return => {return false;},
            Instruction::Constant(index) => {
                match self.chunk.constants()[index] {
                    Constant::Number(n) => self.state.push(Value::Number(n)),
                    Constant::String(ref string) => {
                        let obj = self.state.manage(Object::String(string.clone()));
                        self.state.push(Value::Object(obj))
                    },
                }
            },
            Instruction::Print => {
                let value = self.state.pop();
                match value {
                    Value::Number(n) => println!("{}", n),
                    Value::Object(obj) => {
                        match *obj.borrow() {
                            Object::String(ref string) => println!("{}", string),
                        }
                    },
                    Value::True => println!("true"),
                    Value::False => println!("false"),
                    Value::Nil => println!("nil"),
                }
            },
            Instruction::Add => {
                match (self.state.pop(), self.state.pop()) {
                    (Value::Number(b), Value::Number(a)) => self.state.push_number(a+b),
                    (Value::Object(b), Value::Object(a)) => {
                        match (&*a.borrow(), &*b.borrow()) {
                            (Object::String(ref a), Object::String(ref b)) => {
                                let obj = self.state.manage(Object::String(format!("{}{}", a, b)));
                                self.state.push(Value::Object(obj))
                            },
                        }
                    },
                    (_,_) => unimplemented!(),
                }
            },
            Instruction::Subtract => {
                match (self.state.pop(), self.state.pop()) {
                    (Value::Number(b), Value::Number(a)) => self.state.push_number(a-b),
                    (_,_) => unimplemented!(),
                }
            },
            Instruction::Multiply => {
                match (self.state.pop(), self.state.pop()) {
                    (Value::Number(b), Value::Number(a)) => self.state.push_number(a*b),
                    (_, _) => unimplemented!(),
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
                    let value = self.state.globals.get(identifier).cloned(); // We need to clone/copy here because otherwise we end up borrowing before mutable borrow which is not allowed
                    if let Some(value) = value {
                        self.state.push(value);
                    } else {
                        panic!("Runtime error, global not defined"); //TODO else runtime error
                    }
                } else { panic!("String constant expected") } //TODO else runtime error
            },
            Instruction::GetLocal(index) => {
                self.state.push(self.state.stack[index]);
            },
            Instruction::SetLocal(index) => {
                self.state.stack[index] = *self.state.peek();
            }
            Instruction::SetGlobal(index) => {
                if let Constant::String(identifier) = &self.chunk.constants()[index] {
                    let value = *self.state.peek();
                    if !self.state.globals.contains_key(identifier) { panic!("Runtime error, global not defined"); } //TODO else runtime error
                    self.state.globals.insert(identifier.to_string(), value);
                } else { panic!("String constant expected") } //TODO else runtime error
            },
            Instruction::Pop => {
                self.state.pop();
            },
            Instruction::False => {
                self.state.push(Value::False);
            },
            Instruction::Nil => {
                self.state.push(Value::Nil);
            },
            Instruction::True => {
                self.state.push(Value::True);
            },
            Instruction::JumpIfFalse(index) => {
                if is_falsey(&self.state.peek()) {
                    self.program_counter = index;
                }
            },
            Instruction::Jump(index) => {
                self.program_counter = index;
            }
            _ => unimplemented!(),
        }
        true
    }
}


fn is_falsey(value: &Value) -> bool {
    match value {
        Value::False => true,
        Value::Nil => true,
        _ => false,
    }
}
use crate::bytecode::*;
use std::collections::HashMap;
use std::cell::RefCell;
use crate::gc::*;

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
    //Nil, Bool
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
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    heap: Vec<Box<Root<RefCell<Object>>>>, //TODO GcHeap / Heap
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
            heap: vec![],
        }
    }

    //TODO Move to Heap
    pub fn manage(&mut self, object: Object) -> Gc<RefCell<Object>> {
        let root = Box::new(Root::new(RefCell::new(object)));
        let gc = root.as_gc();
        self.heap.push(root);
        gc
    }

    //TODO Move to Heap
    pub fn mark(&mut self) {
        for root in &self.heap {
            root.unmark();
        }
        self.stack.trace();
        self.globals.trace();
    }

    //TODO Move to Heap
    pub fn sweep(&mut self) {
        self.heap.retain(|el| el.marked() );
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
        // println!("before: {:?}", self.state.heap);
        self.state.mark();
        // println!("between: {:?}", self.state.heap);
        self.state.sweep();
        // println!("after: {:?}", self.state.heap);
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
                    (Value::Number(_), Value::Object(_)) => unimplemented!(),
                    (Value::Object(_), Value::Number(_)) => unimplemented!(),
                }
            },
            Instruction::Multiply => {
                match (self.state.pop(), self.state.pop()) {
                    (Value::Number(b), Value::Number(a)) => self.state.push_number(a*b),
                    (Value::Object(_), Value::Object(_)) => unimplemented!(),
                    (Value::Number(_), Value::Object(_)) => unimplemented!(),
                    (Value::Object(_), Value::Number(_)) => unimplemented!(),
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
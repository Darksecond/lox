use std::fmt::Display;
use crate::value::Value;
use std::cell::UnsafeCell;
use lox_gc::{Trace, Tracer};
use crate::stack::Stack;
use crate::array::Array;

pub struct List {
    data: UnsafeCell<Array<Value>>,
}

impl List {
    pub fn new(size: usize) -> Self {
        Self {
            data: UnsafeCell::new(Array::with_contents(Value::NIL, size)),
        }
    }

    pub fn with_stack(arity: usize, stack: &mut Stack) -> Self {
        let list = Self::new(arity as _);

        for index in (0..arity as usize).rev() {
            let value = stack.pop();
            list.set(index, value);
        }

        list
    }

    pub fn get(&self, index: usize) -> Value {
        self.data()[index]
    }

    pub fn set(&self, index: usize, value: Value) {
        self.data_mut()[index] = value;
    }

    pub fn push(&self, value: Value) {
        self.data_mut().push(value);
    }

    pub fn is_valid(&self, index: usize) -> bool {
        index < self.data().len()
    }

    fn data(&self) -> &Array<Value> {
        unsafe {
            &*self.data.get()
        }
    }

    fn data_mut(&self) -> &mut Array<Value> {
        unsafe {
            &mut *self.data.get()
        }
    }
}

unsafe impl Trace for List {
    fn trace(&self, tracer: &mut Tracer) {
        self.data.trace(tracer);
    }
}

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, element) in self.data().iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", element)?;
        }
        write!(f, "]")
    }
}


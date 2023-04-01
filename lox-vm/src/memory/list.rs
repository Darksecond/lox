use std::fmt::Display;
use crate::value::Value;
use std::cell::UnsafeCell;
use crate::gc::Trace;

pub struct List {
    data: UnsafeCell<Vec<Value>>,
}

impl List {
    pub fn new(size: usize) -> Self {
        Self {
            data: UnsafeCell::new(vec![Value::NIL; size]),
        }
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

    fn data(&self) -> &Vec<Value> {
        unsafe {
            &*self.data.get()
        }
    }

    fn data_mut(&self) -> &mut Vec<Value> {
        unsafe {
            &mut *self.data.get()
        }
    }
}

impl Trace for List {
    fn trace(&self) {
        self.data.trace();
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


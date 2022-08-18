use crate::bettervm::Value;
use crate::bettergc::Trace;
use std::ptr;

pub struct Stack {
    top: *mut Value,
    bottom: *mut Value,
    _stack: Box<[Value]>,
}

impl Stack {
    pub fn new(size: usize) -> Self {
        let mut stack = vec![Value::Nil; size].into_boxed_slice();
        Self {
            top: stack.as_mut_ptr(),
            bottom: stack.as_mut_ptr(),
            _stack: stack,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe {
            self.top.offset_from(self.bottom) as usize
        }
    }

    #[inline]
    pub fn truncate(&mut self, top: usize) {
        unsafe {
            self.top = self.bottom.add(top);
        }
    }

    #[inline]
    pub fn push(&mut self, value: Value) {
        unsafe {
            ptr::write(self.top, value);
            self.top = self.top.add(1);
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Value {
        unsafe {
            self.top = self.top.sub(1);
            ptr::read(self.top as _)
        }
    }

    #[inline]
    pub fn rset(&mut self, n: usize, value: Value) {
        unsafe {
            let ptr = self.top.sub(n+1);
            ptr::write(ptr, value);
        }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: Value) {
        unsafe {
            let ptr = self.bottom.add(index);
            ptr::write(ptr, value);
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> &Value {
        unsafe {
            &*self.bottom.add(index)
        }
    }

    #[inline]
    pub fn peek_n(&self, n: usize) -> &Value {
        unsafe {
            &*self.top.sub(n+1)
        }
    }
}

impl Trace for Stack {
    #[inline]
    fn trace(&self) {
        for i in 0..self.len() {
            self.get(i).trace();
        }
    }
}
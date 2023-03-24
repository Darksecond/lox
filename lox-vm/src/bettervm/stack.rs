use crate::bettervm::Value;
use crate::bettergc::Trace;
use std::ptr;

pub struct Stack {
    top: *mut Value,
    bottom: *mut Value,

    stack: *mut [Value],
}

impl Drop for Stack {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.stack));
        }
    }
}

impl Stack {
    pub fn new(size: usize) -> Self {
        let stack = vec![Value::Nil; size].into_boxed_slice();
        let stack = Box::into_raw(stack);
        Self {
            top: stack as *mut Value,
            bottom: stack as *mut Value,
            stack,
        }
    }

    pub fn len(&self) -> usize {
        unsafe {
            self.top.offset_from(self.bottom) as usize
        }
    }

    pub fn truncate(&mut self, top: usize) {
        unsafe {
            self.top = self.bottom.add(top);
        }
    }

    pub fn push(&mut self, value: Value) {
        unsafe {
            ptr::write(self.top, value);
            self.top = self.top.add(1);
        }
    }

    pub fn pop(&mut self) -> Value {
        unsafe {
            self.top = self.top.sub(1);
            ptr::read(self.top)
        }
    }

    pub fn rset(&mut self, n: usize, value: Value) {
        unsafe {
            let ptr = self.top.sub(n+1);
            ptr::write(ptr, value);
        }
    }

    pub fn set(&mut self, index: usize, value: Value) {
        unsafe {
            let ptr = self.bottom.add(index);
            ptr::write(ptr, value);
        }
    }

    pub fn get(&self, index: usize) -> &Value {
        unsafe {
            &*self.bottom.add(index)
        }
    }

    pub fn peek_n(&self, n: usize) -> &Value {
        unsafe {
            &*self.top.sub(n+1)
        }
    }

    pub fn pop_n(&mut self, n: usize) -> Vec<Value> {
        unsafe {
            self.top = self.top.sub(n);
            let slice = std::ptr::slice_from_raw_parts(self.top, n);
            (*slice).into()
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

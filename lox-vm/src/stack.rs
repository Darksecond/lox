use crate::value::Value;
use super::gc::Trace;
use std::ptr;

pub struct StackBlock {
    stack: *mut [Value],
}

impl StackBlock {
    pub fn new(size: usize) -> Self {
        let stack = vec![Value::NIL; size].into_boxed_slice();
        let stack = Box::into_raw(stack);

        Self {
            stack,
        }
    }
}

impl Drop for StackBlock {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.stack));
        }
    }
}

#[derive(Copy, Clone)]
pub struct Stack {
    top: *mut Value,
    bottom: *mut Value,
}

impl Stack {
    pub fn with_block(block: &StackBlock) -> Self {
        Self {
            top: block.stack as *mut Value,
            bottom: block.stack as *mut Value,
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
            ptr::read(self.top)
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
    pub fn get(&self, index: usize) -> Value {
        unsafe {
            let ptr = self.bottom.add(index);
            ptr::read(ptr)
        }
    }

    #[inline]
    pub fn peek_n(&self, n: usize) -> Value {
        unsafe {
            let ptr = self.top.sub(n+1);
            ptr::read(ptr)
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
    fn trace(&self) {
        for i in 0..self.len() {
            self.get(i).trace();
        }
    }
}

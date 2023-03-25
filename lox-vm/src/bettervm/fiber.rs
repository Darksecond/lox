use lox_bytecode::bytecode::Chunk;
use crate::bettervm::memory::*;
use crate::bettergc::{Trace, Gc};
use std::cell::{Cell, UnsafeCell};
use crate::bettervm::stack::Stack;
use crate::bettervm::VmError;
use crate::bettervm::vm::Signal;

pub struct CallFrame {
    pub base_counter: usize,
    pub closure: Gc<Closure>,

    chunk: *const Chunk,
    ip: *const u8,
}

impl Trace for CallFrame {
    #[inline]
    fn trace(&self) {
        self.closure.trace();
    }
}

impl CallFrame {
    pub fn new(closure: Gc<Closure>, base_counter: usize) -> Self {
        let chunk: *const Chunk = closure.function.import.chunk(closure.function.chunk_index);
        let ip = closure.function.import.chunk(closure.function.chunk_index).as_ptr();
        Self {
            base_counter,
            closure,
            chunk,
            ip,
        }
    }

    fn chunk(&self) -> &Chunk {
        // We use unsafe here because it's way faster
        // This is safe, because we have `Root<Closure>` which eventually has a `Gc<Import>`.
        unsafe { &*self.chunk }
    }

    pub fn load_ip(&self) -> *const u8 {
        self.ip
    }

    pub fn store_ip(&mut self, ip: *const u8) {
        self.ip = ip;
    }

    pub fn set_pc(&mut self, value: usize) {
        unsafe {
            self.ip = self.chunk().as_slice().as_ptr().add(value);
        }
    }
}

pub struct Fiber {
    pub parent: Option<Gc<UnsafeCell<Fiber>>>,
    frames: Vec<CallFrame>,
    pub stack: Stack,
    upvalues: Vec<Gc<Cell<Upvalue>>>,
    pub error: Cell<Option<VmError>>,

    // We use a pointer for the current call frame becaeuse this is way faster than using last().
    current_frame: *mut CallFrame,
}

impl Trace for Fiber {
    #[inline]
    fn trace(&self) {
        self.parent.trace();
        self.frames.trace();
        self.stack.trace();
        self.upvalues.trace();
    }
}

impl Fiber {
    pub fn new(parent: Option<Gc<UnsafeCell<Fiber>>>) -> Self {
        Self {
            parent,
            frames: Vec::with_capacity(2048),
            stack: Stack::new(2048),
            upvalues: Vec::with_capacity(2048),
            error: Cell::new(None),

            current_frame: std::ptr::null_mut(),
        }
    }

    pub fn runtime_error(&self, error: VmError) -> Signal {
        self.error.set(Some(error));
        Signal::RuntimeError
    }

    pub fn begin_frame(&mut self, closure: Gc<Closure>) {
        self.frames.push(CallFrame::new(closure, self.stack.len() - closure.function.arity - 1));

        // We don't just offset(1) here because Vec might reallocate contents.
        unsafe {
            self.current_frame = self.frames.as_mut_ptr().add(self.frames.len() - 1);
        }
    }

    pub fn end_frame(&mut self) -> Option<Signal> {
        if self.frames.pop().is_some() {
            unsafe {
                //TODO fix this, the pointer becomes invalid if we end the last frame (when exiting the program)
                self.current_frame = self.current_frame.offset(-1);
            }
            None
        } else {
            Some(self.runtime_error(VmError::FrameEmpty))
        }
    }

    pub fn has_current_frame(&self) -> bool {
        !self.frames.is_empty()
    }

    pub fn current_frame(&self) -> &CallFrame {
        unsafe {
            &*self.current_frame
        }
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        unsafe {
            &mut *self.current_frame
        }
    }

    pub fn push_upvalue(&mut self, upvalue: Gc<Cell<Upvalue>>) {
        self.upvalues.push(upvalue);
    }

    pub fn close_upvalues(&mut self, index: usize) {
        for upvalue in self.upvalues.iter() {
            if let Some(index) = upvalue.get().is_open_with_range(index) {
                let value = *self.stack.get(index);
                upvalue.set(Upvalue::Closed(value));
            }
        }

        for index in (0..self.upvalues.len()).rev() {
            let upvalue = unsafe { self.upvalues.get_unchecked(index) };
            if !upvalue.get().is_open() {
                self.upvalues.swap_remove(index);
            }
        }
    }

    pub fn find_upvalue_by_index(&self, index: usize) -> Gc<Cell<Upvalue>> {
        let frame = self.current_frame();
        frame.closure.upvalues[index]
    }

    pub fn find_open_upvalue_with_index(&self, index: usize) -> Option<Gc<Cell<Upvalue>>> {
        for upvalue in self.upvalues.iter().rev() {
            if upvalue.get().is_open_with_index(index) {
                return Some(*upvalue);
            }
        }

        None
    }

    pub fn resolve_upvalue_into_value(&self, upvalue: Gc<Cell<Upvalue>>) -> Value {
        match upvalue.get() {
            Upvalue::Closed(value) => value,
            Upvalue::Open(index) => *self.stack.get(index),
        }
    }

    pub fn set_upvalue(&mut self, upvalue: Gc<Cell<Upvalue>>, new_value: Value) {
        match upvalue.get() {
            Upvalue::Closed(_) => upvalue.set(Upvalue::Closed(new_value)),
            Upvalue::Open(index) => self.stack.set(index, new_value),
        }
    }
}

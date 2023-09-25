use crate::memory::*;
use crate::value::Value;
use lox_gc::{Trace, Gc, Tracer};
use std::cell::Cell;
use crate::stack::{Stack, StackBlock};
use crate::VmError;
use crate::runtime::Signal;
use arrayvec::ArrayVec;
use crate::array::Array;

pub struct CallFrame {
    pub base_counter: usize,
    pub closure: Gc<Closure>,

    ip: Cell<*const u8>,
}

unsafe impl Trace for CallFrame {
    #[inline]
    fn trace(&self, tracer: &mut Tracer) {
        self.closure.trace(tracer);
    }
}

impl CallFrame {
    pub fn new(object: Gc<Closure>, base_counter: usize) -> Self {
        let ip = object.function.import.chunk(object.function.chunk_index).as_ptr();
        Self {
            base_counter,
            closure: object,
            ip: Cell::new(ip),
        }
    }

    #[inline(always)]
    pub fn load_ip(&self) -> *const u8 {
        self.ip.get()
    }

    #[inline(always)]
    pub fn store_ip(&self, ip: *const u8) {
        self.ip.set(ip);
    }
}

pub struct Fiber {
    pub stack: Stack,
    frames: ArrayVec<CallFrame, 256>,
    stack_block: StackBlock,
    upvalues: Array<Gc<Cell<Upvalue>>>,
    error: Option<VmError>,
}

unsafe impl Trace for Fiber {
    fn trace(&self, tracer: &mut Tracer) {
        self.frames.trace(tracer);
        self.stack_block.trace(tracer);
        self.stack.trace(tracer);
        self.upvalues.trace(tracer);
    }
}

impl Fiber {
    pub fn new() -> Self {
        let block = StackBlock::new(2048);
        let frames = ArrayVec::new();
        Self {
            frames,
            stack: Stack::with_block(&block),
            stack_block: block,
            upvalues: Array::with_capacity(128),
            error: None,
        }
    }

    #[cold]
    pub fn error(&self) -> Option<VmError> {
        self.error
    }

    #[cold]
    pub fn runtime_error(&mut self, error: VmError) -> Signal {
        //panic!("runtime error: {:?}", error);

        self.error = Some(error);
        Signal::RuntimeError
    }

    pub fn begin_frame(&mut self, closure: Gc<Closure>) {
        let base_counter = self.stack.len() - closure.function.arity - 1;

        self.frames.push(CallFrame::new(closure, base_counter));
    }

    pub fn end_frame(&mut self) -> Option<Signal> {
        if self.frames.pop().is_some() {
            None
        } else {
            Some(self.runtime_error(VmError::FrameEmpty))
        }
    }

    #[inline]
    pub fn has_current_frame(&self) -> bool {
        self.frames.len() > 0
    }

    #[inline]
    pub fn current_frame(&self) -> &CallFrame {
        unsafe {
            self.frames.last().unwrap_unchecked()
        }
    }

    #[inline]
    pub fn current_import(&self) -> Gc<Import> {
        self.current_frame().closure.function.import
    }

    pub fn push_upvalue(&mut self, upvalue: Gc<Cell<Upvalue>>) {
        self.upvalues.push(upvalue);
    }

    pub fn close_upvalues(&mut self, index: usize) {
        for upvalue in self.upvalues.iter() {
            if let Some(index) = upvalue.get().is_open_with_range(index) {
                let value = self.stack.get(index);
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
            Upvalue::Open(index) => self.stack.get(index),
        }
    }

    pub fn set_upvalue(&mut self, upvalue: Gc<Cell<Upvalue>>, new_value: Value) {
        match upvalue.get() {
            Upvalue::Closed(_) => upvalue.set(Upvalue::Closed(new_value)),
            Upvalue::Open(index) => self.stack.set(index, new_value),
        }
    }
}

use crate::memory::*;
use crate::value::Value;
use lox_gc::{Trace, Gc, Tracer};
use std::cell::{Cell, UnsafeCell};
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
    pub parent: Option<Gc<Fiber>>,
    frames: UnsafeCell<ArrayVec<CallFrame, 256>>,
    stack: UnsafeCell<Stack>,
    stack_block: StackBlock,
    upvalues: UnsafeCell<Array<Gc<Cell<Upvalue>>>>,
    error: Cell<Option<VmError>>,
}

unsafe impl Trace for Fiber {
    fn trace(&self, tracer: &mut Tracer) {
        self.parent.trace(tracer);
        self.frames.trace(tracer);
        self.stack_block.trace(tracer);
        self.stack.trace(tracer);
        self.upvalues.trace(tracer);
    }
}

impl Fiber {
    pub fn new(parent: Option<Gc<Fiber>>) -> Self {
        let block = StackBlock::new(2048);
        let frames = ArrayVec::new();
        Self {
            parent,
            frames: UnsafeCell::new(frames),
            stack: UnsafeCell::new(Stack::with_block(&block)),
            stack_block: block,
            upvalues: UnsafeCell::new(Array::with_capacity(128)),
            error: Cell::new(None),
        }
    }

    #[inline]
    pub fn with_stack<T>(&self, func: impl FnOnce(&mut Stack) -> T) -> T {
        let stack = unsafe {
            &mut *self.stack.get()
        };

        func(stack)
    }

    #[cold]
    pub fn error(&self) -> Option<VmError> {
        self.error.get()
    }

    #[cold]
    pub fn runtime_error(&self, error: VmError) -> Signal {
        //panic!("runtime error: {:?}", error);

        self.error.set(Some(error));
        Signal::RuntimeError
    }

    pub fn begin_frame(&self, closure: Gc<Closure>) {
        let base_counter = self.with_stack(|stack| {
            stack.len() - closure.function.arity - 1
        });

        let frames = unsafe { &mut *self.frames.get() };

        frames.push(CallFrame::new(closure, base_counter));
    }

    pub fn end_frame(&self) -> Option<Signal> {
        let frames = unsafe { &mut *self.frames.get() };

        if frames.pop().is_some() {
            None
        } else {
            Some(self.runtime_error(VmError::FrameEmpty))
        }
    }

    #[inline]
    pub fn has_current_frame(&self) -> bool {
        let frames = unsafe { &*self.frames.get() };

        frames.len() > 0
    }

    #[inline]
    pub fn current_frame(&self) -> &CallFrame {
        let frames = unsafe { &*self.frames.get() };

        unsafe {
            frames.last().unwrap_unchecked()
        }
    }

    #[inline]
    pub fn current_import(&self) -> Gc<Import> {
        self.current_frame().closure.function.import
    }

    pub fn push_upvalue(&self, upvalue: Gc<Cell<Upvalue>>) {
        let upvalues = unsafe { &mut *self.upvalues.get() };
        upvalues.push(upvalue);
    }

    pub fn close_upvalues(&self, index: usize) {
        let upvalues = unsafe { &mut *self.upvalues.get() };
        for upvalue in upvalues.iter() {
            if let Some(index) = upvalue.get().is_open_with_range(index) {
                self.with_stack(|stack| {
                    let value = stack.get(index);
                    upvalue.set(Upvalue::Closed(value));
                });
            }
        }

        for index in (0..upvalues.len()).rev() {
            let upvalue = unsafe { upvalues.get_unchecked(index) };
            if !upvalue.get().is_open() {
                upvalues.swap_remove(index);
            }
        }
    }

    pub fn find_upvalue_by_index(&self, index: usize) -> Gc<Cell<Upvalue>> {
        let frame = self.current_frame();
        frame.closure.upvalues[index]
    }

    pub fn find_open_upvalue_with_index(&self, index: usize) -> Option<Gc<Cell<Upvalue>>> {
        let upvalues = unsafe { &*self.upvalues.get() };
        for upvalue in upvalues.iter().rev() {
            if upvalue.get().is_open_with_index(index) {
                return Some(*upvalue);
            }
        }

        None
    }

    pub fn resolve_upvalue_into_value(&self, upvalue: Gc<Cell<Upvalue>>) -> Value {
        match upvalue.get() {
            Upvalue::Closed(value) => value,
            Upvalue::Open(index) => self.with_stack(|stack| stack.get(index)),
        }
    }

    pub fn set_upvalue(&self, upvalue: Gc<Cell<Upvalue>>, new_value: Value) {
        match upvalue.get() {
            Upvalue::Closed(_) => upvalue.set(Upvalue::Closed(new_value)),
            Upvalue::Open(index) => self.with_stack(|stack| stack.set(index, new_value)),
        }
    }
}

use crate::memory::*;
use crate::value::Value;
use super::gc::{Trace, Gc};
use std::cell::{Cell, UnsafeCell};
use crate::stack::{Stack, StackBlock};
use crate::VmError;
use crate::runtime::Signal;

pub struct CallFrame {
    pub base_counter: usize,
    pub closure: Gc<Object<Closure>>,

    ip: Cell<*const u8>,
}

impl Trace for CallFrame {
    #[inline]
    fn trace(&self) {
        self.closure.trace();
    }
}

impl CallFrame {
    pub fn new(object: Gc<Object<Closure>>, base_counter: usize) -> Self {
        let ip = object.function.import.chunk(object.function.chunk_index).as_ptr();
        Self {
            base_counter,
            closure: object,
            ip: Cell::new(ip),
        }
    }

    #[inline]
    pub fn load_ip(&self) -> *const u8 {
        self.ip.get()
    }

    #[inline]
    pub fn store_ip(&self, ip: *const u8) {
        self.ip.set(ip);
    }
}

pub struct Fiber {
    pub parent: Option<Gc<Fiber>>,
    frames: UnsafeCell<Vec<CallFrame>>,
    stack: UnsafeCell<Stack>,
    _stack_block: StackBlock,
    upvalues: UnsafeCell<Vec<Gc<Cell<Upvalue>>>>,
    error: Cell<Option<VmError>>,

    // We use a pointer for the current call frame becaeuse this is way faster than using last().
    current_frame: Cell<*mut CallFrame>,
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
    pub fn new(parent: Option<Gc<Fiber>>) -> Self {
        let block = StackBlock::new(2028);
        let frames = Vec::with_capacity(2048);
        Self {
            parent,
            frames: UnsafeCell::new(frames),
            stack: UnsafeCell::new(Stack::with_block(&block)),
            _stack_block: block,
            upvalues: UnsafeCell::new(Vec::with_capacity(2048)),
            error: Cell::new(None),

            current_frame: Cell::new(std::ptr::null_mut()),
        }
    }

    #[inline]
    pub fn with_stack<T>(&self, mut func: impl FnMut(&mut Stack) -> T) -> T {
        let stack = unsafe {
            &mut *self.stack.get()
        };

        func(stack)
    }

    pub fn error(&self) -> Option<VmError> {
        self.error.get()
    }

    pub fn runtime_error(&self, error: VmError) -> Signal {
        //panic!("runtime error: {:?}", error);
        self.error.set(Some(error));
        Signal::RuntimeError
    }

    pub fn begin_frame(&self, closure: Gc<Object<Closure>>) {
        let base_counter = self.with_stack(|stack| {
            stack.len() - closure.function.arity - 1
        });

        let frames = unsafe { &mut *self.frames.get() };

        frames.push(CallFrame::new(closure, base_counter));

        // We don't just offset(1) here because Vec might reallocate contents.
        unsafe {
            self.current_frame.set(frames.as_mut_ptr().add(frames.len() - 1));
        }
    }

    pub fn end_frame(&self) -> Option<Signal> {
        let frames = unsafe { &mut *self.frames.get() };

        if frames.pop().is_some() {
            if frames.is_empty() {
                self.current_frame.set(std::ptr::null_mut());
            } else {
                unsafe {
                    self.current_frame.set(self.current_frame.get().offset(-1));
                }
            }
            None
        } else {
            Some(self.runtime_error(VmError::FrameEmpty))
        }
    }

    #[inline]
    pub fn has_current_frame(&self) -> bool {
        self.current_frame.get() != std::ptr::null_mut()
    }

    #[inline]
    pub fn current_frame(&self) -> &CallFrame {
        unsafe {
            &*self.current_frame.get()
        }
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

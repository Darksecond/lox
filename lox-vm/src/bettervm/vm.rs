use lox_bytecode::bytecode::Chunk;
use lox_bytecode::opcode;

use super::context::VmContext;
use super::memory::*;
use super::stack::Stack;
use crate::bettergc::{Gc, Trace};
use std::cell::Cell;

#[derive(PartialEq)]
pub enum InterpretResult {
    Done,
    More,
}

//TODO thiserror
#[derive(Debug)]
pub enum VmError {
    StackEmpty,
    FrameEmpty,
    StringConstantExpected,
    GlobalNotDefined,
    InvalidCallee,
    IncorrectArity,
    UnexpectedConstant,
    ClosureConstantExpected,
    UnexpectedValue,
    UndefinedProperty,
}

struct CallFrame {
    program_counter: usize,
    base_counter: usize,
    closure: Gc<Closure>,

    chunk: *const Chunk,
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
        Self {
            program_counter: 0,
            base_counter,
            closure,
            chunk,
        }
    }

    #[inline]
    fn chunk(&self) -> &Chunk {
        // We use unsafe here because it's way faster
        // This is safe, because we have `Root<Closure>` which eventually has a `Gc<Import>`.
        unsafe { &*self.chunk }
    }

    #[inline]
    pub fn next_u8(&mut self) -> u8 {
        let instr = self.chunk().get_u8(self.program_counter);
        self.program_counter += 1;
        instr
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        let instr = self.chunk().get_u32(self.program_counter);
        self.program_counter += 4;
        instr
    }

    #[inline]
    pub fn set_pc(&mut self, value: usize) {
        self.program_counter = value;
    }
}

pub struct Fiber {
    frames: Vec<CallFrame>,
    stack: Stack,
    upvalues: Vec<Gc<Cell<Upvalue>>>,

    // We use a pointer for the current call frame becaeuse this is way faster than using last().
    current_frame: *mut CallFrame,
}

impl Trace for Fiber {
    #[inline]
    fn trace(&self) {
        self.frames.trace();
        self.stack.trace();
        self.upvalues.trace();
    }
}

impl Fiber {
    pub fn with_closure(closure: Gc<Closure>) -> Self {
        let mut fiber = Self {
            frames: Vec::with_capacity(2048),
            stack: Stack::new(2048),
            upvalues: Vec::with_capacity(2048),
            current_frame: std::ptr::null_mut(),
        };

        fiber.stack.push(Value::Closure(closure));
        fiber.begin_frame(closure);

        fiber
    }

    fn current_import(&self) -> Gc<Import> {
        self.current_frame().closure.function.import
    }

    pub fn set_native_fn(&mut self, identifier: &str, code: fn(&[Value]) -> Value, context: &mut VmContext) {
        let native_function = NativeFunction {
            name: identifier.to_string(),
            code,
        };

        let identifier = context.intern(identifier);
        let root = context.manage(native_function);
        self.current_import().set_global(identifier, Value::NativeFunction(root))
    }

    #[inline]
    pub fn interpret_next(&mut self, context: &mut VmContext) -> Result<InterpretResult, VmError> {
        let instr = self.next_u8();

        match instr {
            opcode::CONSTANT => {
                let index = self.next_u32() as _;
                self.op_constant(index, context);
            },
            opcode::IMPORT => {
                let _index: usize = self.next_u32() as _;
                unimplemented!()
            },
            opcode::IMPORT_GLOBAL => {
                let _index: usize = self.next_u32() as _;
                unimplemented!()
            },
            opcode::CLOSURE => {
                let index = self.next_u32() as _;
                self.op_closure(index, context);
            }
            opcode::CLASS => {
                let index = self.next_u8() as _;
                self.op_class(index, context);
            }
            opcode::METHOD => {
                let index = self.next_u32() as _;
                self.op_method(index)?;
            },
            opcode::SET_PROPERTY => {
                let index = self.next_u32() as _;
                self.op_set_property(index)?;
            }
            opcode::GET_PROPERTY => {
                let index = self.next_u32() as _;
                self.op_get_property(index, context)?;
            }
            opcode::PRINT => {
                self.op_print(context);
            },
            opcode::NIL => {
                self.op_nil();
            },
            opcode::RETURN => {
                if let Some(result) = self.op_return()? {
                    return Ok(result);
                }
            }
            opcode::ADD => {
                self.op_add(context)?;
            },
            opcode::SUBTRACT => {
                self.op_subtract()?;
            },
            opcode::MULTIPLY => {
                self.op_multiply()?;
            },
            opcode::DIVIDE => {
                self.op_divide()?;
            },
            opcode::POP => {
                self.op_pop();
            }
            opcode::DEFINE_GLOBAL => {
                let index = self.next_u32() as _;
                self.op_define_global(index);
            }
            opcode::GET_GLOBAL => {
                let index = self.next_u32() as _;
                self.op_get_global(index)?;
            }
            opcode::SET_GLOBAL => {
                let index = self.next_u32() as _;
                self.op_set_global(index)?;
            }
            opcode::GET_LOCAL => {
                let index = self.next_u32() as _;
                self.op_get_local(index);
            }
            opcode::SET_LOCAL => {
                let index = self.next_u32() as _;
                self.op_set_local(index);
            }
            opcode::TRUE => {
                self.op_bool(true);
            },
            opcode::FALSE => {
                self.op_bool(false);
            },
            opcode::JUMP_IF_FALSE => {
                let to = self.next_u32() as _;
                self.op_jump_if_false(to);
            },
            opcode::JUMP => {
                let to = self.next_u32() as _;
                self.op_jump(to);
            },
            opcode::LESS => {
                self.op_less()?;
            },
            opcode::GREATER => {
                self.op_greater()?;
            },
            opcode::EQUAL => {
                self.op_equal();
            }
            opcode::CALL => {
                let arity = self.next_u8() as _;
                self.op_call(arity, context)?;
            }
            opcode::NEGATE => {
                self.op_negate()?;
            },
            opcode::NOT => {
                self.op_not();
            }
            opcode::GET_UPVALUE => {
                let index = self.next_u32() as _;
                self.op_get_upvalue(index);
            }
            opcode::SET_UPVALUE => {
                let index = self.next_u32() as _;
                self.op_set_upvalue(index);
            }
            opcode::CLOSE_UPVALUE => {
                self.op_close_upvalue();
            }
            opcode::INVOKE => {
                let arity = self.next_u8() as _;
                let index = self.next_u32() as _;

                self.op_invoke(index, arity, context)?;
            }
            _ => unreachable!(),
        }

        Ok(InterpretResult::More)
    }

    fn op_jump(&mut self, to: usize) {
        self.current_frame_mut().set_pc(to);
    }

    fn op_jump_if_false(&mut self, to: usize) {
        if self.stack.peek_n(0).is_falsey() {
            self.current_frame_mut().set_pc(to);
        }
    }

    fn op_class(&mut self, index: usize, context: &mut VmContext) {
        let current_import = self.current_import();
        let class = current_import.class(index);
        let class = context.manage(Class::new(class.name.clone()));
        self.stack.push(Value::Class(class));
    }

    fn op_bool(&mut self, value: bool) {
        self.stack.push(Value::Boolean(value));
    }

    fn op_nil(&mut self) {
        self.stack.push(Value::Nil);
    }

    //TODO Rewrite if's to improve error handling
    //TODO Pretty sure it leaves the stack clean, but double check
    fn op_method(&mut self, index: usize) -> Result<(), VmError> {
        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        if let Value::Class(class) = self.stack.peek_n(1) {
            if let Value::Closure(closure) = self.stack.peek_n(0) {
                class.set_method(identifier, *closure);
            } else {
                return Err(VmError::UnexpectedConstant);
            }
        } else {
            return Err(VmError::UnexpectedConstant);
        }

        self.stack.pop();
        Ok(())
    }

    fn op_return(&mut self) -> Result<Option<InterpretResult>, VmError> {
        let result = self.stack.pop();
        let frame = self.end_frame()?;

        self.close_upvalues(frame.base_counter..self.stack.len());

        self.stack.truncate(frame.base_counter);

        if self.frames.len() == 0 {
            // We are done interpreting, don't push a result as it'll be nil
            return Ok(Some(InterpretResult::Done));
        }

        self.stack.push(result);

        Ok(None)
    }

    fn op_pop(&mut self) {
        self.stack.pop();
    }

    fn op_call(&mut self, arity: usize, context: &mut VmContext) -> Result<(), VmError> {
        let callee = *self.stack.peek_n(arity);
        self.call(arity, callee, context)?;
        Ok(())
    }

    fn op_constant(&mut self, index: usize, context: &mut VmContext) {
        use crate::bytecode::Constant;
        let current_import = self.current_import();
        match current_import.constant(index) {
            Constant::Number(n) => self.stack.push(Value::Number(*n)),
            Constant::String(string) => self.push_string(string, context),
        }
    }

    fn op_greater(&mut self) -> Result<(), VmError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.stack.push((a > b).into()),
            _ => return Err(VmError::UnexpectedValue),
        }

        Ok(())
    }

    fn op_less(&mut self) -> Result<(), VmError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.stack.push((a < b).into()),
            _ => return Err(VmError::UnexpectedValue),
        }

        Ok(())
    }

    fn op_negate(&mut self) -> Result<(), VmError> {
        match self.stack.pop() {
            Value::Number(n) => self.stack.push(Value::Number(-n)),
            _ => return Err(VmError::UnexpectedValue),
        }

        Ok(())
    }

    fn op_not(&mut self) {
        let is_falsey = self.stack.pop().is_falsey();
        self.stack.push(is_falsey.into());
    }

    fn op_divide(&mut self) -> Result<(), VmError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.stack.push(Value::Number(a / b)),
            _ => return Err(VmError::UnexpectedValue),
        }

        Ok(())
    }

    fn op_multiply(&mut self) -> Result<(), VmError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.stack.push(Value::Number(a * b)),
            _ => return Err(VmError::UnexpectedValue),
        }

        Ok(())
    }

    fn op_subtract(&mut self) -> Result<(), VmError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.stack.push(Value::Number(a - b)),
            _ => return Err(VmError::UnexpectedValue),
        }

        Ok(())
    }

    fn op_add(&mut self, context: &mut VmContext) -> Result<(), VmError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.stack.push(Value::Number(a + b)),
            (Value::String(b), Value::String(a)) => self.push_string(format!("{}{}", a, b), context),
            _ => return Err(VmError::UnexpectedValue),
        }

        Ok(())
    }

    fn op_get_upvalue(&mut self, index: usize) {
        let upvalue = self.current_frame().closure.upvalues[index];
        self.stack.push(self.resolve_upvalue_into_value(upvalue));
    }

    fn op_set_upvalue(&mut self, index: usize) {
        let value = self.stack.peek_n(0);
        let upvalue = self.current_frame().closure.upvalues[index];
        self.set_upvalue(upvalue, *value);
    }

    fn op_close_upvalue(&mut self) {
        let index = self.stack.len() - 1;
        self.close_upvalues(index..index+1);
        self.stack.pop();
    }

    //TODO consider redesigning
    fn op_print(&mut self, context: &mut VmContext) {
        (context.print)(&format!("{}", self.stack.pop()));
    }

    fn op_get_local(&mut self, index: usize) {
        let index = self.current_frame().base_counter + index;
        self.stack.push(*self.stack.get(index));
    }

    fn op_set_local(&mut self, index: usize) {
        let index = self.current_frame().base_counter + index;
        let value = self.stack.peek_n(0);
        self.stack.set(index, *value);
    }

    fn op_equal(&mut self) {
        let b = self.stack.pop();
        let a = self.stack.pop();

        if Value::is_same_type(&a, &b) {
            match (b, a) {
                (Value::Number(b), Value::Number(a)) => self.stack.push((a == b).into()),
                (Value::Boolean(b), Value::Boolean(a)) => self.stack.push((a == b).into()),
                (Value::String(b), Value::String(a)) => self.stack.push((*a == *b).into()),
                (Value::Closure(b), Value::Closure(a)) => self.stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::NativeFunction(b), Value::NativeFunction(a)) => self.stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Nil, Value::Nil) => self.stack.push(true.into()),
                (Value::BoundMethod(b), Value::BoundMethod(a)) => self.stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Class(b), Value::Class(a)) => self.stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Instance(b), Value::Instance(a)) => self.stack.push((Gc::ptr_eq(&a, &b)).into()),
                _ => unimplemented!(),
            };
        } else {
            self.stack.push(false.into())
        }
    }

    fn op_define_global(&mut self, index: usize) {
        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = self.stack.pop();
        current_import.set_global(identifier, value);
    }

    fn op_get_global(&mut self, index: usize) -> Result<(), VmError> {
        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = current_import.global(identifier);
        if let Some(value) = value {
            self.stack.push(value);
        } else {
            return Err(VmError::GlobalNotDefined);
        }

        Ok(())
    }

    fn op_set_global(&mut self, index: usize) -> Result<(), VmError> {
        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = self.stack.peek_n(0);
        if current_import.has_global(identifier) {
            current_import.set_global(identifier, *value);
        } else {
            return Err(VmError::GlobalNotDefined);
        }

        Ok(())
    }

    fn op_set_property(&mut self, index: usize) -> Result<(), VmError> {
        let current_import = self.current_import();
        let property = current_import.symbol(index);
        if let Value::Instance(instance) = self.stack.peek_n(1) {
            instance.set_field(property, *self.stack.peek_n(0));

            let value = self.stack.pop();
            self.stack.pop();
            self.stack.push(value);
        } else {
            return Err(VmError::UnexpectedValue);
        }

        Ok(())
    }

    fn op_get_property(&mut self, index: usize, context: &mut VmContext) -> Result<(), VmError> {
        let current_import = self.current_import();
        let property = current_import.symbol(index);
        if let Value::Instance(instance) = self.stack.pop() {
            if let Some(value) = instance.field(&property) {
                self.stack.push(*value);
            } else if let Some(method) = instance.class.method(&property) {
                let bind = context.manage(BoundMethod {
                    receiver: instance,
                    method: *method,
                });
                self.stack.push(Value::BoundMethod(bind));
            } else {
                return Err(VmError::UndefinedProperty);
            };
        } else {
            return Err(VmError::UnexpectedValue);
        }
        Ok(())
    }

    fn op_closure(&mut self, index: usize, context: &mut VmContext) {
        let current_import = self.current_import();
        let closure = current_import.closure(index);
        let base = self.current_frame().base_counter;
        let upvalues = closure
            .upvalues
            .iter()
            .map(|u| {
                match u {
                    crate::bytecode::Upvalue::Local(index) => {
                        let index = base + *index;

                        if let Some(upvalue) = self.find_open_upvalue_with_index(index)
                        {
                            upvalue
                        } else {
                            let root = context.manage(Cell::new(Upvalue::Open(index)));
                            self.upvalues.push(root);
                            root
                        }
                    }
                    crate::bytecode::Upvalue::Upvalue(u) => {
                        self.find_upvalue_by_index(*u)
                    }
                }
            })
        .collect();

        let closure_root = context.manage(Closure {
            function: Function::new(&closure.function, current_import),
            upvalues,
        });
        self.stack.push(Value::Closure(closure_root));
    }

    fn op_invoke(&mut self, index: usize, arity: usize, context: &mut VmContext) -> Result<(), VmError> {
        let current_import = self.current_import();
        let property = current_import.symbol(index);
        if let Value::Instance(instance) = *self.stack.peek_n(arity) {
            if let Some(value) = instance.field(&property) {
                self.stack.rset(arity, *value);
                self.call(arity, *value, context)?;
            } else if let Some(method) = instance.class.method(&property) {
                if method.function.arity != arity {
                    return Err(VmError::IncorrectArity);
                }
                self.begin_frame(*method);
            } else {
                return Err(VmError::UndefinedProperty);
            };
        } else {
            return Err(VmError::UnexpectedValue);
        }

        Ok(())
    }

    //TODO investigate
    fn close_upvalues(&mut self, indexes: std::ops::Range<usize>) {
        for index in indexes {
            for upvalue in self.upvalues.iter() {
                if upvalue.get().is_open_with_index(index) {
                    let value = *self.stack.get(index);
                    upvalue.replace(Upvalue::Closed(value));
                }
            }
        }

        self.upvalues.retain(|u| u.get().is_open());
    }

    fn find_upvalue_by_index(&self, index: usize) -> Gc<Cell<Upvalue>> {
        let frame = self.current_frame();
        frame.closure.upvalues[index]
    }

    fn find_open_upvalue_with_index(&self, index: usize) -> Option<Gc<Cell<Upvalue>>> {
        for upvalue in self.upvalues.iter().rev() {
            if upvalue.get().is_open_with_index(index) {
                return Some(*upvalue);
            }
        }

        None
    }

    fn resolve_upvalue_into_value(&self, upvalue: Gc<Cell<Upvalue>>) -> Value {
        match upvalue.get() {
            Upvalue::Closed(value) => value,
            Upvalue::Open(index) => *self.stack.get(index),
        }
    }

    fn set_upvalue(&mut self, upvalue: Gc<Cell<Upvalue>>, new_value: Value) {
        match upvalue.get() {
            Upvalue::Closed(_) => upvalue.set(Upvalue::Closed(new_value)),
            Upvalue::Open(index) => self.stack.set(index, new_value),
        }
    }

    //TODO Reduce duplicate code paths
    fn call(&mut self, arity: usize, callee: Value, outer: &mut VmContext) -> Result<(), VmError> {
        match callee {
            Value::Closure(callee) => {
                if callee.function.arity != arity {
                    return Err(VmError::IncorrectArity);
                }
                self.begin_frame(callee);
            }
            Value::NativeFunction(callee) => {
                let mut args = self.pop_n(arity);
                args.reverse();
                self.stack.pop(); // discard callee
                let result = (callee.code)(&args);
                self.stack.push(result);
            }
            Value::Class(class) => {
                let instance = outer.manage(Instance::new(class));
                self.stack.rset(arity, Value::Instance(instance));

                let init_symbol = outer.intern("init"); //TODO move to constructor
                if let Some(initializer) = class.method(&init_symbol) {
                    if initializer.function.arity != arity {
                        return Err(VmError::IncorrectArity);
                    }
                    self.begin_frame(*initializer);
                } else if arity != 0 {
                    return Err(VmError::IncorrectArity); // Arity must be 0 without initializer
                }
            }
            Value::BoundMethod(bind) => {
                let callee = bind.method;
                if callee.function.arity != arity {
                    return Err(VmError::IncorrectArity);
                }
                self.stack.rset(arity, Value::Instance(bind.receiver));
                self.begin_frame(callee);
            },
            _ => return Err(VmError::InvalidCallee),
        }

        Ok(())
    }

    fn current_frame(&self) -> &CallFrame {
        unsafe {
            &*self.current_frame
        }
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        unsafe {
            &mut *self.current_frame
        }
    }

    fn push_string(&mut self, string: impl Into<String>, outer: &mut VmContext) {
        let root = outer.manage(string.into());
        self.stack.push(Value::String(root));
    }

    //TODO investigate
    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        let mut result = Vec::with_capacity(n);
        for _ in 0..n {
            result.push(self.stack.pop());
        }

        result
    }

    fn begin_frame(&mut self, closure: Gc<Closure>) {
        self.frames.push(CallFrame::new(closure, self.stack.len() - closure.function.arity - 1));

        // We don't just offset(1) here because Vec might reallocate contents.
        unsafe {
            self.current_frame = self.frames.as_mut_ptr().add(self.frames.len() - 1);
        }
    }

    fn end_frame(&mut self) -> Result<CallFrame, VmError> {
        let frame = self.frames.pop().ok_or(VmError::FrameEmpty)?;

        // This might result in a invalid pointer 
        // because we might point to 1 below the Vector if it's empty.
        unsafe {
            self.current_frame = self.current_frame.offset(-1);
        }
        Ok(frame)
    }

    fn next_u32(&mut self) -> u32 {
        self.current_frame_mut().next_u32()
    }

    fn next_u8(&mut self) -> u8 {
        self.current_frame_mut().next_u8()
    }
}

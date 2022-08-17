use lox_bytecode::bytecode::Chunk;
use lox_bytecode::opcode;

use super::context::VmContext;
use super::memory::*;
use super::stack::Stack;
use crate::bettergc::{Gc, Trace};
use std::cell::Cell;
use std::{cell::RefCell, io::Write};
use fxhash::FxHashMap;

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
}

impl Trace for Fiber {
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
        };

        fiber.push(Value::Closure(closure));
        fiber.begin_frame(closure);

        fiber
    }

    fn current_import(&self) -> Gc<Import> {
        self.current_frame().closure.function.import
    }

    pub fn set_native_fn<W: Write>(&mut self, identifier: &str, code: fn(&[Value]) -> Value, context: &mut VmContext<W>) {
        let native_function = NativeFunction {
            name: identifier.to_string(),
            code,
        };

        let identifier = context.intern(identifier);
        let root = context.manage(native_function);
        self.current_import().set_global(identifier, Value::NativeFunction(root.as_gc()))
    }

    #[inline(always)]
    pub fn interpret_next<W: Write>(&mut self, context: &mut VmContext<W>) -> Result<InterpretResult, VmError> {
        let current_import = self.current_import();

        let instr = {
            let frame = self.current_frame_mut();
            frame.next_u8()
        };

        match instr {
            opcode::CONSTANT => {
                let index = self.current_frame_mut().next_u32() as _;

                self.op_constant(index, context);
            },

            opcode::IMPORT => {
                let _index: usize = self.current_frame_mut().next_u32() as _;
                unimplemented!()
            },
            opcode::IMPORT_GLOBAL => {
                let _index: usize = self.current_frame_mut().next_u32() as _;
                unimplemented!()
            },

            opcode::CLOSURE => {
                let index = self.current_frame_mut().next_u32() as _;
                self.op_closure(index, context);
            }
            opcode::CLASS => {
                let index = self.current_frame_mut().next_u32() as _;

                let class = current_import.class(index);
                let class = context.manage(RefCell::new(Class {
                    name: class.name.clone(),
                    methods: FxHashMap::default(),
                }));
                self.push(Value::Class(class.as_gc()));
            }
            //TODO Rewrite if's to improve error handling
            //TODO Pretty sure it leaves the stack clean, but double check
            opcode::METHOD => {
                let index = self.current_frame_mut().next_u32() as _;

                let identifier = current_import.symbol(index);
                if let Value::Class(class) = self.peek_n(1) {
                    if let Value::Closure(closure) = self.peek_n(0) {
                        class.borrow_mut().methods.insert(identifier, *closure);
                    } else {
                        return Err(VmError::UnexpectedConstant);
                    }
                } else {
                    return Err(VmError::UnexpectedConstant);
                }

                self.pop();
            },
            opcode::SET_PROPERTY => {
                let index = self.current_frame_mut().next_u32() as _;
                self.op_set_property(index)?;
            }
            opcode::GET_PROPERTY => {
                let index = self.current_frame_mut().next_u32() as _;
                self.op_get_property(index, context)?;
            }
            opcode::PRINT => {
                self.op_print(context);
            },
            opcode::NIL => self.push(Value::Nil),
            opcode::RETURN => {
                let result = self.pop();
                let frame = self.frames.pop().ok_or(VmError::FrameEmpty)?;

                self.close_upvalues(frame.base_counter..self.stack.len());

                self.stack.truncate(frame.base_counter);

                if self.frames.len() == 0 {
                    // We are done interpreting, don't push a result as it'll be nil
                    return Ok(InterpretResult::Done);
                }

                self.push(result);
            }
            opcode::ADD => {
                self.op_add(context)?;
            },
            opcode::SUBTRACT => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a - b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            opcode::MULTIPLY => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a * b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            opcode::DIVIDE => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a / b)),
                _ => return Err(VmError::UnexpectedValue),
            },
            opcode::POP => {
                self.pop();
            }
            opcode::DEFINE_GLOBAL => {
                let index = self.current_frame_mut().next_u32() as _;

                let identifier = current_import.symbol(index);
                let value = self.pop();
                current_import.set_global(identifier, value);
            }
            opcode::GET_GLOBAL => {
                let index = self.current_frame_mut().next_u32() as _;
                self.op_get_global(index)?;
            }
            opcode::SET_GLOBAL => {
                let index = self.current_frame_mut().next_u32() as _;
                self.op_set_global(index)?;
            }
            opcode::GET_LOCAL => {
                let index: usize = self.current_frame_mut().next_u32() as _;
                self.op_get_local(index);
            }
            opcode::SET_LOCAL => {
                let index: usize = self.current_frame_mut().next_u32() as _;

                self.op_set_local(index);
            }
            opcode::TRUE => self.push(Value::Boolean(true)),
            opcode::FALSE => self.push(Value::Boolean(false)),
            opcode::JUMP_IF_FALSE => {
                let to = self.current_frame_mut().next_u32() as _;
                if self.peek().is_falsey() {
                    self.current_frame_mut().set_pc(to);
                }
            },
            opcode::JUMP => {
                let to = self.current_frame_mut().next_u32() as _;
                self.current_frame_mut().set_pc(to);
            },
            opcode::LESS => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push((a < b).into()),
                _ => return Err(VmError::UnexpectedValue),
            },
            opcode::GREATER => match (self.pop(), self.pop()) {
                (Value::Number(b), Value::Number(a)) => self.push((a > b).into()),
                _ => return Err(VmError::UnexpectedValue),
            },
            opcode::EQUAL => {
                self.op_equal();
            }
            opcode::CALL => {
                let arity = self.current_frame_mut().next_u32() as _;

                let callee = *self.peek_n(arity);
                self.call(arity, callee, context)?;
            }
            opcode::NEGATE => match self.pop() {
                Value::Number(n) => self.push(Value::Number(-n)),
                _ => return Err(VmError::UnexpectedValue),
            },
            opcode::NOT => {
                let is_falsey = self.pop().is_falsey();
                self.push(is_falsey.into());
            }
            opcode::GET_UPVALUE => {
                let index: usize = self.current_frame_mut().next_u32() as _;
                self.op_get_upvalue(index);
            }
            opcode::SET_UPVALUE => {
                let index: usize = self.current_frame_mut().next_u32() as _;

                self.op_set_upvalue(index);
            }
            opcode::CLOSE_UPVALUE => {
                let index = self.stack.len() - 1;
                self.close_upvalues(index..index+1);
                self.pop();
            }
            opcode::INVOKE => {
                let index = self.current_frame_mut().next_u32() as _;
                let arity = self.current_frame_mut().next_u32() as _;

                self.op_invoke(index, arity, context)?;
            }
            _ => unreachable!(),
        }

        Ok(InterpretResult::More)
    }

    fn op_constant<W: Write>(&mut self, index: usize, context: &mut VmContext<W>) {
        use crate::bytecode::Constant;
        let current_import = self.current_import();
        match current_import.constant(index) {
            Constant::Number(n) => self.push(Value::Number(*n)),
            Constant::String(string) => self.push_string(string, context),
        }
    }

    fn op_add<W: Write>(&mut self, context: &mut VmContext<W>) -> Result<(), VmError> {
        match (self.pop(), self.pop()) {
            (Value::Number(b), Value::Number(a)) => self.push(Value::Number(a + b)),
            (Value::String(b), Value::String(a)) => self.push_string(format!("{}{}", a, b), context),
            _ => return Err(VmError::UnexpectedValue),
        }

        Ok(())
    }

    fn op_get_upvalue(&mut self, index: usize) {
        let upvalue = self.current_frame().closure.upvalues[index];
        self.push(self.resolve_upvalue_into_value(upvalue));
    }

    fn op_print<W: Write>(&mut self, context: &mut VmContext<W>) {
        match self.pop() {
            Value::Number(n) => writeln!(context.stdout, "{}", n).expect("Could not write to stdout"),
            Value::Nil => writeln!(context.stdout, "nil").expect("Could not write to stdout"),
            Value::Boolean(boolean) => writeln!(context.stdout, "{}", boolean).expect("Could not write to stdout"),
            Value::String(string) => writeln!(context.stdout, "{}", string).expect("Could not write to stdout"),
            Value::NativeFunction(_function) => writeln!(context.stdout, "<native fn>").expect("Could not write to stdout"),
            Value::Closure(closure) => writeln!(context.stdout, "<fn {}>", closure.function.name).expect("Could not write to stdout"),
            Value::Class(class) => writeln!(context.stdout, "{}", class.borrow().name).expect("Could not write to stdout"),
            Value::Instance(instance) => {
                writeln!(context.stdout, "{} instance", instance.borrow().class.borrow().name).expect("Could not write to stdout")
            },
            Value::BoundMethod(bind) => writeln!(context.stdout, "<fn {}>", bind.method.function.name).expect("Could not write to stdout"),
            Value::Import(_) => writeln!(context.stdout, "<import>").expect("Could not write to stdout"),
        }
    }

    fn op_set_upvalue(&mut self, index: usize) {
        let value = self.peek();
        let upvalue = self.current_frame().closure.upvalues[index];
        self.set_upvalue(upvalue, *value);
    }

    fn op_get_local(&mut self, index: usize) {
        let index = self.current_frame().base_counter + index;
        self.push(*self.stack.get(index));
    }

    fn op_set_local(&mut self, index: usize) {
        let index = self.current_frame().base_counter + index;
        let value = self.peek();
        self.stack.set(index, *value);
    }

    fn op_equal(&mut self) {
        let b = self.pop();
        let a = self.pop();

        if Value::is_same_type(&a, &b) {
            match (b, a) {
                (Value::Number(b), Value::Number(a)) => self.push((a == b).into()),
                (Value::Boolean(b), Value::Boolean(a)) => self.push((a == b).into()),
                (Value::String(b), Value::String(a)) => self.push((*a == *b).into()),
                (Value::Closure(b), Value::Closure(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::NativeFunction(b), Value::NativeFunction(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Nil, Value::Nil) => self.push(true.into()),
                (Value::BoundMethod(b), Value::BoundMethod(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Class(b), Value::Class(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Instance(b), Value::Instance(a)) => self.push((Gc::ptr_eq(&a, &b)).into()),
                _ => unimplemented!(),
            };
        } else {
            self.push(false.into())
        }
    }

    fn op_get_global(&mut self, index: usize) -> Result<(), VmError> {
        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = current_import.global(identifier);
        if let Some(value) = value {
            self.push(value);
        } else {
            return Err(VmError::GlobalNotDefined);
        }

        Ok(())
    }

    fn op_set_global(&mut self, index: usize) -> Result<(), VmError> {
        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = self.peek();
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
        if let Value::Instance(instance) = self.peek_n(1) {
            instance
                .borrow_mut()
                .fields
                .insert(property, *self.peek());

            let value = self.pop();
            self.pop();
            self.push(value);
        } else {
            return Err(VmError::UnexpectedValue);
        }

        Ok(())
    }

    fn op_get_property<W: Write>(&mut self, index: usize, context: &mut VmContext<W>) -> Result<(), VmError> {
        let current_import = self.current_import();
        let property = current_import.symbol(index);
        if let Value::Instance(instance) = self.pop() {
            if let Some(value) = instance.borrow().fields.get(&property) {
                self.push(*value);
            } else if let Some(method) = instance.borrow().class.borrow().methods.get(&property) {
                let bind = context.manage(BoundMethod {
                    receiver: instance,
                    method: *method,
                });
                self.push(Value::BoundMethod(bind.as_gc()));
            } else {
                return Err(VmError::UndefinedProperty);
            };
        } else {
            return Err(VmError::UnexpectedValue);
        }
        Ok(())
    }

    fn op_closure<W: Write>(&mut self, index: usize, context: &mut VmContext<W>) {
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
                            self.upvalues.push(root.as_gc());
                            root.as_gc()
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
        self.push(Value::Closure(closure_root.as_gc()));
    }

    fn op_invoke<W: Write>(&mut self, index: usize, arity: usize, context: &mut VmContext<W>) -> Result<(), VmError> {
        let current_import = self.current_import();
        let property = current_import.symbol(index);
        if let Value::Instance(instance) = *self.peek_n(arity) {
            if let Some(value) = instance.borrow().fields.get(&property) {
                self.rset(arity, *value);
                self.call(arity, *value, context)?;
            } else if let Some(method) = instance.borrow().class.borrow().methods.get(&property) {
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
    fn call<W: Write>(&mut self, arity: usize, callee: Value, outer: &mut VmContext<W>) -> Result<(), VmError> {
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
                self.pop(); // discard callee
                let result = (callee.code)(&args);
                self.push(result);
            }
            Value::Class(class) => {
                let instance = outer.manage(RefCell::new(Instance {
                    class,
                    fields: FxHashMap::default(),
                }));
                self.rset(arity, Value::Instance(instance.as_gc()));

                let init_symbol = outer.intern("init"); //TODO move to constructor
                if let Some(initializer) = class.borrow().methods.get(&init_symbol) {
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
                self.rset(arity, Value::Instance(bind.receiver));
                self.begin_frame(callee);
            },
            _ => return Err(VmError::InvalidCallee),
        }

        Ok(())
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("No frame")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("No frame")
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn rset(&mut self, index: usize, value: Value) {
        self.stack.rset(index, value);
    }

    fn push_string<W: Write>(&mut self, string: impl Into<String>, outer: &mut VmContext<W>) {
        let root = outer.manage(string.into());
        self.push(Value::String(root.as_gc()));
    }

    fn pop(&mut self) -> Value {
        self.stack.pop()
    }

    //TODO investigate
    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        let mut result = Vec::with_capacity(n);
        while result.len() < n {
            result.push(self.pop());
        }

        result
    }

    fn peek(&self) -> &Value {
        self.stack.peek_n(0)
    }

    fn peek_n(&self, n: usize) -> &Value {
        self.stack.peek_n(n)
    }

    fn begin_frame(&mut self, closure: Gc<Closure>) {
        self.frames.push(CallFrame::new(closure, self.stack.len() - closure.function.arity - 1));
    }
}

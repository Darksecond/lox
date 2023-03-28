use crate::runtime::{Runtime, Signal, VmError};
use crate::memory::*;
use super::gc::Gc;
use std::cell::Cell;
use super::fiber::Fiber;
use std::cell::UnsafeCell;

impl Runtime {
    pub fn interpret(&mut self) -> Result<(), VmError> {
        use lox_bytecode::opcode;

        loop {
            let result = match self.next_u8() {
                opcode::CONSTANT      => self.op_constant(),
                opcode::IMPORT        => self.op_import(),
                opcode::IMPORT_GLOBAL => self.op_import_global(),
                opcode::CLOSURE       => self.op_closure(),
                opcode::CLASS         => self.op_class(),
                opcode::METHOD        => self.op_method(),
                opcode::SET_PROPERTY  => self.op_set_property(),
                opcode::GET_PROPERTY  => self.op_get_property(),
                opcode::PRINT         => self.op_print(),
                opcode::NIL           => self.op_nil(),
                opcode::RETURN        => self.op_return(),
                opcode::ADD           => self.op_add(),
                opcode::SUBTRACT      => self.op_subtract(),
                opcode::MULTIPLY      => self.op_multiply(),
                opcode::DIVIDE        => self.op_divide(),
                opcode::POP           => self.op_pop(),
                opcode::DEFINE_GLOBAL => self.op_define_global(),
                opcode::GET_GLOBAL    => self.op_get_global(),
                opcode::SET_GLOBAL    => self.op_set_global(),
                opcode::GET_LOCAL     => self.op_get_local(),
                opcode::SET_LOCAL     => self.op_set_local(),
                opcode::TRUE          => self.op_bool(true),
                opcode::FALSE         => self.op_bool(false),
                opcode::JUMP_IF_FALSE => self.op_jump_if_false(),
                opcode::JUMP          => self.op_jump(),
                opcode::LESS          => self.op_less(),
                opcode::GREATER       => self.op_greater(),
                opcode::EQUAL         => self.op_equal(),
                opcode::CALL          => self.op_call(),
                opcode::NEGATE        => self.op_negate(),
                opcode::NOT           => self.op_not(),
                opcode::GET_UPVALUE   => self.op_get_upvalue(),
                opcode::SET_UPVALUE   => self.op_set_upvalue(),
                opcode::CLOSE_UPVALUE => self.op_close_upvalue(),
                opcode::INVOKE        => self.op_invoke(),
                _ => unreachable!(),
            };

            match result {
                Signal::Done => return Ok(()),
                Signal::More => (),
                Signal::RuntimeError => {
                    return Err(self.fiber.as_ref().error.unwrap_or(VmError::Unknown));
                },
                Signal::ContextSwitch => {
                    self.context_switch();
                },
            }
        }
    }

    #[inline(never)]
    pub fn op_import(&mut self) -> Signal {
        let index: usize = self.next_u32() as _;

        let current_import = self.current_import();
        let constant = current_import.constant(index);
        match constant {
            lox_bytecode::bytecode::Constant::String(path) => {
                if let Some(import) = self.import(path) {
                    self.fiber.as_mut().stack.push(Value::Import(import));
                    return Signal::More;
                }

                let import = match self.load_import(path) {
                    Ok(import) => import,
                    Err(err) => return self.fiber.as_mut().runtime_error(err),
                };

                self.fiber.as_mut().stack.push(Value::Import(import));

                let mut fiber = Fiber::new(Some(self.fiber));

                let function = Function {
                    arity: 0,
                    chunk_index: 0,
                    name: "top".into(),
                    import,
                };

                let closure = self.manage(Closure {
                    upvalues: vec![],
                    function,
                });

                fiber.stack.push(Value::Closure(closure));
                fiber.begin_frame(closure);

                let fiber = self.manage(UnsafeCell::new(fiber));

                return self.switch_to(fiber);
            },
            lox_bytecode::bytecode::Constant::Number(_) => {
                return self.fiber.as_mut().runtime_error(VmError::StringConstantExpected);
            },
        }
    }

    #[inline(never)]
    pub fn op_import_global(&mut self) -> Signal {
        let index: usize = self.next_u32() as _;
        let current_import = self.current_import();
        let identifier = current_import.symbol(index);

        let import = self.fiber.as_ref().stack.peek_n(0);
        if let Value::Import(import) = import {
            let value = import.global(identifier).unwrap_or(Value::Nil);
            self.fiber.as_mut().stack.push(value);
        } else {
            return self.fiber.as_mut().runtime_error(VmError::UnexpectedConstant)
        }

        Signal::More
    }

    pub fn op_jump(&mut self) -> Signal {
        let to = self.next_i16();

        self.set_ip(to);

        Signal::More
    }

    pub fn op_jump_if_false(&mut self) -> Signal {
        let to = self.next_i16();

        if self.fiber.as_ref().stack.peek_n(0).is_falsey() {
            self.set_ip(to);
        }

        Signal::More
    }

    #[inline(never)]
    pub fn op_class(&mut self) -> Signal {
        let index = self.next_u8() as _;

        let current_import = self.current_import();
        let class = current_import.class(index);
        let class = self.manage(Class::new(class.name.clone()));
        self.fiber.as_mut().stack.push(Value::Class(class));

        Signal::More
    }

    pub fn op_bool(&mut self, value: bool) -> Signal {
        self.fiber.as_mut().stack.push(Value::Boolean(value));

        Signal::More
    }

    pub fn op_nil(&mut self) -> Signal {
        self.fiber.as_mut().stack.push(Value::Nil);

        Signal::More
    }

    //TODO Rewrite if's to improve error handling
    //TODO Pretty sure it leaves the stack clean, but double check
    #[inline(never)]
    pub fn op_method(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let identifier = current_import.symbol(index);

        let class = match self.fiber.as_ref().stack.peek_n(1) {
            Value::Class(class) => class,
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedConstant),
        };

        let closure = match self.fiber.as_ref().stack.peek_n(0) {
            Value::Closure(closure) => closure,
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedConstant),
        };

        class.set_method(identifier, Value::Closure(closure));

        self.fiber.as_mut().stack.pop();

        Signal::More
    }

    pub fn op_return(&mut self) -> Signal {
        let result = self.fiber.as_mut().stack.pop();

        let base_counter = self.fiber.as_ref().current_frame().base_counter;
        self.fiber.as_mut().close_upvalues(base_counter);
        self.fiber.as_mut().stack.truncate(base_counter);

        if let Some(error) = self.fiber.as_mut().end_frame() {
            return error;
        }

        if !self.fiber.as_ref().has_current_frame() {
            // If we have a parent, switch back to it.
            if let Some(parent) = self.fiber.as_ref().parent {
                return self.switch_to(parent);
            }

            // We are done interpreting, don't push a result as it'll be nil
            Signal::Done
        } else {
            self.load_ip();
            self.fiber.as_mut().stack.push(result);
            Signal::More
        }
    }

    pub fn op_pop(&mut self) -> Signal {
        self.fiber.as_mut().stack.pop();
        Signal::More
    }

    pub fn op_call(&mut self) -> Signal {
        let arity = self.next_u8() as _;

        let callee = self.fiber.as_ref().stack.peek_n(arity);

        self.call(arity, callee)
    }

    pub fn op_constant(&mut self) -> Signal {
        let index = self.next_u32() as _;
        use lox_bytecode::bytecode::Constant;
        let current_import = self.current_import();
        match current_import.constant(index) {
            Constant::Number(n) => self.fiber.as_mut().stack.push(Value::Number(*n)),
            Constant::String(string) => self.push_string(string),
        }

        Signal::More
    }

    pub fn op_greater(&mut self) -> Signal {
        match (self.fiber.as_mut().stack.pop(), self.fiber.as_mut().stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.as_mut().stack.push((a > b).into()),
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_less(&mut self) -> Signal {
        match (self.fiber.as_mut().stack.pop(), self.fiber.as_mut().stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.as_mut().stack.push((a < b).into()),
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_negate(&mut self) -> Signal {
        match self.fiber.as_mut().stack.pop() {
            Value::Number(n) => self.fiber.as_mut().stack.push(Value::Number(-n)),
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_not(&mut self) -> Signal {
        let is_falsey = self.fiber.as_mut().stack.pop().is_falsey();
        self.fiber.as_mut().stack.push(is_falsey.into());
        Signal::More
    }

    pub fn op_divide(&mut self) -> Signal {
        match (self.fiber.as_mut().stack.pop(), self.fiber.as_mut().stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.as_mut().stack.push(Value::Number(a / b)),
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_multiply(&mut self) -> Signal {
        match (self.fiber.as_mut().stack.pop(), self.fiber.as_mut().stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.as_mut().stack.push(Value::Number(a * b)),
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_subtract(&mut self) -> Signal {
        match (self.fiber.as_mut().stack.pop(), self.fiber.as_mut().stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.as_mut().stack.push(Value::Number(a - b)),
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_add(&mut self) -> Signal {
        match (self.fiber.as_mut().stack.pop(), self.fiber.as_mut().stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.as_mut().stack.push(Value::Number(a + b)),
            (Value::String(b), Value::String(a)) => self.push_string(format!("{}{}", a, b)),
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_get_upvalue(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let upvalue = self.fiber.as_ref().current_frame().closure.upvalues[index];
        let value = self.fiber.as_ref().resolve_upvalue_into_value(upvalue);
        self.fiber.as_mut().stack.push(value);

        Signal::More
    }

    pub fn op_set_upvalue(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let value = self.fiber.as_ref().stack.peek_n(0);
        let upvalue = self.fiber.as_ref().current_frame().closure.upvalues[index];
        self.fiber.as_mut().set_upvalue(upvalue, value);
        Signal::More
    }

    pub fn op_close_upvalue(&mut self) -> Signal {
        let index = self.fiber.as_ref().stack.len() - 1;
        self.fiber.as_mut().close_upvalues(index);
        self.fiber.as_mut().stack.pop();
        Signal::More
    }

    //TODO consider redesigning
    #[inline(never)]
    pub fn op_print(&mut self) -> Signal {
        let value = self.fiber.as_mut().stack.pop();
        self.print(&format!("{}", value));
        Signal::More
    }

    pub fn op_get_local(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let index = self.fiber.as_ref().current_frame().base_counter + index;
        let value = self.fiber.as_ref().stack.get(index);
        self.fiber.as_mut().stack.push(value);
        Signal::More
    }

    pub fn op_set_local(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let index = self.fiber.as_ref().current_frame().base_counter + index;
        let value = self.fiber.as_ref().stack.peek_n(0);
        self.fiber.as_mut().stack.set(index, value);

        Signal::More
    }

    pub fn op_equal(&mut self) -> Signal {
        let b = self.fiber.as_mut().stack.pop();
        let a = self.fiber.as_mut().stack.pop();

        if Value::is_same_type(&a, &b) {
            match (b, a) {
                (Value::Number(b), Value::Number(a)) => self.fiber.as_mut().stack.push((a == b).into()),
                (Value::Boolean(b), Value::Boolean(a)) => self.fiber.as_mut().stack.push((a == b).into()),
                (Value::String(b), Value::String(a)) => self.fiber.as_mut().stack.push((*a == *b).into()),
                (Value::Closure(b), Value::Closure(a)) => self.fiber.as_mut().stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::NativeFunction(b), Value::NativeFunction(a)) => self.fiber.as_mut().stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Nil, Value::Nil) => self.fiber.as_mut().stack.push(true.into()),
                (Value::BoundMethod(b), Value::BoundMethod(a)) => self.fiber.as_mut().stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Class(b), Value::Class(a)) => self.fiber.as_mut().stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Instance(b), Value::Instance(a)) => self.fiber.as_mut().stack.push((Gc::ptr_eq(&a, &b)).into()),
                _ => unimplemented!(),
            };
        } else {
            self.fiber.as_mut().stack.push(false.into())
        }

        Signal::More
    }

    #[inline(never)]
    pub fn op_define_global(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = self.fiber.as_mut().stack.pop();
        current_import.set_global(identifier, value);

        Signal::More
    }

    pub fn op_get_global(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = current_import.global(identifier);
        if let Some(value) = value {
            self.fiber.as_mut().stack.push(value);
        } else {
            return self.fiber.as_mut().runtime_error(VmError::GlobalNotDefined);
        }

        Signal::More
    }

    pub fn op_set_global(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = self.fiber.as_mut().stack.peek_n(0);
        if current_import.has_global(identifier) {
            current_import.set_global(identifier, value);
        } else {
            return self.fiber.as_mut().runtime_error(VmError::GlobalNotDefined);
        }

        Signal::More
    }

    pub fn op_set_property(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let property = current_import.symbol(index);
        if let Value::Instance(instance) = self.fiber.as_ref().stack.peek_n(1) {
            instance.set_field(property, self.fiber.as_ref().stack.peek_n(0));
            self.adjust_size(instance);

            let value = self.fiber.as_mut().stack.pop();
            self.fiber.as_mut().stack.pop();
            self.fiber.as_mut().stack.push(value);
        } else {
            return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue);
        }

        Signal::More
    }

    pub fn op_get_property(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let property = current_import.symbol(index);

        let instance = match self.fiber.as_mut().stack.pop() {
            Value::Instance(instance) => instance,
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue),
        };

        if let Some(value) = instance.field(property) {
            self.fiber.as_mut().stack.push(value);
            return Signal::More;
        }

        let method = match instance.class.method(property) {
            Some(method) => method,
            None => return self.fiber.as_mut().runtime_error(VmError::UndefinedProperty),
        };

        let bind = self.manage(BoundMethod {
            receiver: instance,
            method,
        });

        self.fiber.as_mut().stack.push(Value::BoundMethod(bind));

        return Signal::More;
    }

    pub fn op_closure(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let closure = current_import.closure(index);
        let base = self.fiber.as_ref().current_frame().base_counter;
        let upvalues = closure
            .upvalues
            .iter()
            .map(|u| {
                match u {
                    lox_bytecode::bytecode::Upvalue::Local(index) => {
                        let index = base + *index;

                        if let Some(upvalue) = self.fiber.as_ref().find_open_upvalue_with_index(index)
                        {
                            upvalue
                        } else {
                            let root = self.manage(Cell::new(Upvalue::Open(index)));
                            self.fiber.as_mut().push_upvalue(root);
                            root
                        }
                    }
                    lox_bytecode::bytecode::Upvalue::Upvalue(u) => {
                        self.fiber.as_ref().find_upvalue_by_index(*u)
                    }
                }
            })
        .collect();

        let closure_root = self.manage(Closure {
            function: Function::new(&closure.function, current_import),
            upvalues,
        });
        self.fiber.as_mut().stack.push(Value::Closure(closure_root));

        Signal::More
    }

    pub fn op_invoke(&mut self) -> Signal {
        let arity = self.next_u8() as _;
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let property = current_import.symbol(index);

        let instance = match self.fiber.as_mut().stack.peek_n(arity) {
            Value::Instance(instance) => instance,
            _ => return self.fiber.as_mut().runtime_error(VmError::UnexpectedValue),
        };

        if let Some(value) = instance.field(property) {
            self.fiber.as_mut().stack.rset(arity, value);
            return self.call(arity, value);
        }

        let method = match instance.class.method(property) {
            Some(value) => value,
            None => return self.fiber.as_mut().runtime_error(VmError::UndefinedProperty),
        };

        match method {
            Value::Closure(method) => {
                if method.function.arity != arity {
                    return self.fiber.as_mut().runtime_error(VmError::IncorrectArity);
                }

                self.store_ip();
                self.fiber.as_mut().begin_frame(method);
                self.load_ip();

                return Signal::More;
            }
            _ => return self.call(arity, method),
        }
    }
}

use crate::bettervm::vm::{Runtime, Signal, VmError};
use crate::bettervm::memory::*;
use crate::bettergc::Gc;
use std::cell::Cell;

impl Runtime {
    pub fn op_import(&mut self) -> Signal {
        let _index: usize = self.next_u32() as _;
        self.fiber.runtime_error(VmError::Unimplemented)
    }

    pub fn op_import_global(&mut self) -> Signal {
        let _index: usize = self.next_u32() as _;
        self.fiber.runtime_error(VmError::Unimplemented)
    }

    pub fn op_jump(&mut self) -> Signal {
        let to = self.next_u32() as _;

        self.set_ip(to);

        Signal::More
    }

    pub fn op_jump_if_false(&mut self) -> Signal {
        let to = self.next_u32() as _;

        if self.fiber.stack.peek_n(0).is_falsey() {
            self.set_ip(to);
        }

        Signal::More
    }

    pub fn op_class(&mut self) -> Signal {
        let index = self.next_u8() as _;

        let current_import = self.current_import();
        let class = current_import.class(index);
        let class = self.manage(Class::new(class.name.clone()));
        self.fiber.stack.push(Value::Class(class));

        Signal::More
    }

    pub fn op_bool(&mut self, value: bool) -> Signal {
        self.fiber.stack.push(Value::Boolean(value));

        Signal::More
    }

    pub fn op_nil(&mut self) -> Signal {
        self.fiber.stack.push(Value::Nil);

        Signal::More
    }

    //TODO Rewrite if's to improve error handling
    //TODO Pretty sure it leaves the stack clean, but double check
    pub fn op_method(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        if let Value::Class(class) = self.fiber.stack.peek_n(1) {
            if let Value::Closure(closure) = self.fiber.stack.peek_n(0) {
                class.set_method(identifier, *closure);
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedConstant);
            }
        } else {
            return self.fiber.runtime_error(VmError::UnexpectedConstant);
        }

        self.fiber.stack.pop();

        Signal::More
    }

    pub fn op_return(&mut self) -> Signal {
        let result = self.fiber.stack.pop();

        let base_counter = self.fiber.current_frame().base_counter;
        self.fiber.close_upvalues(base_counter);
        self.fiber.stack.truncate(base_counter);

        if let Some(error) = self.fiber.end_frame() {
            return error;
        }

        if !self.fiber.has_current_frame() {
            // We are done interpreting, don't push a result as it'll be nil
            Signal::Done
        } else {
            self.load_ip();
            self.fiber.stack.push(result);
            Signal::More
        }
    }

    pub fn op_pop(&mut self) -> Signal {
        self.fiber.stack.pop();
        Signal::More
    }

    pub fn op_call(&mut self) -> Signal {
        let arity = self.next_u8() as _;

        let callee = *self.fiber.stack.peek_n(arity);

        self.call(arity, callee)
    }

    pub fn op_constant(&mut self) -> Signal {
        let index = self.next_u32() as _;
        use crate::bytecode::Constant;
        let current_import = self.current_import();
        match current_import.constant(index) {
            Constant::Number(n) => self.fiber.stack.push(Value::Number(*n)),
            Constant::String(string) => self.push_string(string),
        }

        Signal::More
    }

    pub fn op_greater(&mut self) -> Signal {
        match (self.fiber.stack.pop(), self.fiber.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.stack.push((a > b).into()),
            _ => return self.fiber.runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_less(&mut self) -> Signal {
        match (self.fiber.stack.pop(), self.fiber.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.stack.push((a < b).into()),
            _ => return self.fiber.runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_negate(&mut self) -> Signal {
        match self.fiber.stack.pop() {
            Value::Number(n) => self.fiber.stack.push(Value::Number(-n)),
            _ => return self.fiber.runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_not(&mut self) -> Signal {
        let is_falsey = self.fiber.stack.pop().is_falsey();
        self.fiber.stack.push(is_falsey.into());
        Signal::More
    }

    pub fn op_divide(&mut self) -> Signal {
        match (self.fiber.stack.pop(), self.fiber.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.stack.push(Value::Number(a / b)),
            _ => return self.fiber.runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_multiply(&mut self) -> Signal {
        match (self.fiber.stack.pop(), self.fiber.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.stack.push(Value::Number(a * b)),
            _ => return self.fiber.runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_subtract(&mut self) -> Signal {
        match (self.fiber.stack.pop(), self.fiber.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.stack.push(Value::Number(a - b)),
            _ => return self.fiber.runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_add(&mut self) -> Signal {
        match (self.fiber.stack.pop(), self.fiber.stack.pop()) {
            (Value::Number(b), Value::Number(a)) => self.fiber.stack.push(Value::Number(a + b)),
            (Value::String(b), Value::String(a)) => self.push_string(format!("{}{}", a, b)),
            _ => return self.fiber.runtime_error(VmError::UnexpectedValue),
        }

        Signal::More
    }

    pub fn op_get_upvalue(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let upvalue = self.fiber.current_frame().closure.upvalues[index];
        self.fiber.stack.push(self.fiber.resolve_upvalue_into_value(upvalue));

        Signal::More
    }

    pub fn op_set_upvalue(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let value = self.fiber.stack.peek_n(0);
        let upvalue = self.fiber.current_frame().closure.upvalues[index];
        self.fiber.set_upvalue(upvalue, *value);
        Signal::More
    }

    pub fn op_close_upvalue(&mut self) -> Signal {
        let index = self.fiber.stack.len() - 1;
        self.fiber.close_upvalues(index);
        self.fiber.stack.pop();
        Signal::More
    }

    //TODO consider redesigning
    pub fn op_print(&mut self) -> Signal {
        let value = self.fiber.stack.pop();
        self.print_fn(&format!("{}", value));
        Signal::More
    }

    pub fn op_get_local(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let index = self.fiber.current_frame().base_counter + index;
        self.fiber.stack.push(*self.fiber.stack.get(index));
        Signal::More
    }

    pub fn op_set_local(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let index = self.fiber.current_frame().base_counter + index;
        let value = self.fiber.stack.peek_n(0);
        self.fiber.stack.set(index, *value);

        Signal::More
    }

    pub fn op_equal(&mut self) -> Signal {
        let b = self.fiber.stack.pop();
        let a = self.fiber.stack.pop();

        if Value::is_same_type(&a, &b) {
            match (b, a) {
                (Value::Number(b), Value::Number(a)) => self.fiber.stack.push((a == b).into()),
                (Value::Boolean(b), Value::Boolean(a)) => self.fiber.stack.push((a == b).into()),
                (Value::String(b), Value::String(a)) => self.fiber.stack.push((*a == *b).into()),
                (Value::Closure(b), Value::Closure(a)) => self.fiber.stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::NativeFunction(b), Value::NativeFunction(a)) => self.fiber.stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Nil, Value::Nil) => self.fiber.stack.push(true.into()),
                (Value::BoundMethod(b), Value::BoundMethod(a)) => self.fiber.stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Class(b), Value::Class(a)) => self.fiber.stack.push((Gc::ptr_eq(&a, &b)).into()),
                (Value::Instance(b), Value::Instance(a)) => self.fiber.stack.push((Gc::ptr_eq(&a, &b)).into()),
                _ => unimplemented!(),
            };
        } else {
            self.fiber.stack.push(false.into())
        }

        Signal::More
    }

    pub fn op_define_global(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = self.fiber.stack.pop();
        current_import.set_global(identifier, value);

        Signal::More
    }

    pub fn op_get_global(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = current_import.global(identifier);
        if let Some(value) = value {
            self.fiber.stack.push(value);
        } else {
            return self.fiber.runtime_error(VmError::GlobalNotDefined);
        }

        Signal::More
    }

    pub fn op_set_global(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let identifier = current_import.symbol(index);
        let value = self.fiber.stack.peek_n(0);
        if current_import.has_global(identifier) {
            current_import.set_global(identifier, *value);
        } else {
            return self.fiber.runtime_error(VmError::GlobalNotDefined);
        }

        Signal::More
    }

    pub fn op_set_property(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let property = current_import.symbol(index);
        if let Value::Instance(instance) = self.fiber.stack.peek_n(1) {
            instance.set_field(property, *self.fiber.stack.peek_n(0));

            let value = self.fiber.stack.pop();
            self.fiber.stack.pop();
            self.fiber.stack.push(value);
        } else {
            return self.fiber.runtime_error(VmError::UnexpectedValue);
        }

        Signal::More
    }

    pub fn op_get_property(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let property = current_import.symbol(index);
        if let Value::Instance(instance) = self.fiber.stack.pop() {
            if let Some(value) = instance.field(&property) {
                self.fiber.stack.push(value);
            } else if let Some(method) = instance.class.method(property) {
                let bind = self.manage(BoundMethod {
                    receiver: instance,
                    method,
                });
                self.fiber.stack.push(Value::BoundMethod(bind));
            } else {
                return self.fiber.runtime_error(VmError::UndefinedProperty);
            };
        } else {
            return self.fiber.runtime_error(VmError::UnexpectedValue);
        }

        Signal::More
    }

    pub fn op_closure(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let closure = current_import.closure(index);
        let base = self.fiber.current_frame().base_counter;
        let upvalues = closure
            .upvalues
            .iter()
            .map(|u| {
                match u {
                    crate::bytecode::Upvalue::Local(index) => {
                        let index = base + *index;

                        if let Some(upvalue) = self.fiber.find_open_upvalue_with_index(index)
                        {
                            upvalue
                        } else {
                            let root = self.manage(Cell::new(Upvalue::Open(index)));
                            self.fiber.push_upvalue(root);
                            root
                        }
                    }
                    crate::bytecode::Upvalue::Upvalue(u) => {
                        self.fiber.find_upvalue_by_index(*u)
                    }
                }
            })
        .collect();

        let closure_root = self.manage(Closure {
            function: Function::new(&closure.function, current_import),
            upvalues,
        });
        self.fiber.stack.push(Value::Closure(closure_root));

        Signal::More
    }

    pub fn op_invoke(&mut self) -> Signal {
        let arity = self.next_u8() as _;
        let index = self.next_u32() as _;

        let current_import = self.current_import();
        let property = current_import.symbol(index);
        if let Value::Instance(instance) = *self.fiber.stack.peek_n(arity) {
            if let Some(value) = instance.field(&property) {
                self.fiber.stack.rset(arity, value);
                return self.call(arity, value);
            } else if let Some(method) = instance.class.method(property) {
                if method.function.arity != arity {
                    return self.fiber.runtime_error(VmError::IncorrectArity);
                }
                self.store_ip();
                self.fiber.begin_frame(method);
                self.load_ip();
            } else {
                return self.fiber.runtime_error(VmError::UndefinedProperty);
            };
        } else {
            return self.fiber.runtime_error(VmError::UnexpectedValue);
        }

        Signal::More
    }
}

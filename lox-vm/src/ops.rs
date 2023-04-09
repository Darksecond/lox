use crate::runtime::{Runtime, Signal, VmError};
use crate::memory::*;
use super::gc::Gc;
use super::fiber::Fiber;
use crate::value::Value;

macro_rules! as_obj {
    ($self:ident, $value:expr, $tag:ident) => {
        {
            let value = $value;
            if value.is_object_of_type::<$tag>() {
                value.as_object().cast::<$tag>()
            } else {
                return $self.fiber.runtime_error(VmError::UnexpectedValue);
            }
        }
    };
}

impl Runtime {
    pub fn interpret(&mut self) -> Result<(), VmError> {
        use lox_bytecode::opcode;

        loop {
            let opcode = self.next_u8();
            //println!("opcode: {}", opcode);
            let result = match opcode {
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
                opcode::LIST          => self.op_list(),
                opcode::GET_INDEX     => self.op_get_index(),
                opcode::SET_INDEX     => self.op_set_index(),
                opcode::NUMBER        => self.op_number(),
                opcode::STRING        => self.op_string(),
                _ => unreachable!(),
            };

            match result {
                Signal::Done => return Ok(()),
                Signal::More => (),
                Signal::RuntimeError => {
                    return Err(self.fiber.error().unwrap_or(VmError::Unknown));
                },
                Signal::ContextSwitch => {
                    self.context_switch();
                },
            }
        }
    }

    pub fn op_set_index(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let value = stack.pop();
            let index = stack.pop();
            let list = stack.pop();

            let list = as_obj!(self, list, List);

            let index = if index.is_number() {
                index.as_number() as usize
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            };

            if !list.is_valid(index) {
                return self.fiber.runtime_error(VmError::IndexOutOfRange);
            }

            list.set(index, value);

            stack.push(value);

            Signal::More
        })
    }

    pub fn op_get_index(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let index = stack.pop();
            let list = stack.pop();

            let list = as_obj!(self, list, List);

            let index = if index.is_number() {
                index.as_number() as usize
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            };

            if !list.is_valid(index) {
                return self.fiber.runtime_error(VmError::IndexOutOfRange);
            }

            let value = list.get(index);

            stack.push(value);

            Signal::More
        })
    }

    #[cold]
    pub fn op_list(&mut self) -> Signal {
        let arity = self.next_u8();

        let list = self.fiber.with_stack(|stack| {
            List::with_stack(arity as _, stack)
        });

        let list: Gc<Object<List>> = self.manage(list.into());

        self.fiber.with_stack(|stack| {
            stack.push(Value::from_object(list))
        });

        Signal::More
    }

    #[cold]
    pub fn op_import(&mut self) -> Signal {
        let index: usize = self.next_u32() as _;

        let current_import = self.fiber.current_import();
        let path = current_import.string(index);

        if let Some(import) = self.import(path.as_str()) {
            self.fiber.with_stack(|stack| {
                stack.push(Value::from_object(import))
            });

            return Signal::More;
        }

        let import = match self.load_import(path.as_str()) {
            Ok(import) => import,
            Err(err) => return self.fiber.runtime_error(err),
        };

        self.fiber.with_stack(|stack| {
            stack.push(Value::from_object(import))
        });

        let fiber = Fiber::new(Some(self.fiber));

        let closure = self.manage(Closure::with_import(import).into());

        fiber.with_stack(|stack| {
            stack.push(Value::from_object(closure))
        });

        fiber.begin_frame(closure);

        let fiber = self.manage(fiber);

        return self.switch_to(Some(fiber));
    }

    #[cold]
    pub fn op_import_global(&mut self) -> Signal {
        let index: usize = self.next_u32() as _;
        let current_import = self.fiber.current_import();
        let identifier = current_import.symbol(index);

        let import = self.fiber.with_stack(|stack| {
            stack.peek_n(0)
        });

        let import = as_obj!(self, import, Import);

        let value = import.global(identifier).unwrap_or(Value::NIL);
        self.fiber.with_stack(|stack| stack.push(value));

        Signal::More
    }

    pub fn op_jump(&mut self) -> Signal {
        let to = self.next_i16();

        self.set_ip(to);

        Signal::More
    }

    pub fn op_jump_if_false(&mut self) -> Signal {
        let to = self.next_i16();

        let value = self.fiber.with_stack(|stack| {
            stack.peek_n(0)
        });

        if value.is_falsey() {
            self.set_ip(to);
        }

        Signal::More
    }

    #[cold]
    pub fn op_class(&mut self) -> Signal {
        let index = self.next_u8() as _;

        let current_import = self.fiber.current_import();
        let class = current_import.class(index);
        let class: Gc<Object<Class>> = self.manage(Class::new(class.name.clone()).into());
        self.fiber.with_stack(|stack| {
            stack.push(Value::from_object(class));
        });

        Signal::More
    }

    pub fn op_bool(&mut self, value: bool) -> Signal {
        self.fiber.with_stack(|stack| {
            stack.push(value.into())
        });

        Signal::More
    }

    pub fn op_nil(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            stack.push(Value::NIL)
        });

        Signal::More
    }

    //TODO Rewrite if's to improve error handling
    //TODO Pretty sure it leaves the stack clean, but double check
    #[cold]
    pub fn op_method(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.fiber.current_import();
        let identifier = current_import.symbol(index);

        let (class, closure) = self.fiber.with_stack(|stack| {
            (stack.peek_n(1), stack.peek_n(0))
        });

        let class = as_obj!(self, class, Class);
        let closure = as_obj!(self, closure, Closure);

        class.set_method(identifier, Value::from_object(closure));

        self.fiber.with_stack(|stack| {
            stack.pop()
        });

        Signal::More
    }

    pub fn op_return(&mut self) -> Signal {
        let result = self.fiber.with_stack(|stack| stack.pop());

        let base_counter = self.fiber.current_frame().base_counter;
        self.fiber.close_upvalues(base_counter);
        self.fiber.with_stack(|stack| stack.truncate(base_counter));

        if let Some(error) = self.fiber.end_frame() {
            return error;
        }

        if self.fiber.has_current_frame() {
            self.load_ip();
            self.fiber.with_stack(|stack| stack.push(result));
            Signal::More
        } else {
            self.switch_to(self.fiber.parent)
        }
    }

    pub fn op_pop(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            stack.pop()
        });

        Signal::More
    }

    pub fn op_call(&mut self) -> Signal {
        let arity = self.next_u8() as _;

        let callee = self.fiber.with_stack(|stack| {
            stack.peek_n(arity)
        });

        self.call(arity, callee)
    }

    pub fn op_number(&mut self) -> Signal {
        let index = self.next_u16();
        let value = self.fiber.current_import().number(index as _);

        self.fiber.with_stack(|stack| {
            stack.push(value.into())
        });

        Signal::More
    }

    pub fn op_string(&mut self) -> Signal {
        let index = self.next_u16();
        let value = self.fiber.current_import().string(index as _);

        self.fiber.with_stack(|stack| {
            stack.push(Value::from_object(value))
        });

        Signal::More
    }

    pub fn op_greater(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let b = stack.pop();
            let a = stack.pop();
            if a.is_number() && b.is_number() {
                let a = a.as_number();
                let b = b.as_number();
                stack.push((a > b).into());
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            }

            Signal::More
        })
    }

    pub fn op_less(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let b = stack.pop();
            let a = stack.pop();
            if a.is_number() && b.is_number() {
                let a = a.as_number();
                let b = b.as_number();
                stack.push((a < b).into());
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            }

            Signal::More
        })
    }

    pub fn op_negate(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let a = stack.pop();
            if a.is_number() {
                let a = a.as_number();
                stack.push((-a).into())
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            }

            Signal::More
        })
    }

    pub fn op_not(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let is_falsey = stack.pop().is_falsey();
            stack.push(is_falsey.into());
            Signal::More
        })
    }

    pub fn op_divide(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let b = stack.pop();
            let a = stack.pop();
            if a.is_number() && b.is_number() {
                let a = a.as_number();
                let b = b.as_number();
                stack.push((a / b).into());
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            }

            Signal::More
        })
    }

    pub fn op_multiply(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let b = stack.pop();
            let a = stack.pop();
            if a.is_number() && b.is_number() {
                let a = a.as_number();
                let b = b.as_number();
                stack.push((a * b).into());
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            }

            Signal::More
        })
    }

    pub fn op_subtract(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let b = stack.pop();
            let a = stack.pop();
            if a.is_number() && b.is_number() {
                let a = a.as_number();
                let b = b.as_number();
                stack.push((a - b).into());
            } else {
                return self.fiber.runtime_error(VmError::UnexpectedValue);
            }

            Signal::More
        })
    }

    pub fn op_add(&mut self) -> Signal {
        let (b, a) = self.fiber.with_stack(|stack| {
            (stack.pop(), stack.pop())
        });

        if a.is_number() && b.is_number() {
            let a = a.as_number();
            let b = b.as_number();
            self.fiber.with_stack(|stack| stack.push((a + b).into()));
            return Signal::More;
        }

        self.concat(a, b)
    }

    pub fn op_get_upvalue(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let upvalue = self.fiber.current_frame().closure.upvalues[index];
        let value = self.fiber.resolve_upvalue_into_value(upvalue);
        self.fiber.with_stack(|stack| stack.push(value));

        Signal::More
    }

    pub fn op_set_upvalue(&mut self) -> Signal {
        let index = self.next_u32() as usize;

        let value = self.fiber.with_stack(|stack| {
            stack.peek_n(0)
        });

        let upvalue = self.fiber.current_frame().closure.upvalues[index];
        self.fiber.set_upvalue(upvalue, value);
        Signal::More
    }

    pub fn op_close_upvalue(&mut self) -> Signal {
        let index = self.fiber.with_stack(|stack| {
            stack.len() - 1
        });

        self.fiber.close_upvalues(index);

        self.fiber.with_stack(|stack| {
            stack.pop();
        });

        Signal::More
    }

    //TODO consider redesigning
    #[cold]
    pub fn op_print(&mut self) -> Signal {
        let value = self.fiber.with_stack(|stack| stack.pop());
        self.print(&format!("{}", value));
        Signal::More
    }

    pub fn op_get_local(&mut self) -> Signal {
        let index = self.next_u32() as usize;
        let index = self.fiber.current_frame().base_counter + index;

        self.fiber.with_stack(|stack| {
            let value = stack.get(index);
            stack.push(value);
        });

        Signal::More
    }

    pub fn op_set_local(&mut self) -> Signal {
        let index = self.next_u32() as usize;
        let index = self.fiber.current_frame().base_counter + index;

        self.fiber.with_stack(|stack| {
            let value = stack.peek_n(0);
            stack.set(index, value);
        });

        Signal::More
    }

    pub fn op_equal(&mut self) -> Signal {
        self.fiber.with_stack(|stack| {
            let b = stack.pop();
            let a = stack.pop();

            if Value::is_same_type(a, b) {
                stack.push((a == b).into());
            } else {
                stack.push(false.into())
            }

        });

        Signal::More
    }

    #[cold]
    pub fn op_define_global(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.fiber.current_import();
        let identifier = current_import.symbol(index);
        let value = self.fiber.with_stack(|stack| stack.pop());
        current_import.set_global(identifier, value);

        Signal::More
    }

    pub fn op_get_global(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.fiber.current_import();
        let identifier = current_import.symbol(index);
        let value = current_import.global(identifier);
        if let Some(value) = value {
            self.fiber.with_stack(|stack| stack.push(value));
        } else {
            return self.fiber.runtime_error(VmError::GlobalNotDefined);
        }

        Signal::More
    }

    pub fn op_set_global(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.fiber.current_import();
        let identifier = current_import.symbol(index);
        let value = self.fiber.with_stack(|stack| stack.peek_n(0));
        if current_import.has_global(identifier) {
            current_import.set_global(identifier, value);
        } else {
            return self.fiber.runtime_error(VmError::GlobalNotDefined);
        }

        Signal::More
    }

    pub fn op_set_property(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.fiber.current_import();
        let property = current_import.symbol(index);

        let instance = self.fiber.with_stack(|stack| {
            stack.peek_n(1)
        });

        let instance = as_obj!(self, instance, Instance);

        self.fiber.with_stack(|stack| {
            instance.set_field(property, stack.peek_n(0))
        });

        self.adjust_size(instance);

        self.fiber.with_stack(|stack| {
            let value = stack.pop();
            stack.pop();
            stack.push(value)
        });

        Signal::More
    }

    pub fn op_get_property(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let current_import = self.fiber.current_import();
        let property = current_import.symbol(index);

        let instance = self.fiber.with_stack(|stack| stack.pop());

        if !instance.is_object() {
            return self.fiber.runtime_error(VmError::UnexpectedValue);
        }

        if let Some(instance) = instance.try_cast::<Instance>() {
            if let Some(value) = instance.field(property) {
                self.fiber.with_stack(|stack| {
                    stack.push(value)
                });

                return Signal::More;
            }
        }

        let class = self.builtins.class_for_object(instance.as_object());

        let method = match class.method(property) {
            Some(method) => method,
            None => return self.fiber.runtime_error(VmError::UndefinedProperty),
        };

        let bind: Gc<Object<BoundMethod>> = self.manage(BoundMethod {
            receiver: instance.as_object(),
            method,
        }.into());

        self.fiber.with_stack(|stack| {
            stack.push(Value::from_object(bind))
        });

        Signal::More
    }

    pub fn op_closure(&mut self) -> Signal {
        let index = self.next_u32() as _;

        let closure: Gc<Object<Closure>> = self.manage(Closure::new(index, self.fiber, &self.heap).into());

        self.fiber.with_stack(|stack| {
            stack.push(Value::from_object(closure))
        });

        Signal::More
    }

    pub fn op_invoke(&mut self) -> Signal {
        let arity = self.next_u8() as _;
        let index = self.next_u32() as _;

        let current_import = self.fiber.current_import();
        let property = current_import.symbol(index);

        let instance = self.fiber.with_stack(|stack| {
            stack.peek_n(arity)
        });

        if !instance.is_object() {
            return self.fiber.runtime_error(VmError::UnexpectedValue);
        }

        if let Some(instance) = instance.try_cast::<Instance>() {
            if let Some(value) = instance.field(property) {
                self.fiber.with_stack(|stack| {
                    stack.rset(arity, value);
                });
                return self.call(arity, value);
            }
        }

        let class = self.builtins.class_for_object(instance.as_object());

        let method = match class.method(property) {
            Some(value) => value,
            None => return self.fiber.runtime_error(VmError::UndefinedProperty),
        };

        if let Some(method) = method.try_cast::<Closure>() {
            if method.function.arity != arity {
                return self.fiber.runtime_error(VmError::IncorrectArity);
            }

            self.store_ip();
            self.fiber.begin_frame(method);
            self.load_ip();

            Signal::More
        } else {
            self.call(arity, method)
        }
    }
}



use lox_bytecode::bytecode;
use std::cell::Cell;
use crate::gc::{Gc, Trace, Tracer};
use crate::memory::{Import, Upvalue};
use crate::fiber::Fiber;
use arrayvec::ArrayVec;

pub struct Closure {
    pub function: Function,
    pub upvalues: ArrayVec<Gc<Cell<Upvalue>>, 128>,
}

unsafe impl Trace for Closure {
    fn trace(&self, tracer: &mut Tracer) {
        self.upvalues.trace(tracer);
        self.function.import.trace(tracer);
    }
}

//TODO Drop this entirely and merge this into Closure
pub struct Function {
    pub name: String,
    pub chunk_index: bytecode::ChunkIndex,
    pub import: Gc<Import>,
    pub arity: usize,
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
        .field("name", &self.name)
        .finish()
    }
}

impl Function {
    pub(crate) fn new(value: &bytecode::Function, import: Gc<Import>) -> Self {
        Self {
            name: value.name.clone(),
            chunk_index: value.chunk_index,
            arity: value.arity,
            import,
        }
    }
}


impl Closure {
    pub(crate) fn with_import(import: Gc<Import>) -> Self {
        let function = Function {
            arity: 0,
            chunk_index: 0,
            name: "top".into(),
            import,
        };

        Closure {
            upvalues: ArrayVec::new(),
            function,
        }
    }

    #[inline]
    pub(crate) fn new(index: usize, fiber: Gc<Fiber>) -> Self {
        let import = fiber.current_import();
        let closure = import.closure(index);

        let base = fiber.current_frame().base_counter;

        let upvalues = closure
            .upvalues
            .iter()
            .map(|u| {
                match u {
                    bytecode::Upvalue::Local(index) => {
                        let index = base + index;

                        if let Some(upvalue) = fiber.find_open_upvalue_with_index(index) {
                            upvalue
                        } else {
                            let root = lox_gc::manage(Cell::new(Upvalue::Open(index)));
                            fiber.push_upvalue(root);
                            root
                        }
                    }
                    bytecode::Upvalue::Upvalue(u) => {
                        fiber.find_upvalue_by_index(*u)
                    }
                }
            })
        .collect();

        Self {
            function: Function::new(&closure.function, import),
            upvalues,
        }
    }
}

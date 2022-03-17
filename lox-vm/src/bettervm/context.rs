use std::{io::Write, collections::HashMap};
use lox_bytecode::bytecode::Module;

use crate::bettergc::{UniqueRoot, Heap, Gc, Root, Trace};
use super::{interner::{Interner, Symbol}, memory::{Import, Closure, Function}};

//TODO Drop W because it's not 'static
pub struct VmContext<W> where W: Write {
    pub stdout: W,
    interner: Interner,
    imports: UniqueRoot<HashMap<String, Gc<Import>>>,

    heap: Heap,
}

impl<W: Write> VmContext<W> {
    pub fn new(stdout: W) -> Self {
        let mut heap = Heap::new();
        Self {
            stdout,
            interner: Interner::new(),
            imports: heap.unique(HashMap::new()),
            heap,
        }
    }

    #[inline]
    pub fn collect(&mut self) {
        self.heap.collect();
    }

    #[inline]
    pub fn manage<T: Trace>(&mut self, data: T) -> Root<T> {
        self.heap.manage(data)
    }

    pub fn unique<T: Trace>(&mut self, data: T) -> UniqueRoot<T> {
        self.heap.unique(data)
    }

    #[inline]
    pub fn intern(&mut self, string: &str) -> Symbol {
        self.interner.intern(string)
    }

    pub fn prepare_interpret(&mut self, module: Module) -> Root<Closure> {
        let import = Import::new(module, &mut self.interner);
        let import = self.manage(import);
        self.imports.insert("_root".into(), import.as_gc());

        let function = Function {
            arity: 0,
            chunk_index: 0,
            name: "top".into(),
            import: import.as_gc(),
        };
        let closure = self.manage(Closure {
            upvalues: vec![],
            function,
        });
        closure
    }
}
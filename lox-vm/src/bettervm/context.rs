use std::collections::HashMap;
use lox_bytecode::bytecode::Module;

use crate::bettergc::{Heap, Gc, Trace};
use super::{interner::{Interner, Symbol}, memory::{Import, Closure, Function}};

pub struct VmContext {
    pub print: for<'r> fn(&'r str),
    interner: Interner,
    imports: HashMap<String, Gc<Import>>,

    heap: Heap,
}

impl Trace for VmContext {
    fn trace(&self) {
        self.imports.trace();
    }
}

impl VmContext {
    pub fn new(print: for<'r> fn(&'r str)) -> Self {
        let heap = Heap::new();
        Self {
            print,
            interner: Interner::new(),
            imports: HashMap::new(),
            heap,
        }
    }

    #[inline]
    pub fn collect(&mut self, root: &dyn Trace) {
        self.heap.collect(&[root, &self.imports]);
    }

    #[inline]
    pub fn manage<T: Trace>(&mut self, data: T) -> Gc<T> {
        self.heap.manage(data)
    }

    #[inline]
    pub fn intern(&mut self, string: &str) -> Symbol {
        self.interner.intern(string)
    }

    pub fn prepare_interpret(&mut self, module: Module) -> Gc<Closure> {
        let import = Import::new(module, &mut self.interner);
        let import = self.manage(import);
        self.imports.insert("_root".into(), import);

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
        closure
    }
}

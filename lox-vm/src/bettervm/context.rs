use std::{io::Write, collections::HashMap};
use lox_bytecode::bytecode::Module;
//use crate::bettervm::Value;

use crate::bettergc::{Heap, Gc, Trace};
use super::{interner::{Interner, Symbol}, memory::{Import, Closure, Function}};

//TODO Drop W because it's not 'static
pub struct VmContext<W> where W: Write {
    pub stdout: W,
    //pub print: fn(&Value),
    interner: Interner,
    imports: HashMap<String, Gc<Import>>,

    heap: Heap,
}

impl<W: Write> Trace for VmContext<W> {
    fn trace(&self) {
        self.imports.trace();
    }
}

impl<W: Write> VmContext<W> {
    pub fn new(stdout: W) -> Self {
        let heap = Heap::new();
        Self {
            stdout,
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

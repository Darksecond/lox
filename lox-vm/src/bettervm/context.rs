use std::{io::Write, collections::HashMap};
use crate::bettergc::{UniqueRoot, Heap, Gc, Root, Trace};
use super::{interner::Interner, memory::Import};

pub struct VmContext<W> where W: Write {
    pub stdout: W,
    pub interner: Interner,
    pub imports: UniqueRoot<HashMap<String, Gc<Import>>>,
    
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

    pub fn collect(&mut self) {
        self.heap.collect();
    }

    pub fn manage<T: Trace>(&mut self, data: T) -> Root<T> {
        self.heap.manage(data)
    }

    pub fn unique<T: Trace>(&mut self, data: T) -> UniqueRoot<T> {
        self.heap.unique(data)
    }
}
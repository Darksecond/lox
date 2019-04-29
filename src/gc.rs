use std::fmt;
use std::ops::Deref;
use std::hash::Hash;
use std::collections::HashMap;
use std::cell::{RefCell, Cell};

pub trait Trace {
    fn trace(&self);
}

impl<T: Trace> Trace for RefCell<T> {
    fn trace(&self) {
        self.borrow().trace();
    }
}

impl<T: Trace> Trace for Vec<T> {
    fn trace(&self) {
        for el in self {
            el.trace();
        }
    }
}

impl<K: Eq + Hash, T: Trace> Trace for HashMap<K, T> {
    fn trace(&self) {
        for val in self.values() {
            val.trace();
        }
    }
}


#[derive(Debug)]
pub struct Root<T> {
    marked: Cell<bool>,
    data: T,
}

impl<T: Trace> Root<T> {
    pub fn new(data: T) -> Root<T> {
        Root{marked: Cell::new(false), data}
    }

    pub fn as_gc(&self) -> Gc<T> {
        Gc{ptr: self}
    }

    pub fn unmark(&self) {
        self.marked.replace(false);
    }

    pub fn marked(&self) -> bool {
        self.marked.get()
    }
}


pub struct Gc<T> {
    ptr: *const Root<T>,
}

impl<T: Trace> Trace for Gc<T> {
    fn trace(&self) {
        if !unsafe { (*self.ptr).marked.replace(true) } {
            self.deref().trace();
        }
        //TODO
    }
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Gc<T> {
        *self
    }
}

impl<T> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &(*self.ptr).data }
    }
}

impl<T> Copy for Gc<T> { }

impl<T: fmt::Debug> fmt::Debug for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &*self;
        write!(f, "Gc({:?})", inner)
    }
}
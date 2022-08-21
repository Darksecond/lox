use std::cell::Cell;
use std::fmt;
use std::ops::Deref;
use std::ptr::NonNull;

pub trait Trace {
    fn trace(&self);
}

impl fmt::Debug for dyn Trace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Trace>")
    }
}

#[derive(Debug)]
struct Header {
    marked: Cell<bool>,
}

#[derive(Debug)]
struct Allocation<T: 'static + Trace + ?Sized> {
    header: Header,
    data: T,
}

#[derive(Debug)]
pub struct Heap {
    objects: Vec<Box<Allocation<dyn Trace>>>,
    bytes_allocated: usize,
    threshold: usize,
}

pub struct Gc<T: 'static + Trace + ?Sized> {
    ptr: NonNull<Allocation<T>>,
}

impl<T: 'static + Trace + ?Sized> Allocation<T> {
    fn unmark(&self) {
        self.header.marked.set(false);
    }
}
impl<T: 'static + Trace + ?Sized> Trace for Allocation<T> {
    fn trace(&self) {
        if !self.header.marked.replace(true) {
            self.data.trace();
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Header {
            marked: Cell::new(false),
        }
    }
}

impl Heap {
    const THRESHOLD_ADJ: f32 = 1.4;

    pub fn new() -> Self {
        Heap {
            objects: Vec::with_capacity(8192),
            bytes_allocated: 0,
            threshold: 100,
        }
    }

    fn allocate<T: 'static + Trace>(&mut self, data: T) -> NonNull<Allocation<T>> {
        let mut alloc = Box::new(Allocation {
            header: Header::default(),
            data,
        });
        let ptr = unsafe { NonNull::new_unchecked(&mut *alloc) };
        self.objects.push(alloc);
        self.bytes_allocated += std::mem::size_of::<T>();
        ptr
    }

    pub fn manage<T: 'static + Trace>(&mut self, data: T) -> Gc<T> {
        Gc {
            ptr: self.allocate(data),
        }
    }

    #[inline]
    pub fn collect(&mut self, roots: &[&dyn Trace]) {
        if self.should_collect() {
            self.force_collect(roots);
        }
    }

    #[inline]
    pub fn should_collect(&self) -> bool {
        self.bytes_allocated > self.threshold
    }

    pub fn force_collect(&mut self, roots: &[&dyn Trace]) {
        self.mark(roots);
        let bytes = self.bytes_unmarked();
        self.sweep();

        self.bytes_allocated -= bytes;
        self.threshold = (self.bytes_allocated as f32 * Self::THRESHOLD_ADJ) as usize;
        self.threshold += 100; // Offset by 100 so it never reaches 0
    }

    fn mark(&mut self, roots: &[&dyn Trace]) {
        for object in &self.objects {
            object.unmark();
        }

        roots
            .iter()
            .for_each(|o| o.trace());
    }

    fn sweep(&mut self) {
        self.objects.retain(|o| o.header.marked.get());
    }

    fn bytes_unmarked(&self) -> usize {
        let mut bytes = 0;
        for object in &self.objects {
            if !object.header.marked.get() {
                bytes += std::mem::size_of_val(&object.data);
            }
        }
        bytes
    }
}

impl<T: 'static + Trace + ?Sized> Gc<T> {
    #[inline]
    fn allocation(&self) -> &Allocation<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub fn ptr_eq(a: &Gc<T>, b: &Gc<T>) -> bool{
        a.ptr == b.ptr
    }
}
impl<T: 'static + Trace + ?Sized> Copy for Gc<T> {}
impl<T: 'static + Trace + ?Sized> Clone for Gc<T> {
    #[inline]
    fn clone(&self) -> Gc<T> {
        *self
    }
}

impl<T: 'static + Trace + ?Sized> Deref for Gc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.allocation().data
    }
}

impl<T: fmt::Debug + 'static + Trace + ?Sized> fmt::Debug for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &**self;
        write!(f, "Gc({:?})", inner)
    }
}
impl<T: fmt::Display + 'static + Trace + ?Sized> fmt::Display for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &**self;
        inner.fmt(f)
    }
}
impl<T: 'static + Trace + ?Sized> Trace for Gc<T> {
    #[inline]
    fn trace(&self) {
        self.allocation().trace();
    }
}

use std::cell::RefCell;
use std::collections::HashMap;
use fxhash::FxHashMap;
use std::hash::Hash;
impl<T: Trace> Trace for RefCell<T> {
    #[inline]
    fn trace(&self) {
        self.borrow().trace();
    }
}
impl<T: Trace> Trace for Vec<T> {
    #[inline]
    fn trace(&self) {
        for el in self {
            el.trace();
        }
    }
}
impl<T: Trace> Trace for &Vec<T> {
    #[inline]
    fn trace(&self) {
        for el in *self {
            el.trace();
        }
    }
}
impl<K: Eq + Hash, T: Trace> Trace for HashMap<K, T> {
    #[inline]
    fn trace(&self) {
        for val in self.values() {
            val.trace();
        }
    }
}
impl<K: Eq + Hash, T: Trace> Trace for FxHashMap<K, T> {
    #[inline]
    fn trace(&self) {
        for val in self.values() {
            val.trace();
        }
    }
}
impl Trace for String {
    #[inline]
    fn trace(&self) {}
}

impl<T: Trace + Copy> Trace for Cell<T> {
    #[inline]
    fn trace(&self) {
        self.get().trace();
    }
}

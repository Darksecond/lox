use std::cell::{Cell, UnsafeCell};
use std::fmt;
use std::ops::Deref;
use std::ptr::NonNull;

pub trait Trace {
    fn trace(&self);
    fn size_hint(&self) -> usize { 0 }
}

impl fmt::Debug for dyn Trace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Trace>")
    }
}

#[derive(Debug)]
#[repr(C)]
struct Allocation<T: 'static + Trace + ?Sized> {
    next: Cell<Option<NonNull<Allocation<dyn Trace>>>>,
    marked: Cell<bool>,
    size: Cell<usize>,
    data: T,
}

#[derive(Debug)]
pub struct Heap {
    next: Cell<Option<NonNull<Allocation<dyn Trace>>>>,
    bytes_allocated: Cell<usize>,
    threshold: Cell<usize>,
}

pub struct Gc<T: 'static + Trace> {
    ptr: NonNull<Allocation<T>>,
}

impl<T: 'static + Trace + ?Sized> Allocation<T> {
    fn unmark(&self) {
        self.marked.set(false);
    }
}
impl<T: 'static + Trace + ?Sized> Trace for Allocation<T> {
    fn trace(&self) {
        if !self.marked.replace(true) {
            self.data.trace();
        }
    }
}

impl Heap {
    const THRESHOLD_ADJ: f32 = 2.0;

    pub fn new() -> Self {
        Heap {
            next: Cell::new(None),
            bytes_allocated: Cell::new(0),
            threshold: Cell::new(1024 * 1024),
        }
    }

    fn allocate<T: 'static + Trace>(&self, data: T) -> NonNull<Allocation<T>> {
        let size = std::mem::size_of::<T>() + data.size_hint();

        let alloc = Box::new(Allocation {
            marked: Cell::new(false),
            next: Cell::new(self.next.get()),
            size: Cell::new(size),
            data,
        });

        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(alloc)) };

        self.next.set(Some(ptr));
        self.bytes_allocated.set(self.bytes_allocated.get() + size);

        ptr
    }

    pub fn adjust_size<T: 'static + Trace>(&self, object: Gc<T>) {
        self.bytes_allocated.set(self.bytes_allocated.get() - object.allocation().size.get());

        let size = std::mem::size_of::<T>() + object.size_hint();
        object.allocation().size.set(size);

        self.bytes_allocated.set(self.bytes_allocated.get() + size);
    }

    pub fn manage<T: 'static + Trace>(&self, data: T) -> Gc<T> {
        Gc {
            ptr: self.allocate(data),
        }
    }

    #[inline]
    pub fn collect(&self, roots: &[&dyn Trace]) {
        if self.should_collect() {
            self.force_collect(roots);
        }
    }

    #[inline]
    pub fn should_collect(&self) -> bool {
        self.bytes_allocated > self.threshold
    }

    #[inline(never)]
    pub fn force_collect(&self, roots: &[&dyn Trace]) {
        self.mark(roots);
        self.sweep();

        self.threshold.set(((self.bytes_allocated.get() as f32 * Self::THRESHOLD_ADJ) as usize) + 100);
    }

    fn mark(&self, roots: &[&dyn Trace]) {
        roots
            .iter()
            .for_each(|o| o.trace());
    }

    fn sweep(&self) {
        let mut previous = None;
        let mut next = self.next.get();

        while let Some(ptr) = next {
            let object = unsafe { ptr.as_ref() };
            let marked = object.marked.get();

            next = object.next.get();

            if marked {
                object.unmark();
                previous = Some(ptr);
            } else {
                if let Some(previous_ptr) = previous {
                    let previous_object = unsafe { previous_ptr.as_ref() };
                    previous_object.next.set(next);
                } else {
                    self.next.set(next);
                }

                self.free_object(ptr);
            }
        }
    }

    fn free_object(&self, mut ptr: NonNull<Allocation<dyn Trace>>) {
        {
            let object = unsafe { ptr.as_ref() };
            self.bytes_allocated.set(self.bytes_allocated.get() - object.size.get());
        }

        unsafe {
            drop(Box::from_raw(ptr.as_mut()));
        }
    }
}

// Prevent memory leaks when dropping heap
impl Drop for Heap {
    fn drop(&mut self) {
        self.mark(&[]);
        self.sweep();
    }
}

impl<T: 'static + Trace> Gc<T> {
    #[inline]
    fn allocation(&self) -> &Allocation<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub fn ptr_eq(a: &Gc<T>, b: &Gc<T>) -> bool {
        a.ptr == b.ptr
    }
}

impl<T: 'static + Trace> Gc<T> {
    #[inline]
    pub fn to_bits(self) -> u64 {
        self.ptr.as_ptr() as u64
    }

    #[inline]
    pub unsafe fn from_bits(value: u64) -> Self {
        Self {
            ptr: NonNull::new_unchecked(value as *mut Allocation<T>),
        }
    }
}

impl<T: 'static + Trace> Copy for Gc<T> {}
impl<T: 'static + Trace> Clone for Gc<T> {
    #[inline]
    fn clone(&self) -> Gc<T> {
        *self
    }
}

impl<T: 'static + Trace> Deref for Gc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.allocation().data
    }
}

impl<T: fmt::Debug + 'static + Trace> fmt::Debug for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &**self;
        write!(f, "Gc({:?})", inner)
    }
}
impl<T: fmt::Display + 'static + Trace> fmt::Display for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &**self;
        inner.fmt(f)
    }
}
impl<T: 'static + Trace> Trace for Gc<T> {
    #[inline]
    fn trace(&self) {
        self.allocation().trace();
    }
}

use std::cell::RefCell;
use std::collections::HashMap;
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

impl Trace for String {
    #[inline]
    fn trace(&self) {}

    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl<T: Trace + Copy> Trace for Cell<T> {
    #[inline]
    fn trace(&self) {
        self.get().trace();
    }
}

impl<T: Trace> Trace for UnsafeCell<T> {
    fn trace(&self) {
        let inner = unsafe { &*self.get() };
        inner.trace();
    }
}

impl<T: Trace> Trace for Option<T> {
    fn trace(&self) {
        match self {
            Some(inner) => inner.trace(),
            None => (),
        }
    }
}

use std::cell::Cell;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    roots: AtomicUsize,
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

pub struct Root<T: 'static + Trace + ?Sized> {
    ptr: NonNull<Allocation<T>>,
}

pub struct UniqueRoot<T: 'static + Trace + ?Sized> {
    ptr: NonNull<Allocation<T>>,
}

impl<T: 'static + Trace + ?Sized> Allocation<T> {
    fn unmark(&self) {
        self.header.marked.set(false);
    }

    #[inline]
    fn root(&self) {
        self.header.roots.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    fn unroot(&self) {
        self.header.roots.fetch_sub(1, Ordering::Relaxed);
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
            roots: AtomicUsize::new(0),
            marked: Cell::new(false),
        }
    }
}

impl Heap {
    const THRESHOLD_ADJ: f32 = 1.4;

    pub fn new() -> Self {
        Heap {
            objects: vec![],
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

    /// Create a UniqueRoot, it cannot be Copied or Cloned, but it is mutably dereferencing.
    /// Which means it's ideal for Root containers and such.
    pub fn unique<T: 'static + Trace>(&mut self, data: T) -> UniqueRoot<T> {
        let root = UniqueRoot {
            ptr: self.allocate(data),
        };
        root.allocation().root();
        root
    }

    #[inline]
    pub fn root<T: 'static + Trace + ?Sized>(&mut self, obj: Gc<T>) -> Root<T> {
        obj.allocation().root();
        Root { ptr: obj.ptr }
    }

    pub fn manage<T: 'static + Trace>(&mut self, data: T) -> Root<T> {
        let root = Root {
            ptr: self.allocate(data),
        };
        root.allocation().root();
        root
    }

    pub fn collect(&mut self) -> usize {
        if self.bytes_allocated > self.threshold {
            self.force_collect()
        } else {
            0
        }
    }

    fn force_collect(&mut self) -> usize {
        self.mark();
        let bytes = self.bytes_marked();
        self.sweep();

        self.bytes_allocated -= bytes;
        self.threshold = (self.bytes_allocated as f32 * Self::THRESHOLD_ADJ) as usize;

        bytes
    }

    fn mark(&mut self) {
        for object in &self.objects {
            object.unmark();
        }
        self.objects
            .iter()
            .filter(|o| o.header.roots.load(Ordering::Relaxed) > 0)
            .for_each(|o| o.trace());
    }

    fn sweep(&mut self) {
        self.objects.retain(|o| o.header.marked.get());
    }

    fn bytes_marked(&self) -> usize {
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
        unsafe { &self.ptr.as_ref() }
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
        let inner: &T = &*self;
        write!(f, "Gc({:?})", inner)
    }
}
impl<T: fmt::Display + 'static + Trace + ?Sized> fmt::Display for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &*self;
        inner.fmt(f)
    }
}
impl<T: 'static + Trace + ?Sized> Trace for Gc<T> {
    #[inline]
    fn trace(&self) {
        self.allocation().trace();
    }
}

impl<T: 'static + Trace + ?Sized> Trace for Root<T> {
    #[inline]
    fn trace(&self) {
        self.allocation().trace();
    }
}
impl<T: 'static + Trace + ?Sized> Clone for Root<T> {
    #[inline]
    fn clone(&self) -> Root<T> {
        self.allocation().root();
        Root { ptr: self.ptr }
    }
}
impl<T: 'static + Trace + ?Sized> Root<T> {
    #[inline]
    fn allocation(&self) -> &Allocation<T> {
        unsafe { &self.ptr.as_ref() }
    }

    #[inline]
    pub fn as_gc(&self) -> Gc<T> {
        Gc { ptr: self.ptr }
    }
}
impl<T: 'static + Trace + ?Sized> Drop for Root<T> {
    #[inline]
    fn drop(&mut self) {
        self.allocation().unroot();
    }
}
impl<T: 'static + Trace + ?Sized> Deref for Root<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.allocation().data
    }
}
impl<T: fmt::Debug + 'static + Trace + ?Sized> fmt::Debug for Root<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &*self;
        write!(f, "Root({:?})", inner)
    }
}

impl<T: 'static + Trace + ?Sized> Trace for UniqueRoot<T> {
    #[inline]
    fn trace(&self) {
        self.allocation().trace();
    }
}
impl<T: 'static + Trace + ?Sized> UniqueRoot<T> {
    #[inline]
    fn allocation_mut(&mut self) -> &mut Allocation<T> {
        unsafe { self.ptr.as_mut() }
    }

    #[inline]
    fn allocation(&self) -> &Allocation<T> {
        unsafe { &self.ptr.as_ref() }
    }
}
impl<T: 'static + Trace + ?Sized> Drop for UniqueRoot<T> {
    fn drop(&mut self) {
        self.allocation().unroot();
    }
}
impl<T: 'static + Trace + ?Sized> Deref for UniqueRoot<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.allocation().data
    }
}
impl<T: 'static + Trace + ?Sized> DerefMut for UniqueRoot<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.allocation_mut().data
    }
}
impl<T: fmt::Debug + 'static + Trace + ?Sized> fmt::Debug for UniqueRoot<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &*self;
        write!(f, "UniqueRoot({:?})", inner)
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

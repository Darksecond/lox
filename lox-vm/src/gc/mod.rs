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
struct Allocation<T: ?Sized> {
    next: Cell<Option<NonNull<Allocation<()>>>>,
    vtable: *mut (),
    marked: Cell<bool>,
    size: Cell<usize>,
    data: T,
}

#[derive(Debug)]
pub struct Heap {
    next: Cell<Option<NonNull<Allocation<()>>>>,
    bytes_allocated: Cell<usize>,
    threshold: Cell<usize>,
}

pub struct Gc<T: ?Sized> {
    ptr: NonNull<Allocation<T>>,
}

impl<T: ?Sized> Allocation<T> {
    fn unmark(&self) {
        self.marked.set(false);
    }

    fn erased(&self) -> &Allocation<()> {
        let ptr = self as *const Allocation<T>;
        let ptr = ptr as *const Allocation<()>;
        unsafe {
            &*ptr
        }
    }

    pub fn dyn_data(&self) -> &dyn Trace {
        let data = &self.erased().data as *const ();
        unsafe {
            vtable::construct(data, self.vtable)
        }
    }

    pub fn dyn_data_mut(&self) -> &mut dyn Trace {
        let data = &self.erased().data as *const ();
        unsafe {
            vtable::construct_mut(data, self.vtable)
        }
    }
}

impl<T: Trace> Allocation<T> {
    pub fn new(data: T) -> NonNull<Allocation<T>> {
        let size = std::mem::size_of::<T>() + data.size_hint();

        let vtable = vtable::extract(&data);

        let alloc = Box::new(Allocation {
            marked: Cell::new(false),
            next: Cell::new(None),
            size: Cell::new(size),
            vtable,
            data,
        });

        unsafe {
            NonNull::new_unchecked(Box::into_raw(alloc))
        }
    }
}

impl<T: ?Sized> Trace for Allocation<T> {
    fn trace(&self) {
        if !self.marked.replace(true) {
            self.dyn_data().trace();
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

    fn allocate<T: Trace>(&self, data: T) -> Gc<T> {
        let ptr = Allocation::new(data);
        Gc {
            ptr,
        }
    }

    pub fn adjust_size<T: 'static + Trace>(&self, object: Gc<T>) {
        self.bytes_allocated.set(self.bytes_allocated.get() - object.allocation().size.get());

        let size = std::mem::size_of::<T>() + object.size_hint();
        object.allocation().size.set(size);

        self.bytes_allocated.set(self.bytes_allocated.get() + size);
    }

    pub fn manage<T: Trace>(&self, data: T) -> Gc<T> {
        let gc = self.allocate(data);

        gc.allocation().next.set(self.next.get());
        self.next.set(Some(gc.erased()));

        self.bytes_allocated.set(self.bytes_allocated.get() + gc.allocation().size.get());

        gc
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

    fn free_object(&self, ptr: NonNull<Allocation<()>>) {
        let allocation = unsafe { ptr.as_ref() };

        self.bytes_allocated.set(self.bytes_allocated.get() - allocation.size.get());

        // Call destructor for data.
        unsafe {
            let ptr = allocation.dyn_data_mut() as *mut dyn Trace;
            std::ptr::drop_in_place(ptr);
        }

        drop(allocation);

        // Deallocate data.
        unsafe {
            drop(Box::from_raw(ptr.as_ptr()));
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

impl<T: ?Sized> Gc<T> {
    #[inline]
    fn allocation(&self) -> &Allocation<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub fn ptr_eq(a: &Gc<T>, b: &Gc<T>) -> bool {
        a.ptr == b.ptr
    }

    fn erased(&self) -> NonNull<Allocation<()>> {
        let ptr = self.ptr.as_ptr() as *mut Allocation<()>;

        unsafe {
            NonNull::new_unchecked(ptr)
        }
    }
}

impl<T> Gc<T> {
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

impl<T: ?Sized> Copy for Gc<T> {}
impl<T: ?Sized> Clone for Gc<T> {
    #[inline]
    fn clone(&self) -> Gc<T> {
        *self
    }
}

impl<T: ?Sized> Deref for Gc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.allocation().data
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &**self;
        write!(f, "Gc({:?})", inner)
    }
}
impl<T: fmt::Display + ?Sized> fmt::Display for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &**self;
        inner.fmt(f)
    }
}
impl<T: ?Sized> Trace for Gc<T> {
    #[inline]
    fn trace(&self) {
        self.allocation().trace();
    }
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use arrayvec::ArrayVec;
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

impl<T: Trace, const C: usize> Trace for ArrayVec<T, C> {
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

mod vtable {
    use super::Trace;

    #[repr(C)]
    struct Object {
        data: *const (),
        vtable: *mut (),
    }

    pub fn extract<T: Trace>(data: &T) -> *mut () {
        unsafe {
            let obj = data as &dyn Trace;
            std::mem::transmute::<&dyn Trace, Object>(obj).vtable
        }
    }

    pub unsafe fn construct<'a>(data: *const (), vtable: *mut ()) -> &'a dyn Trace {
        unsafe {
            let object = Object {
                data,
                vtable,
            };
            std::mem::transmute::<Object, &dyn Trace>(object)
        }
    }

    pub unsafe fn construct_mut<'a>(data: *const (), vtable: *mut ()) -> &'a mut dyn Trace {
        unsafe {
            let object = Object {
                data,
                vtable,
            };
            std::mem::transmute::<Object, &mut dyn Trace>(object)
        }
    }
}

mod heap;

use std::{ptr::NonNull, ops::Deref, any::TypeId, cell::Cell};

thread_local! {
    pub static HEAP: ManagedHeap = ManagedHeap::new();
}

pub fn manage<T>(data: T) -> Gc<T> where T: Trace + 'static {
    HEAP.with(|heap| {
        heap.manage(data)
    })
}

pub unsafe fn alloc(layout: std::alloc::Layout) -> *mut u8 {
    HEAP.with(|heap| {
        heap.alloc(layout)
    })
}

pub fn collect(roots: &[&dyn Trace]) {
    HEAP.with(|heap| {
        heap.collect(roots)
    })
}

pub unsafe fn mark(ptr: *const u8) {
    HEAP.with(|heap| {
        heap.heap.mark(ptr)
    })
}


//TODO Move to gc.rs

#[repr(C)]
struct Allocation<T: ?Sized> {
    tag: TypeId,
    vtable: *mut (),
    data: T,
}

impl<T: Trace + 'static> Allocation<T> {
    pub fn new(data: T) -> Allocation<T> {
        let vtable = vtable::extract(&data);

        Allocation {
            tag: TypeId::of::<T>(),
            vtable,
            data,
        }
    }
}

pub struct Tracer<'heap> {
    heap: &'heap ManagedHeap,
}

impl Tracer<'_> {
    pub unsafe fn mark(&self, ptr: *const u8) {
        self.heap.heap.mark(ptr);
    }
}

pub unsafe trait Trace {
    fn trace(&self, tracer: &mut Tracer);
}

pub struct ManagedHeap {
    heap: heap::Heap,
    threshold: Cell<usize>,
}

impl ManagedHeap {
    const THRESHOLD_ADJ: f32 = 2.0;

    pub fn new() -> Self {
        Self {
            threshold: Cell::new(1024 * 1024),
            heap: heap::Heap::new().unwrap(),
        }
    }

    pub unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        self.heap.alloc(layout)
    }

    pub fn manage<T>(&self, data: T) -> Gc<T> where T: Trace + 'static {
        let layout = std::alloc::Layout::new::<Allocation<T>>();
        let ptr = self.heap.alloc(layout) as *mut Allocation<T>;
        unsafe {
            ptr.write(Allocation::new(data));

            Gc {
                ptr: NonNull::new_unchecked(ptr),
            }
        }
    }

    pub fn collect(&self, roots: &[&dyn Trace]) {
        if self.heap.bytes_used() > self.threshold.get() {
            self.force_collect(roots);
            self.threshold.set(((self.heap.bytes_used() as f32 * Self::THRESHOLD_ADJ) as usize) + 100);
        }
    }

    pub fn force_collect(&self, roots: &[&dyn Trace]) {
        unsafe {
            self.heap.start_gc();
        }

        let mut tracer = Tracer {
            heap: self,
        };

        for root in roots {
            root.trace(&mut tracer);
        }

        unsafe {
            self.heap.sweep();
        }
    }
}

pub struct Gc<T: ?Sized> {
    ptr: NonNull<Allocation<T>>,
}

impl<T: ?Sized> Copy for Gc<T> {}
impl<T: ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Gc<T> {
    #[inline]
    pub fn ptr_eq(a: Gc<T>, b: Gc<T>) -> bool {
        a.ptr == b.ptr
    }
}

impl Gc<()> {
    pub fn is<T>(self) -> bool where T: 'static {
        self.allocation().tag == TypeId::of::<T>()
    }

    pub fn cast<T>(self) -> Gc<T> where T: 'static {
        debug_assert_eq!(self.allocation().tag, TypeId::of::<T>());

        Gc {
            ptr: unsafe {
                NonNull::new_unchecked(self.ptr.as_ptr() as *mut Allocation<T>)
            },
        }
    }

    pub fn try_cast<T>(self) -> Option<Gc<T>> where T: 'static {
        if self.is::<T>() {
            Some(self.cast::<T>())
        } else {
            None
        }
    }
}

impl<T> Gc<T> where T: 'static {
    pub fn erase(self) -> Gc<()> {
        Gc {
            ptr: unsafe {
                NonNull::new_unchecked(self.ptr.as_ptr() as *mut Allocation<()>)
            },
        }
    }

    pub fn is_same_type(a: &Self, b: &Self) -> bool {
        a.allocation().tag == b.allocation().tag
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

impl<T: ?Sized> Deref for Gc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        let allocation = unsafe {
            self.ptr.as_ref()
        };

        &allocation.data
    }
}

impl<T: ?Sized> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(*self, *other)
    }
}

impl<T: ?Sized> std::fmt::Display for Gc<T> where T: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T: ?Sized> Gc<T> {
    fn allocation(&self) -> &Allocation<T> {
        unsafe {
            self.ptr.as_ref()
        }
    }

    fn dyn_data(&self) -> &dyn Trace {
        let ptr = self.ptr.as_ptr() as *const Allocation<()>;
        unsafe {
            let data = std::ptr::addr_of!((*ptr).data);
            vtable::construct(data, self.allocation().vtable)
        }
    }
}

unsafe impl<T: ?Sized> Trace for Gc<T> {

    fn trace(&self, tracer: &mut Tracer) {
        let ptr = self.ptr.as_ptr() as *const u8;

        if !tracer.heap.heap.is_marked(ptr) {
            unsafe {
                tracer.heap.heap.mark(ptr);
            }

            self.dyn_data().trace(tracer);
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

    pub fn extract<T: Trace>(data: *const T) -> *mut () {
        unsafe {
            let obj = data as *const dyn Trace;
            std::mem::transmute::<*const dyn Trace, Object>(obj).vtable
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
}

mod trace_impls {
    use super::*;
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::cell::{UnsafeCell, Cell};
    use arrayvec::ArrayVec;

    unsafe impl Trace for String {
        fn trace(&self, _tracer: &mut Tracer) {}
    }

    unsafe impl<T: Trace> Trace for Option<T> {
        fn trace(&self, tracer: &mut Tracer) {
            match self {
                Some(inner) => inner.trace(tracer),
                None => (),
            }
        }
    }

    unsafe impl<K: Eq + Hash + Trace, T: Trace> Trace for HashMap<K, T> {
        #[inline]
        fn trace(&self, tracer: &mut Tracer) {
            for key in self.keys() {
                key.trace(tracer);
            }
            for val in self.values() {
                val.trace(tracer);
            }
        }
    }

    unsafe impl<T: Trace> Trace for Vec<T> {
        #[inline]
        fn trace(&self, tracer: &mut Tracer) {
            for el in self {
                el.trace(tracer);
            }
        }
    }

    unsafe impl<T: Trace> Trace for UnsafeCell<T> {
        fn trace(&self, tracer: &mut Tracer) {
            let inner = unsafe { &*self.get() };
            inner.trace(tracer);
        }
    }

    unsafe impl<T> Trace for Cell<T> where T: Trace + Copy + Clone {
        fn trace(&self, tracer: &mut Tracer) {
            self.get().trace(tracer);
        }
    }

    unsafe impl<T: Trace, const C: usize> Trace for ArrayVec<T, C> {
        #[inline]
        fn trace(&self, tracer: &mut Tracer) {
            for el in self {
                el.trace(tracer);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    unsafe impl Trace for u32 {
        fn trace(&self, _tracer: &mut Tracer) {}
    }


    #[test]
    fn it_works() {
        let heap = ManagedHeap::new();
        let x = heap.manage(std::cell::Cell::new(1234));
        assert_eq!(x.get(), 1234);
        x.set(2345);
        assert_eq!(x.get(), 2345);
    }
}

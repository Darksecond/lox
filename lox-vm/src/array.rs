use lox_gc::{Trace, Tracer};
use std::ptr::{self, NonNull};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::alloc::Layout;

pub struct Array<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
}

impl<T> Default for Array<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<T> for Array<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();

        //TODO use size_hint
        let mut array = Self::new();

        for elem in iter {
            array.push(elem);
        }

        array
    }
}

impl<T> Array<T> where T: Copy {
    pub fn with_contents(elem: T, size: usize) -> Self {
        let mut array = Self::with_capacity(size);

        for _ in 0..size {
            array.push(elem);
        }

        array
    }
}

impl<T> Array<T> where T: Clone {
    //TODO rewrite.
    pub fn extend_from_slice(&mut self, other: &[T]) {
        for elem in other {
            self.push(elem.clone());
        }
    }
}

impl<T> Clone for Array<T> {
    fn clone(&self) -> Self {
        unsafe {
            let ptr = lox_gc::alloc(std::alloc::Layout::array::<T>(self.cap).unwrap()) as *mut T;
            std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), ptr, self.cap);

            Self {
                cap: self.cap,
                len: self.len,
                ptr: NonNull::new_unchecked(ptr),
            }
        }
    }
}

impl<T> Array<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "must not be ZST");

        Self {
            cap: 0,
            len: 0,
            ptr: NonNull::dangling(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Array::new();
        }

        let ptr = unsafe {
            NonNull::new_unchecked(lox_gc::alloc(Layout::array::<T>(capacity).unwrap()) as *mut T)
        };

        Self {
            cap: capacity,
            len: 0,
            ptr,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap { self.grow() }

        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), value);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;

            unsafe {
                Some(ptr::read(self.ptr.as_ptr().add(self.len)))
            }
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("swap_remove index (is {index}) should be < len (is {len})");
        }

        let len = self.len;
        if index >= len {
            assert_failed(index, len);
        }
        unsafe {
            // We replace self[index] with the last element. Note that if the
            // bounds check above succeeds there must be a last element (which
            // can be self[index] itself).
            let value = ptr::read(self.as_ptr().add(index));
            let base_ptr = self.as_mut_ptr();
            ptr::copy(base_ptr.add(len - 1), base_ptr.add(index), 1);
            self.len -= 1;
            value
        }
    }

    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.cap;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = if self.cap == 0 {
            let new_ptr = unsafe { lox_gc::alloc(new_layout) };
            dbg!(new_layout, new_ptr);
            new_ptr
        } else {
            //TODO We copy more than we need to.
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            let new_ptr = unsafe { lox_gc::alloc(new_layout) };
            dbg!(new_layout, old_layout.size(), old_ptr, new_ptr);

            unsafe {
                ptr::copy(old_ptr, new_ptr, old_layout.size());
            }

            new_ptr
        };

        self.ptr = unsafe { NonNull::new_unchecked(new_ptr as *mut T) };
        self.cap = new_cap;
    }

    pub fn mark(&self) {
        unsafe {
            lox_gc::mark(self.ptr.as_ptr() as *const u8);
        }
    }
}

impl<T> Deref for Array<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

unsafe impl<T> Trace for Array<T> where T: Trace {
    fn trace(&self, tracer: &mut Tracer) {
        self.mark();

        for elem in self.iter() {
            elem.trace(tracer);
        }
    }
}

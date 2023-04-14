use std::ops::Deref;
use crate::gc::{Gc, Trace, Tracer};
use super::{Closure, BoundMethod, NativeFunction, Class, Instance, Import, List};
use std::fmt::Display;
use std::any::TypeId;

#[derive(Debug)]
#[repr(C)]
pub struct ErasedObject {
    tag: TypeId,
}

#[derive(Debug)]
#[repr(C)]
pub struct Object<T> {
    tag: TypeId,
    data: T,
}

impl<T> Display for Object<T> where T: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<T> Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

unsafe impl<T: Trace> Trace for Object<T> {
    fn trace(&self, tracer: &mut Tracer) {
        self.data.trace(tracer);
    }
}

impl<T> From<T> for Object<T> where T: 'static {
    fn from(value: T) -> Self {
        Self {
            tag: TypeId::of::<T>(),
            data: value,
        }
    }
}


impl<T> Object<T> {
    fn erase(value: Gc<Object<T>>) -> Gc<ErasedObject> {
        unsafe {
            Gc::from_bits(value.to_bits())
        }
    }

    pub fn is_same_type(a: &Self, b: &Self) -> bool {
        a.tag == b.tag
    }
}

impl ErasedObject {
    pub fn is_same_type(a: &Self, b: &Self) -> bool {
        a.tag == b.tag
    }

    pub fn is<T>(&self) -> bool where T: 'static {
        self.tag == TypeId::of::<T>()
    }

    pub fn cast<T>(&self) -> &Object<T> where T: 'static {
        debug_assert_eq!(self.tag, TypeId::of::<T>());

        unsafe {
            std::mem::transmute(self)
        }
    }

    pub fn try_cast<T>(&self) -> Option<&Object<T>> where T: 'static {
        if self.is::<T>() {
            Some(self.cast::<T>())
        } else {
            None
        }
    }
}

impl PartialEq for ErasedObject {
    fn eq(&self, other: &Self) -> bool {
        if self.is::<String>() && other.is::<String>() {
            self.cast::<String>().as_str() == other.cast::<String>().as_str()
        } else {
            let a = self as *const ErasedObject;
            let b = other as *const ErasedObject;
            a == b
        }
    }
}

impl Display for ErasedObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is::<String>() {
            write!(f, "{}", self.cast::<String>().as_str())
        } else if self.is::<Closure>() {
            write!(f, "<fn {}>", self.cast::<Closure>().function.name)
        } else if self.is::<BoundMethod>() {
            write!(f, "<bound {}>", self.cast::<BoundMethod>().method)
        } else if self.is::<NativeFunction>() {
            write!(f, "<native fn>")
        } else if self.is::<Class>() {
            write!(f, "{}", self.cast::<Class>().name)
        } else if self.is::<Instance>() {
            write!(f, "{} instance", self.cast::<Instance>().class.name)
        } else if self.is::<Import>() {
            write!(f, "<import {}>", self.cast:: <Import>().name)
        } else if self.is::<List>() {
            write!(f, "{}", self.cast::<List>())
        } else {
            write!(f, "<unknown>")
        }
    }
}

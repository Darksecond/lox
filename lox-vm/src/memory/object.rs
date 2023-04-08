use std::ops::Deref;
use crate::gc::{Gc, Trace};
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

impl<T: Trace> Trace for Object<T> {
    fn trace(&self) {
        self.data.trace();
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


impl<T> From<Gc<Object<T>>> for Gc<ErasedObject> where T: Trace {
    fn from(value: Gc<Object<T>>) -> Self {
        unsafe {
            Gc::from_bits(value.to_bits())
        }
    }
}

impl Gc<ErasedObject> {
    pub fn cast<T>(self) -> Gc<Object<T>> where T: 'static {
        debug_assert_eq!(self.tag, TypeId::of::<T>());

        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn try_cast<T>(self) -> Option<Gc<Object<T>>> where T: 'static {
        if self.is::<T>() {
            Some(self.cast::<T>())
        } else {
            None
        }
    }

    pub fn is<T>(self) -> bool where T: 'static {
        self.tag == TypeId::of::<T>()
    }
}

impl<T> Object<T> {
    pub fn is_same_type(a: &Self, b: &Self) -> bool {
        a.tag == b.tag
    }
}

impl ErasedObject {
    pub fn is_same_type(a: &Self, b: &Self) -> bool {
        a.tag == b.tag
    }
}

impl PartialEq for Gc<ErasedObject> {
    fn eq(&self, other: &Self) -> bool {
        if self.is::<String>() && other.is::<String>() {
            self.cast::<String>().as_str() == other.cast::<String>().as_str()
        } else {
            Gc::ptr_eq(self, other)
        }
    }
}

impl Display for Gc<ErasedObject> {
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
            write!(f, "<import {}>", self.cast::<Import>().name)
        } else if self.is::<List>() {
            write!(f, "{}", self.cast::<List>())
        } else {
            write!(f, "<unknown>")
        }
    }
}

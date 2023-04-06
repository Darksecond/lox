use std::ops::Deref;
use crate::gc::{Gc, Trace};
use super::{Closure, BoundMethod, NativeFunction, Class, Instance, Import, List};
use std::fmt::Display;

trait ObjectInner: Display {}

impl<T> ObjectInner for T where T: Display {}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ObjectTag {
    String,
    Closure,
    BoundMethod,
    NativeFunction,
    Class,
    Instance,
    Import,
    List,
}

#[derive(Debug)]
#[repr(C)]
pub struct ErasedObject {
    pub tag: ObjectTag,
}

#[derive(Debug)]
#[repr(C)]
pub struct Object<T> {
    pub tag: ObjectTag,
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

impl Object<NativeFunction> {
    pub fn from_native_function(data: NativeFunction) -> Object<NativeFunction> {
        Self {
            tag: ObjectTag::NativeFunction,
            data,
        }
    }
}

impl From<Closure> for Object<Closure> {
    fn from(value: Closure) -> Self {
        Self {
            tag: ObjectTag::Closure,
            data: value,
        }
    }
}

impl From<List> for Object<List> {
    fn from(value: List) -> Self {
        Self {
            tag: ObjectTag::List,
            data: value,
        }
    }
}

impl From<Instance> for Object<Instance> {
    fn from(value: Instance) -> Self {
        Self {
            tag: ObjectTag::Instance,
            data: value,
        }
    }
}

impl From<Class> for Object<Class> {
    fn from(value: Class) -> Self {
        Self {
            tag: ObjectTag::Class,
            data: value,
        }
    }
}

impl From<String> for Object<String> {
    fn from(value: String) -> Self {
        Self {
            tag: ObjectTag::String,
            data: value,
        }
    }
}

impl From<Import> for Object<Import> {
    fn from(value: Import) -> Self {
        Self {
            tag: ObjectTag::Import,
            data: value,
        }
    }
}

impl From<BoundMethod> for Object<BoundMethod> {
    fn from(value: BoundMethod) -> Self {
        Self {
            tag: ObjectTag::BoundMethod,
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
    pub fn as_string(self) -> Gc<Object<String>> {
        debug_assert_eq!(self.tag, ObjectTag::String);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_closure(self) -> Gc<Object<Closure>> {
        debug_assert_eq!(self.tag, ObjectTag::Closure);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_list(self) -> Gc<Object<List>> {
        debug_assert_eq!(self.tag, ObjectTag::List);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_bound_method(self) -> Gc<Object<BoundMethod>> {
        debug_assert_eq!(self.tag, ObjectTag::BoundMethod);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_native_function(self) -> Gc<Object<NativeFunction>> {
        debug_assert_eq!(self.tag, ObjectTag::NativeFunction);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_class(self) -> Gc<Object<Class>> {
        debug_assert_eq!(self.tag, ObjectTag::Class);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_instance(self) -> Gc<Object<Instance>> {
        debug_assert_eq!(self.tag, ObjectTag::Instance);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
    }

    pub fn as_import(self) -> Gc<Object<Import>> {
        debug_assert_eq!(self.tag, ObjectTag::Import);
        unsafe {
            Gc::from_bits(self.to_bits())
        }
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
        if self.tag == ObjectTag::String && other.tag == ObjectTag::String {
            self.as_string().as_str() == other.as_string().as_str()
        } else {
            Gc::ptr_eq(self, other)
        }
    }
}

impl Display for Gc<ErasedObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            ObjectTag::String => write!(f, "{}", self.as_string().as_str()),
            ObjectTag::Closure => write!(f, "<fn {}>", self.as_closure().function.name),
            ObjectTag::BoundMethod => write!(f, "<bound {}>", self.as_bound_method().method),
            ObjectTag::NativeFunction => write!(f, "<native fn>"),
            ObjectTag::Class => write!(f, "{}", self.as_class().name),
            ObjectTag::Instance => write!(f, "{} instance", self.as_instance().class.name),
            ObjectTag::Import => write!(f, "<import>"),
            ObjectTag::List => write!(f, "{}", self.as_list()),
        }
    }
}

mod vtable {
    use super::ObjectInner;

    #[repr(C)]
    struct Object {
        data: *const (),
        vtable: *mut (),
    }

    fn extract<T: ObjectInner>(data: &T) -> *mut () {
        unsafe {
            let obj = data as &dyn ObjectInner;
            std::mem::transmute::<&dyn ObjectInner, Object>(obj).vtable
        }
    }

    unsafe fn construct<'a>(data: *const (), vtable: *mut ()) -> &'a dyn ObjectInner {
        unsafe {
            let object = Object {
                data,
                vtable,
            };
            std::mem::transmute::<Object, &dyn ObjectInner>(object)
        }
    }

    unsafe fn construct_mut<'a>(data: *const (), vtable: *mut ()) -> &'a mut dyn ObjectInner {
        unsafe {
            let object = Object {
                data,
                vtable,
            };
            std::mem::transmute::<Object, &mut dyn ObjectInner>(object)
        }
    }
}

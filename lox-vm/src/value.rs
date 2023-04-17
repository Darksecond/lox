use lox_gc::{Gc, Trace, Tracer};
use std::fmt::Display;

const QNAN: u64 = 0x7ffc000000000000;
const SIGN_BIT: u64 = 0x8000000000000000;
const TAG_NIL: u64 = 1;
const TAG_FALSE: u64 = 2;
const TAG_TRUE: u64 = 3;

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Value(u64);

impl Value {
    pub const NIL: Self = Self(QNAN | TAG_NIL);
    pub const FALSE: Self = Self(QNAN | TAG_FALSE);
    pub const TRUE: Self = Self(QNAN | TAG_TRUE);

    #[inline]
    pub fn from_object<T>(value: Gc<T>) -> Self {
        let bits = value.to_bits();
        Self(SIGN_BIT | QNAN | bits)
    }

    #[inline]
    pub fn is_object(self) -> bool {
        self.0 & (SIGN_BIT | QNAN) == (SIGN_BIT | QNAN)
    }

    #[inline]
    pub fn as_object(self) -> Gc<()> {
        unsafe {
            let bits = self.0 & (!(SIGN_BIT | QNAN));
            Gc::from_bits(bits)
        }
    }

    #[inline]
    pub fn is_number(self) -> bool {
        self.0 & QNAN != QNAN
    }

    #[inline]
    pub fn is_bool(self) -> bool {
        self.0 == Self::TRUE.0 || self.0 == Self::FALSE.0
    }

    #[inline]
    pub fn is_nil(self) -> bool {
        self.0 == Self::NIL.0
    }

    #[inline]
    pub fn as_number(self) -> f64 {
        f64::from_bits(self.0)
    }

    #[inline]
    pub fn to_bits(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn is_falsey(self) -> bool {
        if self.0 == Self::FALSE.0 {
            true
        } else if self.0 == Self::NIL.0 {
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn is_same_type(a: Self, b: Self) -> bool {
        if a.is_number() && b.is_number() {
            true
        } else if a.is_nil() && b.is_nil() {
            true
        } else if a.is_bool() && b.is_bool() {
            true
        } else if a.is_object() && b.is_object() {
            Gc::is_same_type(&a.as_object(), &b.as_object())
        } else {
            false
        }
    }

    #[inline]
    pub fn is_object_of_type<T>(self) -> bool where T: 'static {
        self.is_object() && self.as_object().is::<T>()
    }

    #[inline]
    pub fn try_cast<T>(self) -> Option<Gc<T>> where T: 'static {
        if self.is_object() && self.as_object().is::<T>() {
            Some(self.as_object().cast::<T>())
        } else {
            None
        }
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value.to_bits())
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }
}

impl PartialEq for Value {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        use crate::string::LoxString;

        if self.is_number() && other.is_number() {
            self.as_number() == other.as_number()
        } else if self.is_object_of_type::<LoxString>() && other.is_object_of_type::<LoxString>() {
            &*self.as_object().cast::<LoxString>() == &*other.as_object().cast::<LoxString>()
        } else if self.is_object() && other.is_object() {
            self.as_object() == other.as_object()
        } else {
            self.0 == other.0
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == Self::NIL.0 {
            write!(f, "nil")
        } else if self.0 == Self::TRUE.0 {
            write!(f, "true")
        } else if self.0 == Self::FALSE.0 {
            write!(f, "false")
        } else if self.is_object() {
            crate::memory::print(self.as_object(), f)
        } else {
            write!(f, "{}", self.as_number())
        }
    }
}

unsafe impl Trace for Value {
    fn trace(&self, tracer: &mut Tracer) {
        if self.is_object() {
            self.as_object().trace(tracer)
        }
    }
}

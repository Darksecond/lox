use crate::gc::{Gc, Trace};
use crate::memory::ErasedObject;
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

    pub fn from_object(value: impl Into<Gc<ErasedObject>>) -> Self {
        let bits = value.into().to_bits();
            println!("F {:08x}", bits);
        Self(SIGN_BIT | QNAN | bits)
    }

    pub fn is_object(self) -> bool {
        self.0 & (SIGN_BIT | QNAN) == (SIGN_BIT | QNAN)
    }

    pub fn as_object(self) -> Gc<ErasedObject> {
        unsafe {
            let bits = self.0 & (!(SIGN_BIT | QNAN));
            println!("T {:08x}", bits);
            Gc::from_bits(bits)
        }
    }

    pub fn is_number(self) -> bool {
        self.0 & QNAN != QNAN
    }

    pub fn as_number(self) -> f64 {
        f64::from_bits(self.0)
    }

    pub fn to_bits(self) -> u64 {
        self.0
    }

    pub const fn is_falsey(self) -> bool {
        if self.0 == Self::FALSE.0 {
            true
        } else if self.0 == Self::NIL.0 {
            true
        } else {
            false
        }
    }

    pub fn is_same_type(a: &Self, b: &Self) -> bool {
        if a.is_number() && b.is_number() {
            true
        } else if a.is_object() && b.is_object() {
            ErasedObject::is_same_type(&a.as_object(), &b.as_object())
        } else {
            false
        }
    }

    pub fn is_import(&self) -> bool {
        use crate::memory::ObjectTag;
        self.is_object() && self.as_object().tag == ObjectTag::Import
    }

    pub fn is_instance(&self) -> bool {
        use crate::memory::ObjectTag;
        self.is_object() && self.as_object().tag == ObjectTag::Instance
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self(value.to_bits())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        if value {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.is_number() && other.is_number() {
            self.as_number() == other.as_number()
        } else {
            self.0 == other.0
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::memory::ObjectTag;
        if self.0 == Self::NIL.0 {
            write!(f, "nil")
        } else if self.0 == Self::TRUE.0 {
            write!(f, "true")
        } else if self.0 == Self::FALSE.0 {
            write!(f, "false")
        } else if self.is_object() {
            let obj = self.as_object();
            if obj.tag == ObjectTag::String {
                write!(f, "{}", obj.as_string().as_str())
            } else {
                write!(f, "object {:?}", obj.tag) //TODO proper display
            }
        } else {
            write!(f, "{}", self.as_number())
        }
    }
}

impl Trace for Value {
    fn trace(&self) {
        if self.is_object() {
            self.as_object().trace()
        }
    }
}

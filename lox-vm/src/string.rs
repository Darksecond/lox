use crate::array::Array;
use std::ops::{Deref, DerefMut};
use std::str;
use std::borrow::{Borrow, BorrowMut};
use std::fmt;
use std::hash::Hash;
use lox_gc::{Trace, Tracer};

pub struct LoxString {
    vec: Array<u8>,
}

impl LoxString {
    pub fn new() -> Self {
        Self {
            vec: Array::new(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.vec
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Array::with_capacity(capacity),
        }
    }

    pub fn as_str(&self) -> &str {
        self
    }

    pub fn push_str(&mut self, string: &str) {
        self.vec.extend_from_slice(string.as_bytes());
    }
}

unsafe impl Trace for LoxString {
    fn trace(&self, _tracer: &mut Tracer) {
        self.vec.mark();
    }
}

impl PartialEq<LoxString> for LoxString {
    fn eq(&self, other: &LoxString) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }

    fn ne(&self, other: &LoxString) -> bool {
        PartialEq::ne(&self[..], &other[..])
    }
}

impl From<String> for LoxString {
    fn from(value: String) -> Self {
        let mut str = LoxString::with_capacity(value.len());
        str.push_str(&value);
        str
    }
}

impl From<&String> for LoxString {
    fn from(value: &String) -> Self {
        let mut str = LoxString::with_capacity(value.len());
        str.push_str(value);
        str
    }
}

impl From<&str> for LoxString {
    fn from(value: &str) -> Self {
        let mut str = LoxString::with_capacity(value.len());
        str.push_str(value);
        str
    }
}

impl Clone for LoxString {
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
        }
    }
}

impl fmt::Debug for LoxString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl fmt::Display for LoxString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl Eq for LoxString {}

impl Hash for LoxString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&**self).hash(state)
    }
}

impl Default for LoxString {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for LoxString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { str::from_utf8_unchecked(&self.vec) }
    }
}

impl DerefMut for LoxString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { str::from_utf8_unchecked_mut(&mut self.vec) }
    }
}

impl AsRef<str> for LoxString {
    fn as_ref(&self) -> &str {
        self
    }
}

impl Borrow<str> for LoxString {
    fn borrow(&self) -> &str {
        &self[..]
    }
}

impl BorrowMut<str> for LoxString {
    fn borrow_mut(&mut self) -> &mut str {
        &mut self[..]
    }
}

impl AsRef<[u8]> for LoxString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

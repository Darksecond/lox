use crate::value::Value;
use lox_gc::{Trace, Tracer};

#[derive(Copy, Clone)]
pub enum Upvalue {
    Open(usize),
    Closed(Value),
}

impl Upvalue {
    pub const fn is_open_with_range(&self, index: usize) -> Option<usize> {
        match self {
            Self::Open(i) => {
                if *i >= index {
                    Some(*i)
                } else {
                    None
                }
            }
            Self::Closed(_) => None,
        }
    }

    pub const fn is_open_with_index(&self, index: usize) -> bool {
        match self {
            Self::Open(i) => {
                *i == index
            }
            Self::Closed(_) => false,
        }
    }

    pub const fn is_open(&self) -> bool {
        match self {
            Self::Open(..) => true,
            Self::Closed(_) => false,
        }
    }
}

unsafe impl Trace for Upvalue {
    #[inline]
    fn trace(&self, tracer: &mut Tracer) {
        match self {
            Upvalue::Closed(value) => value.trace(tracer),
            Upvalue::Open(_) => (),
        }
    }
}

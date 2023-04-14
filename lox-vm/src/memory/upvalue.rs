use crate::value::Value;
use crate::gc::{Trace, Tracer};

#[derive(Debug, Copy, Clone)]
pub enum Upvalue {
    Open(usize), //TODO Track fiber (for closures over fibers)
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
            Self::Open(_) => true,
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

use crate::value::Value;
use crate::gc::{Trace, Tracer};

pub struct NativeFunction {
    pub name: String,
    pub code: fn(Value, &[Value]) -> Value,
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native function {}>", self.name)
    }
}

unsafe impl Trace for NativeFunction {
    #[inline]
    fn trace(&self, _tracer: &mut Tracer) {}
}

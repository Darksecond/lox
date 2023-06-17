use crate::value::Value;
use lox_gc::{Trace, Tracer};
use crate::string::LoxString;
use crate::Native;

pub struct NativeFunction {
    pub name: LoxString,
    pub code: fn(Native, Value, &[Value]) -> Value,
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native function {}>", self.name)
    }
}

unsafe impl Trace for NativeFunction {
    fn trace(&self, tracer: &mut Tracer) {
        self.name.trace(tracer);
    }
}

use crate::value::Value;
use lox_gc::{Gc, Trace, Tracer};

#[derive(Copy, Clone)]
pub struct BoundMethod {
    pub receiver: Gc<()>,
    pub method: Value,
}

unsafe impl Trace for BoundMethod {
    #[inline]
    fn trace(&self, tracer: &mut Tracer) {
        self.receiver.trace(tracer);
        self.method.trace(tracer);
    }
}

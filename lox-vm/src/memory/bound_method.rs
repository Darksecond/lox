use crate::value::Value;
use crate::gc::{Gc, Trace};
use crate::memory::ErasedObject;

#[derive(Debug, Copy, Clone)]
pub struct BoundMethod {
    pub receiver: Gc<ErasedObject>,
    pub method: Value,
}

impl Trace for BoundMethod {
    #[inline]
    fn trace(&self) {
        self.receiver.trace();
        self.method.trace();
    }
}

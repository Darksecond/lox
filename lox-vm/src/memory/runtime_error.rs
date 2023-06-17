use lox_gc::{Trace, Gc};
use crate::{string::LoxString, interner::Symbol};

pub struct RuntimeError {
    error: LoxString,
}

impl RuntimeError {
    pub fn new(error: impl AsRef<str>) -> Gc<Self> {
        lox_gc::manage(Self {
            error: error.as_ref().into(),
        })
    }

    pub fn undefined_variable(identifier: Symbol) -> Gc<Self> {
        Self::new(format!("Undefined variable '{}'.", identifier.to_string()))
    }

    pub fn undefined_property(identifier: Symbol) -> Gc<Self> {
        Self::new(format!("Undefined property '{}'.", identifier.to_string()))
    }

    pub fn invalid_call() -> Gc<Self> {
        Self::new("Can only call functions and classes.")
    }

    pub fn mismatch_arity(expected: usize, actual: usize) -> Gc<Self> {
        Self::new(format!("Expected {} arguments but got {}.", expected, actual))
    }

    pub fn only_instances_have_properties() -> Gc<Self> {
        Self::new("Only instances have properties.")
    }

    pub fn only_instances_have_fields() -> Gc<Self> {
        Self::new("Only instances have fields.")
    }

    pub fn stack_overflow() -> Gc<Self> {
        Self::new("Stack overflow.")
    }

    pub fn invalid_binary_operands() -> Gc<Self> {
        Self::new("Operands must be two numbers or two strings.")
    }

    pub fn invalid_operand_types() -> Gc<Self> {
        Self::new("Operands must be numbers.")
    }
}

unsafe impl Trace for RuntimeError {
    fn trace(&self, tracer: &mut lox_gc::Tracer) {
        self.error.trace(tracer);
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

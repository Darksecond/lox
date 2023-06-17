mod import;
mod list;
mod closure;
mod upvalue;
mod instance;
mod class;
mod native_function;
mod bound_method;
mod runtime_error;

pub use import::*;
pub use list::*;
pub use closure::*;
pub use upvalue::*;
pub use instance::*;
pub use class::*;
pub use native_function::*;
pub use bound_method::*;
pub use runtime_error::*;

use crate::string::LoxString;
use lox_gc::Gc;
use std::fmt;

pub fn print(value: Gc<()>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if value.is::<String>() {
        write!(f, "{}", value.cast::<String>().as_str())
    } else if value.is::<LoxString>() {
        write!(f, "{}", value.cast::<LoxString>().as_str())
    } else if value.is::<Closure>() {
        write!(f, "<fn {}>", value.cast::<Closure>().function.name)
    } else if value.is::<BoundMethod>() {
        write!(f, "<bound {}>", value.cast::<BoundMethod>().method)
    } else if value.is::<NativeFunction>() {
        write!(f, "<native fn>")
    } else if value.is::<Class>() {
        write!(f, "{}", value.cast::<Class>().name)
    } else if value.is::<Instance>() {
        write!(f, "{} instance", value.cast::<Instance>().class.name)
    } else if value.is::<Import>() {
        write!(f, "<import {}>", value.cast:: <Import>().name)
    } else if value.is::<List>() {
        write!(f, "{}", value.cast::<List>())
    } else if value.is::<RuntimeError>() {
        todo!()
    } else {
        write!(f, "<unknown>")
    }
}

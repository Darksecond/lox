mod import;
mod list;
mod closure;
mod upvalue;
mod instance;
mod class;
mod native_function;
mod bound_method;

pub use import::*;
pub use list::*;
pub use closure::*;
pub use upvalue::*;
pub use instance::*;
pub use class::*;
pub use native_function::*;
pub use bound_method::*;

pub fn print(value: crate::gc::Gc<()>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if value.is::<String>() {
        write!(f, "{}", value.cast::<String>().as_str())
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
    } else {
        write!(f, "<unknown>")
    }
}

pub mod tokenizer;
#[macro_use]
mod common;
pub mod ast;
pub mod bytecode;
// pub mod compiler;
mod expr_parser;
pub mod stmt_parser;
mod token;
pub mod vm;
// pub mod gc;
mod bettergc;
pub mod bettercompiler;

//TODO Better errors

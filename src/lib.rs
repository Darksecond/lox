pub mod tokenizer;
#[macro_use]
mod common;
pub mod ast;
pub mod bytecode;
mod expr_parser;
pub mod stmt_parser;
mod token;
pub mod vm;
mod bettergc;
pub mod bettercompiler;
pub mod bettervm;

//TODO Better errors

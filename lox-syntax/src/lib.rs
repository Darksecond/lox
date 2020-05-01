pub mod ast;
pub mod position;

#[macro_use]
mod common;
mod expr_parser;
mod parser;
mod stmt_parser;
mod token;
mod tokenizer;

use ast::Ast;
pub use common::ParseError;

#[derive(PartialEq, Debug, Clone)]
pub enum SyntaxError {}

pub fn parse(code: &str) -> Result<Ast, ParseError> {
  use stmt_parser::parse;
  use tokenizer::tokenize_with_context;
  let tokens = tokenize_with_context(code);
  let mut it = tokens.as_slice().into_iter().peekable();
  parse(&mut it)
}

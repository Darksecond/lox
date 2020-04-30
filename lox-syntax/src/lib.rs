pub mod ast;
#[macro_use]
pub mod common;
mod expr_parser;
pub mod position;
pub mod stmt_parser;
mod token;
pub mod tokenizer;

#[derive(PartialEq, Debug, Clone)]
pub enum SyntaxError {
  UnterminatedString,
  InvalidCharacter(char),
}

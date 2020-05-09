pub mod ast;
pub mod position;

#[macro_use]
mod parser;
mod common;
mod expr_parser;
mod stmt_parser;
mod token;
mod tokenizer;


use ast::{Ast, Expr};
use token::{Token, TokenKind};
use position::WithSpan;

#[derive(PartialEq, Debug, Clone)]
pub enum SyntaxError {
  Expected(TokenKind, WithSpan<Token>),
  Unexpected(WithSpan<Token>),
  ExpectedUnaryOperator(WithSpan<Token>),
  ExpectedBinaryOperator(WithSpan<Token>),
  ExpectedPrimary(WithSpan<Token>),
  InvalidLeftValue(WithSpan<Expr>),
}

pub fn parse(code: &str) -> Result<Ast, SyntaxError> {
  use stmt_parser::parse;
  use tokenizer::tokenize_with_context;
  let tokens = tokenize_with_context(code);
  let mut parser = crate::parser::Parser::new(&tokens);
  parse(&mut parser)
}

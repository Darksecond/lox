use crate::{
  SyntaxError,
  token::Token,
  position::WithSpan,
};

pub struct Parser {
  errors: Vec<SyntaxError>,
  tokens: Vec<WithSpan<Token>>,
  //iterator of some kind
}
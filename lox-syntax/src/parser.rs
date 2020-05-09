use crate::position::{WithSpan};
use crate::token::Token;
use std::iter::{Iterator, Peekable};
use crate::ParseError;

pub struct Parser<'a, It>
where
  It: Iterator<Item = &'a WithSpan<Token>>,
{
  iterator: Peekable<It>,
}

impl<'a, It> Parser<'a, It>
where
  It: Iterator<Item = &'a WithSpan<Token>>,
{
  pub fn new(iterator: It) -> Self {
    Parser {
      iterator: iterator.peekable(),
    }
  }

  pub fn raw_peek(&mut self) -> Option<&&'a WithSpan<Token>> {
    self.iterator.peek()
  }

  pub fn raw_next(&mut self) -> Option<&'a WithSpan<Token>> {
    self.iterator.next()
  }

  pub fn peek(&mut self) -> Result<&'a Token, String> {
    match self.iterator.peek() {
      Some(&t) => Ok(&t.value),
      None => Err(String::from("No more tokens")),
    }
  }

  pub fn error<S: AsRef<str>>(&mut self, error: S) -> ParseError {
    if let Some(token) = self.iterator.peek() {
      ParseError { error: error.as_ref().to_string(), span: Some(token.span) }
    } else {
      "No more tokens".into()
    }
  }

  pub fn next(&mut self) -> Result<&'a WithSpan<Token>, String> {
    match self.iterator.next() {
      Some(t) => Ok(t),
      None => Err(String::from("No more tokens")),
    }
  }

  pub fn expect(&mut self, expected: &Token) -> Result<&'a Token, ParseError> {
      let token = self.next()?;
      if &token.value == expected {
        Ok(&token.value)
      } else {
        Err(ParseError { error: format!("Expected {:?} got {:?}", expected, &token.value).into(), span: Some(token.span) })
      }
  }

  pub fn optionally(&mut self, expected: &Token) -> Result<bool, ParseError> {
    match self.iterator.peek() {
      Some(&token) => {
        if &token.value == expected {
          self.expect(expected)?;
          Ok(true)
        } else {
          Ok(false)
        }
      }
      None => Ok(false),
    }
  }
}

macro_rules! expect {
  ($x:ident, $y:pat) => {{
      let tc = $x.next()?;
      match &tc.value {
          $y => Ok(&t.token),
          t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), span: Some(tc.span) }),
      }
  }};
  ($x:ident, $y:pat => $z:expr) => {{
      let tc = $x.next()?;
      match &tc.value {
          $y => Ok($z),
          t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), span: Some(tc.span) }),
      }
  }};
}

macro_rules! expect_with_span {
  ($x:ident, $y:pat => $z:expr) => {{
      let tc = $x.next()?;
      match &tc.value {
          $y => Ok(WithSpan::new($z, tc.span)),
          _ => Err(ParseError { error: "Unexpected token".into(), span: Some(tc.span) }),
      }
  }};
}
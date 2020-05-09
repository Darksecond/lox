use crate::position::{WithSpan};
use crate::token::{Token, TokenKind};
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

  pub fn is_eof(&mut self) -> bool {
    self.check(TokenKind::Eof)
  }

  pub fn peek(&mut self) -> TokenKind {
    match self.iterator.peek() {
      Some(&t) => t.into(),
      None => TokenKind::Eof,
    }
  }

  pub fn check(&mut self, match_token: TokenKind) -> bool {
    let token = self.peek();
    token == match_token
  }

  pub fn error<S: AsRef<str>>(&mut self, error: S) -> ParseError {
    if let Some(token) = self.iterator.peek() {
      ParseError { error: error.as_ref().to_string(), span: Some(token.span) }
    } else {
      "No more tokens".into()
    }
  }

  pub fn advance(&mut self) -> Result<&'a WithSpan<Token>, String> {
    match self.iterator.next() {
      Some(t) => Ok(t),
      None => Err(String::from("No more tokens")),
    }
  }

  pub fn expect(&mut self, expected: TokenKind) -> Result<&'a Token, ParseError> {
      let token = self.advance()?;
      if TokenKind::from(token) == expected {
        Ok(&token.value)
      } else {
        Err(ParseError { error: format!("Expected {:?} got {:?}", expected, &token.value).into(), span: Some(token.span) })
      }
  }

  pub fn optionally(&mut self, expected: TokenKind) -> Result<bool, ParseError> {
    match self.iterator.peek() {
      Some(&token) => {
        if TokenKind::from(token) == expected {
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
      let tc = $x.advance()?;
      match &tc.value {
          $y => Ok(&t.token),
          t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), span: Some(tc.span) }),
      }
  }};
  ($x:ident, $y:pat => $z:expr) => {{
      let tc = $x.advance()?;
      match &tc.value {
          $y => Ok($z),
          t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), span: Some(tc.span) }),
      }
  }};
}

macro_rules! expect_with_span {
  ($x:ident, $y:pat => $z:expr) => {{
      let tc = $x.advance()?;
      match &tc.value {
          $y => Ok(WithSpan::new($z, tc.span)),
          _ => Err(ParseError { error: "Unexpected token".into(), span: Some(tc.span) }),
      }
  }};
}
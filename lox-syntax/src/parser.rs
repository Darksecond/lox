use crate::position::{WithSpan};
use crate::token::{Token, TokenKind};
use crate::ParseError;

static EOF_TOKEN: WithSpan<Token> = WithSpan::empty(Token::Eof);

pub struct Parser<'a> {
  tokens: &'a [WithSpan<Token>],
  cursor: usize,
}

impl<'a> Parser<'a> {
  pub fn new(tokens: &'a [WithSpan<Token>]) -> Self {
    Parser {
      tokens,
      cursor: 0,
    }
  }

  pub fn is_eof(&mut self) -> bool {
    self.check(TokenKind::Eof)
  }

  pub fn peek(&mut self) -> TokenKind {
    match self.tokens.get(self.cursor) {
      Some(t) => t.into(),
      None => TokenKind::Eof,
    }
  }

  pub fn check(&mut self, match_token: TokenKind) -> bool {
    let token = self.peek();
    token == match_token
  }

  pub fn error<S: AsRef<str>>(&mut self, error: S) -> ParseError {
    if !self.check(TokenKind::Eof) {
      let token = self.advance();
      ParseError { error: error.as_ref().to_string(), span: Some(token.span) }
    } else {
      "No more tokens".into()
    }
  }

  pub fn advance(&mut self) -> &'a WithSpan<Token> {
    let token = self.tokens.get(self.cursor);
    if let Some(token) = token {
      self.cursor = self.cursor + 1;
      token
    } else {
      &EOF_TOKEN
    }
  }

  pub fn expect(&mut self, expected: TokenKind) -> Result<&'a Token, ParseError> {
      let token = self.advance();
      if TokenKind::from(token) == expected {
        Ok(&token.value)
      } else {
        Err(ParseError { error: format!("Expected {:?} got {:?}", expected, &token.value).into(), span: Some(token.span) })
      }
  }

  pub fn optionally(&mut self, expected: TokenKind) -> Result<bool, ParseError> {
    let token = self.peek();
    if TokenKind::from(token) == expected {
      self.expect(expected)?;
      Ok(true)
    } else {
      Ok(false)
    }
  }
}

macro_rules! expect {
  ($x:ident, $y:pat) => {{
      let tc = $x.advance();
      match &tc.value {
          $y => Ok(&t.token),
          t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), span: Some(tc.span) }),
      }
  }};
  ($x:ident, $y:pat => $z:expr) => {{
      let tc = $x.advance();
      match &tc.value {
          $y => Ok($z),
          t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), span: Some(tc.span) }),
      }
  }};
}

macro_rules! expect_with_span {
  ($x:ident, $y:pat => $z:expr) => {{
      let tc = $x.advance();
      match &tc.value {
          $y => Ok(WithSpan::new($z, tc.span)),
          _ => Err(ParseError { error: "Unexpected token".into(), span: Some(tc.span) }),
      }
  }};
}
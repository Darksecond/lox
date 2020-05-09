use crate::position::{WithSpan};
use crate::token::{Token, TokenKind};
use crate::SyntaxError;

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
    self.peek_token().into()
  }

  pub fn peek_token(&mut self) -> &'a WithSpan<Token> {
    match self.tokens.get(self.cursor) {
      Some(t) => t,
      None => &EOF_TOKEN,
    }
  }

  pub fn check(&mut self, match_token: TokenKind) -> bool {
    let token = self.peek();
    token == match_token
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

  pub fn expect(&mut self, expected: TokenKind) -> Result<&'a Token, SyntaxError> {
      let token = self.advance();
      if TokenKind::from(token) == expected {
        Ok(&token.value)
      } else {
        Err(SyntaxError::Expected(expected, token.clone()))
      }
  }

  pub fn optionally(&mut self, expected: TokenKind) -> Result<bool, SyntaxError> {
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
          _ => Err(SyntaxError::Unexpected(tc.clone())),
      }
  }};
  ($x:ident, $y:pat => $z:expr) => {{
      let tc = $x.advance();
      match &tc.value {
          $y => Ok($z),
          _ => Err(SyntaxError::Unexpected(tc.clone())),
      }
  }};
}

macro_rules! expect_with_span {
  ($x:ident, $y:pat => $z:expr) => {{
      let tc = $x.advance();
      match &tc.value {
          $y => Ok(WithSpan::new($z, tc.span)),
          _ => Err(SyntaxError::Unexpected(tc.clone())),
      }
  }};
}
use crate::position::{WithSpan, Diagnostic, Span};
use crate::token::{Token, TokenKind};

static EOF_TOKEN: WithSpan<Token> = WithSpan::empty(Token::Eof);

pub struct Parser<'a> {
    tokens: &'a [WithSpan<Token>],
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [WithSpan<Token>]) -> Self {
        Parser { tokens, cursor: 0, diagnostics: Vec::new() }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn error(&mut self, message: &str, span: Span) {
        self.diagnostics.push(Diagnostic {
            message: message.to_string(),
            span,
        });
    }

    pub fn is_eof(&self) -> bool {
        self.check(TokenKind::Eof)
    }

    pub fn peek(&self) -> TokenKind {
        self.peek_token().into()
    }

    pub fn peek_token(&self) -> &'a WithSpan<Token> {
        match self.tokens.get(self.cursor) {
            Some(t) => t,
            None => &EOF_TOKEN,
        }
    }

    pub fn check(&self, match_token: TokenKind) -> bool {
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

    pub fn expect(&mut self, expected: TokenKind) -> Result<&'a WithSpan<Token>, ()> {
        let token = self.advance();
        if TokenKind::from(token) == expected {
            Ok(token)
        } else {
            self.error(&format!("Expected {} got {}", expected, token.value), token.span);
            Err(())
        }
    }

    pub fn optionally(&mut self, expected: TokenKind) -> Result<bool, ()> {
        let token = self.peek();
        if TokenKind::from(token) == expected {
            self.expect(expected)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

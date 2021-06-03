use crate::ast::Identifier;
use crate::parser::Parser;
use crate::position::WithSpan;
use crate::token::{Token, TokenKind};
use crate::SyntaxError;

pub fn expect_identifier(p: &mut Parser) -> Result<WithSpan<Identifier>, SyntaxError> {
    let token = p.advance();
    match &token.value {
        Token::Identifier(ident) => Ok(WithSpan::new(ident.clone(), token.span)),
        _ => Err(SyntaxError::Expected(TokenKind::Identifier, token.clone())),
    }
}

pub fn expect_string(p: &mut Parser) -> Result<WithSpan<String>, SyntaxError> {
    let token = p.advance();
    match &token.value {
        Token::String(ident) => Ok(WithSpan::new(ident.clone(), token.span)),
        _ => Err(SyntaxError::Expected(TokenKind::String, token.clone())),
    }
}
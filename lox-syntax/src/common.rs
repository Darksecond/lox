use crate::parser::Parser;
use crate::SyntaxError;
use crate::token::{Token, TokenKind};
use crate::position::WithSpan;
use crate::ast::Identifier;

pub fn expect_identifier(p: &mut Parser) -> Result<WithSpan<Identifier>, SyntaxError> {
    let token = p.advance();
    match &token.value {
        Token::Identifier(ident) => Ok(WithSpan::new(ident.clone(), token.span)),
        _ => Err(SyntaxError::Expected(TokenKind::Identifier, token.clone())),
    }
}
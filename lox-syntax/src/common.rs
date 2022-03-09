use crate::ast::Identifier;
use crate::parser::Parser;
use crate::position::WithSpan;
use crate::token::{Token, TokenKind};

pub fn expect_identifier(p: &mut Parser) -> Result<WithSpan<Identifier>, ()> {
    let token = p.advance();
    match &token.value {
        Token::Identifier(ident) => Ok(WithSpan::new(ident.clone(), token.span)),
        _ => {
            p.error(&format!("Expected {} got {}", TokenKind::Identifier, token.value), token.span);
            Err(())
        },
    }
}

pub fn expect_string(p: &mut Parser) -> Result<WithSpan<String>, ()> {
    let token = p.advance();
    match &token.value {
        Token::String(ident) => Ok(WithSpan::new(ident.clone(), token.span)),
        _ => {
            p.error(&format!("Expected {} got {}", TokenKind::String, token.value), token.span);
            Err(())
        },
    }
}
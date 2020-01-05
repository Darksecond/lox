use super::token::Token;
use std::iter::{Iterator, Peekable};
use crate::position::{Span, WithSpan};

//TODO Replace with WithSpan<String>
#[derive(Debug)]
pub struct ParseError {
    pub error: String,
    pub span: Option<Span>,
}

impl From<&str> for ParseError {
    fn from(error: &str) -> ParseError {
        ParseError {
            error: error.to_string(),
            span: None,
        }
    }
}
impl From<&String> for ParseError {
    fn from(error: &String) -> ParseError {
        ParseError {
            error: error.to_string(),
            span: None,
        }
    }
}
impl From<String> for ParseError {
    fn from(error: String) -> ParseError {
        ParseError {
            error: error,
            span: None,
        }
    }
}

pub fn error<'a, It, S: AsRef<str>>(it: &mut Peekable<It>, error: S) -> ParseError
where
    It: Iterator<Item = &'a WithSpan<Token>>,
{
    if let Some(token) = it.peek() {
        ParseError { error: error.as_ref().to_string(), span: Some(token.span) }
    } else {
        "No more tokens".into()
    }
}

pub fn peek<'a, It>(it: &mut Peekable<It>) -> Result<&'a Token, String>
where
    It: Iterator<Item = &'a WithSpan<Token>>,
{
    match it.peek() {
        Some(&t) => Ok(&t.value),
        None => Err(String::from("No more tokens")),
    }
}

pub fn next_with_context<'a, It>(it: &mut Peekable<It>) -> Result<&'a WithSpan<Token>, String>
where
    It: Iterator<Item = &'a WithSpan<Token>>,
{
    match it.next() {
        Some(t) => Ok(t),
        None => Err(String::from("No more tokens")),
    }
}

pub fn expect<'a, It>(it: &mut Peekable<It>, expected: &Token) -> Result<&'a Token, ParseError>
where
    It: Iterator<Item = &'a WithSpan<Token>>,
{
    let token = next_with_context(it)?;
    if &token.value == expected {
        Ok(&token.value)
    } else {
        Err(ParseError { error: format!("Expected {:?} got {:?}", expected, &token.value).into(), span: Some(token.span) })
    }
}

pub fn optionally<'a, It>(it: &mut Peekable<It>, expected: &Token) -> Result<bool, ParseError>
where
    It: Iterator<Item = &'a WithSpan<Token>>,
{
    match it.peek() {
        Some(&token) => {
            if &token.value == expected {
                expect(it, expected)?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
        None => Ok(false),
    }
}

macro_rules! expect {
    ($x:expr, $y:pat) => {{
        let tc = next_with_context($x)?;
        match &tc.value {
            $y => Ok(&t.token),
            t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), span: Some(tc.span) }),
        }
    }};
    ($x:expr, $y:pat => $z:expr) => {{
        let tc = next_with_context($x)?;
        match &tc.value {
            $y => Ok($z),
            t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), span: Some(tc.span) }),
        }
    }};
}

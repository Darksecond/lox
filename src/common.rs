use super::token::Token;
use super::tokenizer::TokenWithContext;
use std::iter::{Iterator, Peekable};
use super::tokenizer::Position;

#[derive(Debug)]
pub struct ParseError {
    pub error: String,
    pub position: Option<Position>,
}

impl From<&str> for ParseError {
    fn from(error: &str) -> ParseError {
        ParseError {
            error: error.to_string(),
            position: None,
        }
    }
}
impl From<&String> for ParseError {
    fn from(error: &String) -> ParseError {
        ParseError {
            error: error.to_string(),
            position: None,
        }
    }
}
impl From<String> for ParseError {
    fn from(error: String) -> ParseError {
        ParseError {
            error: error,
            position: None,
        }
    }
}

pub fn error<'a, It, S: AsRef<str>>(it: &mut Peekable<It>, error: S) -> ParseError
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    if let Some(token) = it.peek() {
        ParseError { error: error.as_ref().to_string(), position: Some(token.position) }
    } else {
        "No more tokens".into()
    }
}

pub fn peek<'a, It>(it: &mut Peekable<It>) -> Result<&'a Token, String>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match it.peek() {
        Some(&t) => Ok(&t.token),
        None => Err(String::from("No more tokens")),
    }
}

//TODO Merge with next_with_context
pub fn next<'a, It>(it: &mut Peekable<It>) -> Result<&'a Token, String>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match it.next() {
        Some(t) => Ok(&t.token),
        None => Err(String::from("No more tokens")),
    }
}

pub fn next_with_context<'a, It>(it: &mut Peekable<It>) -> Result<&'a TokenWithContext, String>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match it.next() {
        Some(t) => Ok(t),
        None => Err(String::from("No more tokens")),
    }
}

pub fn expect<'a, It>(it: &mut Peekable<It>, expected: &Token) -> Result<&'a Token, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    let token = next_with_context(it)?;
    if &token.token == expected {
        Ok(&token.token)
    } else {
        Err(ParseError { error: format!("Expected {:?} got {:?}", expected, &token.token).into(), position: Some(token.position) })
    }
}

pub fn optionally<'a, It>(it: &mut Peekable<It>, expected: &Token) -> Result<bool, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match it.peek() {
        Some(&token) => {
            if &token.token == expected {
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
        match &tc.token {
            $y => Ok(&t.token),
            t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), position: Some(tc.position) }),
        }
    }};
    ($x:expr, $y:pat => $z:expr) => {{
        let tc = next_with_context($x)?;
        match &tc.token {
            $y => Ok($z),
            t => Err(ParseError { error: format!("Unexpected {:?}", t).into(), position: Some(tc.position) }),
        }
    }};
}

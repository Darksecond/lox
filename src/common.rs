use super::token::Token;
use std::iter::{Iterator, Peekable};

pub fn peek<'a, It>(it: &mut Peekable<It>) -> Result<&'a Token, String>
where
    It: Iterator<Item = &'a Token>,
{
    match it.peek() {
        Some(&t) => Ok(t),
        None => Err(String::from("No more tokens")),
    }
}

pub fn next<'a, It>(it: &mut Peekable<It>) -> Result<&'a Token, String>
where
    It: Iterator<Item = &'a Token>,
{
    match it.next() {
        Some(t) => Ok(t),
        None => Err(String::from("No more tokens")),
    }
}

pub fn expect<'a, It>(it: &mut Peekable<It>, expected: &Token) -> Result<&'a Token, String>
where
    It: Iterator<Item = &'a Token>,
{
    let token = next(it)?;
    if token == expected {
        Ok(token)
    } else {
        Err(format!("Expected {:?} got {:?}", expected, token))
    }
}

pub fn optionally<'a, It>(it: &mut Peekable<It>, expected: &Token) -> Result<bool, String>
where
    It: Iterator<Item = &'a Token>,
{
    match it.peek() {
        Some(&token) => {
            if token == expected {
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
        let t = next($x)?;
        match t {
            $y => Ok(t),
            t => Err(format!("Unexpected {:?}", t)),
        }
    }};
    ($x:expr, $y:pat => $z:expr) => {{
        let t = next($x)?;
        match t {
            $y => Ok($z),
            t => Err(format!("Unexpected {:?}", t)),
        }
    }};
}

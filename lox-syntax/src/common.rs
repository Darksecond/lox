use crate::position::{Span};

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
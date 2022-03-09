pub mod ast;
pub mod position;

#[macro_use]
mod parser;
mod common;
mod expr_parser;
mod stmt_parser;
mod token;
mod tokenizer;

use ast::Ast;
use position::Diagnostic;

pub fn parse(code: &str) -> Result<Ast, Vec<Diagnostic>> {
    use stmt_parser::parse;
    use tokenizer::tokenize_with_context;
    let tokens = tokenize_with_context(code);
    let mut parser = crate::parser::Parser::new(&tokens);
    match parse(&mut parser) {
        Ok(ast) if parser.diagnostics().is_empty() => Ok(ast),
        Ok(_) => Err(parser.diagnostics().to_vec()),
        Err(_) => Err(parser.diagnostics().to_vec()),
    }
}

use super::ast::*;
use super::token::*;
use crate::common::*;
use crate::parser::Parser;
use crate::position::WithSpan;
use crate::SyntaxError;

fn parse_program(it: &mut Parser) -> Result<Vec<Stmt>, SyntaxError> {
    let mut statements = Vec::new();
    while !it.is_eof() {
        statements.push(parse_declaration(it)?);
    }

    Ok(statements)
}

fn parse_declaration(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    match it.peek() {
        TokenKind::Var => parse_var_declaration(it),
        TokenKind::Fun => parse_function_declaration(it),
        TokenKind::Class => parse_class_declaration(it),
        _ => parse_statement(it),
    }
}

fn parse_statement(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    match it.peek() {
        TokenKind::Print => parse_print_statement(it),
        TokenKind::If => parse_if_statement(it),
        TokenKind::LeftBrace => parse_block_statement(it),
        TokenKind::While => parse_while_statement(it),
        TokenKind::Return => parse_return_statement(it),
        TokenKind::For => parse_for_statement(it),
        _ => parse_expr_statement(it),
    }
}

fn parse_class_declaration(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::Class)?;
    let name = expect_identifier(it)?;
    let superclass = if it.optionally(TokenKind::Less)? {
        let name = expect_identifier(it)?;
        Some(name.clone())
    } else {
        None
    };
    it.expect(TokenKind::LeftBrace)?;
    let mut functions: Vec<Stmt> = vec![];
    while !it.check(TokenKind::RightBrace) {
        functions.push(parse_function(it)?);
    }
    it.expect(TokenKind::RightBrace)?;

    Ok(Stmt::Class(name.clone(), superclass, functions))
}

fn parse_function_declaration(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::Fun)?;
    parse_function(it)
}

fn parse_function(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    let name = expect_identifier(it)?;
    it.expect(TokenKind::LeftParen)?;
    let params = if !it.check(TokenKind::RightParen) {
        parse_params(it)?
    } else {
        Vec::new()
    };
    it.expect(TokenKind::RightParen)?;
    it.expect(TokenKind::LeftBrace)?;
    let mut body: Vec<Stmt> = Vec::new();
    while !it.check(TokenKind::RightBrace) {
        body.push(parse_declaration(it)?);
    }
    it.expect(TokenKind::RightBrace)?;
    Ok(Stmt::Function(name.clone(), params, body))
}

fn parse_params(it: &mut Parser) -> Result<Vec<WithSpan<Identifier>>, SyntaxError> {
    let mut params: Vec<WithSpan<Identifier>> = Vec::new();
    params.push(expect_identifier(it)?);
    while it.check(TokenKind::Comma) {
        it.expect(TokenKind::Comma)?;
        params.push(expect_identifier(it)?);
    }
    Ok(params)
}

fn parse_var_declaration(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::Var)?;
    let name = expect_identifier(it)?;
    let mut initializer = None;

    if it.optionally(TokenKind::Equal)? {
        initializer = Some(parse_expr(it)?);
    }

    it.expect(TokenKind::Semicolon)?;

    Ok(Stmt::Var(name, initializer.map(Box::new)))
}

fn parse_expr(it: &mut Parser) -> Result<WithSpan<Expr>, SyntaxError> {
    super::expr_parser::parse(it).map_err(|e| e.into())
}

fn parse_for_statement(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::For)?;
    it.expect(TokenKind::LeftParen)?;
    let initializer = match it.peek() {
        TokenKind::Var => Some(parse_var_declaration(it)?),
        TokenKind::Semicolon => {
            it.expect(TokenKind::Semicolon)?;
            None
        }
        _ => Some(parse_expr_statement(it)?),
    };
    let condition = if !it.check(TokenKind::Semicolon) {
        parse_expr(it)?
    } else {
        WithSpan::empty(Expr::Boolean(true))
    };
    it.expect(TokenKind::Semicolon)?;
    let increment = if !it.check(TokenKind::RightParen) {
        Some(parse_expr(it)?)
    } else {
        None
    };
    it.expect(TokenKind::RightParen)?;
    let body = parse_statement(it)?;
    // Add increment if it exists
    let body = match increment {
        Some(expr) => Stmt::Block(vec![body, Stmt::Expression(Box::new(expr))]),
        None => body,
    };
    let body = Stmt::While(Box::new(condition), Box::new(body));
    let body = match initializer {
        Some(stmt) => Stmt::Block(vec![stmt, body]),
        None => body,
    };

    Ok(body)
}

fn parse_return_statement(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::Return)?;
    let mut expr = None;
    if !it.check(TokenKind::Semicolon) {
        expr = Some(parse_expr(it)?);
    }
    it.expect(TokenKind::Semicolon)?;
    Ok(Stmt::Return(expr.map(Box::new)))
}

fn parse_expr_statement(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    let expr = parse_expr(it)?;
    it.expect(TokenKind::Semicolon)?;

    Ok(Stmt::Expression(Box::new(expr)))
}

fn parse_block_statement(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::LeftBrace)?;
    let mut statements: Vec<Stmt> = Vec::new();
    while !it.check(TokenKind::RightBrace) {
        statements.push(parse_declaration(it)?);
    }
    it.expect(TokenKind::RightBrace)?;
    Ok(Stmt::Block(statements))
}

fn parse_while_statement(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::While)?;
    it.expect(TokenKind::LeftParen)?;
    let condition = parse_expr(it)?;
    it.expect(TokenKind::RightParen)?;
    let statement = parse_statement(it)?;
    Ok(Stmt::While(Box::new(condition), Box::new(statement)))
}

fn parse_if_statement(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::If)?;
    it.expect(TokenKind::LeftParen)?;
    let condition = parse_expr(it)?;
    it.expect(TokenKind::RightParen)?;
    let if_stmt = parse_statement(it)?;
    let mut else_stmt: Option<Stmt> = None;

    if it.optionally(TokenKind::Else)? {
        else_stmt = Some(parse_statement(it)?);
    }

    Ok(Stmt::If(
        Box::new(condition),
        Box::new(if_stmt),
        else_stmt.map(Box::new),
    ))
}

fn parse_print_statement(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::Print)?;
    let expr = parse_expr(it)?;
    it.expect(TokenKind::Semicolon)?;
    Ok(Stmt::Print(Box::new(expr)))
}

pub fn parse(it: &mut Parser) -> Result<Vec<Stmt>, SyntaxError> {
    parse_program(it)
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::super::tokenizer::*;
    use super::*;
    fn parse_str(data: &str) -> Result<Vec<Stmt>, SyntaxError> {
        let tokens = tokenize_with_context(data);
        let mut parser = crate::parser::Parser::new(&tokens);
        parse(&mut parser)
    }
    pub fn ws<T>(value: T, range: Range<u32>) -> WithSpan<T> {
        unsafe { WithSpan::new_unchecked(value, range.start, range.end) }
    }

    #[test]
    fn test_expr_stmt() {
        assert_eq!(
            parse_str("nil;"),
            Ok(vec![Stmt::Expression(Box::new(ws(Expr::Nil, 0..3))),])
        );
        assert_eq!(
            parse_str("nil;nil;"),
            Ok(vec![
                Stmt::Expression(Box::new(ws(Expr::Nil, 0..3))),
                Stmt::Expression(Box::new(ws(Expr::Nil, 4..7))),
            ])
        );
    }

    #[test]
    fn test_print_stmt() {
        assert_eq!(
            parse_str("print nil;"),
            Ok(vec![Stmt::Print(Box::new(ws(Expr::Nil, 6..9))),])
        );
    }

    fn make_span_string(string: &str, offset: u32) -> WithSpan<String> {
        unsafe { WithSpan::new_unchecked(string.into(), offset, offset+string.len() as u32) }
    }

    #[test]
    fn test_var_decl() {
        assert_eq!(
            parse_str("var beverage;"),
            Ok(vec![Stmt::Var(make_span_string("beverage", 4), None),])
        );
        assert_eq!(
            parse_str("var beverage = nil;"),
            Ok(vec![Stmt::Var(
                make_span_string("beverage", 4),
                Some(Box::new(ws(Expr::Nil, 15..18)))
            ),])
        );

        unsafe {
            assert_eq!(
                parse_str("var beverage = x = nil;"),
                Ok(vec![Stmt::Var(
                    make_span_string("beverage", 4),
                    Some(Box::new(ws(Expr::Assign(
                        WithSpan::new_unchecked("x".into(), 15, 16),
                        Box::new(ws(Expr::Nil, 19..22))
                    ), 15..22)))
                ),])
            );
        }

        assert!(matches!(parse_str("if (nil) var beverage = nil;"), Err(SyntaxError::Unexpected(WithSpan{span:_,value: Token::Var}))));
    }

    #[test]
    fn test_if_stmt() {
        assert_eq!(
            parse_str("if(nil) print nil;"),
            Ok(vec![Stmt::If(
                Box::new(ws(Expr::Nil, 3..6)),
                Box::new(Stmt::Print(Box::new(ws(Expr::Nil, 14..17)))),
                None,
            ),])
        );
        assert_eq!(
            parse_str("if(nil) print nil; else print false;"),
            Ok(vec![Stmt::If(
                Box::new(ws(Expr::Nil, 3..6)),
                Box::new(Stmt::Print(Box::new(ws(Expr::Nil, 14..17)))),
                Some(Box::new(Stmt::Print(Box::new(ws(Expr::Boolean(false), 30..35))))),
            ),])
        );
    }

    #[test]
    fn test_block_stmt() {
        assert_eq!(parse_str("{}"), Ok(vec![Stmt::Block(vec![])]));
        assert_eq!(
            parse_str("{nil;}"),
            Ok(vec![Stmt::Block(vec![Stmt::Expression(Box::new(
                ws(Expr::Nil, 1..4)
            )),])])
        );
        assert_eq!(
            parse_str("{nil;nil;}"),
            Ok(vec![Stmt::Block(vec![
                Stmt::Expression(Box::new(ws(Expr::Nil, 1..4))),
                Stmt::Expression(Box::new(ws(Expr::Nil, 5..8))),
            ])])
        );
    }

    #[test]
    fn test_while_stmt() {
        assert_eq!(
            parse_str("while(nil)false;"),
            Ok(vec![Stmt::While(
                Box::new(ws(Expr::Nil, 6..9)),
                Box::new(Stmt::Expression(Box::new(ws(Expr::Boolean(false), 10..15)))),
            )])
        );
    }

    #[test]
    fn test_return_stmt() {
        assert_eq!(parse_str("return;"), Ok(vec![Stmt::Return(None),]));
        assert_eq!(
            parse_str("return nil;"),
            Ok(vec![Stmt::Return(Some(Box::new(ws(Expr::Nil, 7..10))))])
        );
    }

    #[test]
    fn test_function_stmt() {
        unsafe {
            assert_eq!(
                parse_str("fun test(){}"),
                Ok(vec![Stmt::Function(
                    WithSpan::new_unchecked("test".into(), 4, 8),
                    vec![],
                    vec![]
                ),])
            );
            assert_eq!(
                parse_str("fun test(a){}"),
                Ok(vec![Stmt::Function(
                    WithSpan::new_unchecked("test".into(), 4, 8),
                    vec![WithSpan::new_unchecked("a".into(), 9, 10)],
                    vec![]
                ),])
            );
            assert_eq!(
                parse_str("fun test(){nil;}"),
                Ok(vec![Stmt::Function(
                    WithSpan::new_unchecked("test".into(), 4, 8),
                    vec![],
                    vec![Stmt::Expression(Box::new(ws(Expr::Nil, 11..14))),]
                ),])
            );
        }
    }

    #[test]
    fn test_class_stmt() {
        unsafe {
            assert_eq!(
                parse_str("class test{}"),
                Ok(vec![Stmt::Class(
                    WithSpan::new_unchecked("test".into(), 6, 10),
                    None,
                    vec![]
                )])
            );
            assert_eq!(
                parse_str("class test{a(){}}"),
                Ok(vec![Stmt::Class(
                    WithSpan::new_unchecked("test".into(), 6, 10),
                    None,
                    vec![Stmt::Function(
                        WithSpan::new_unchecked("a".into(), 11, 12),
                        vec![],
                        vec![]
                    )]
                )])
            );
        }
    }

    #[test]
    fn test_class_inheritance() {
        unsafe {
            assert_eq!(
                parse_str("class BostonCream < Doughnut {}"),
                Ok(vec![Stmt::Class(
                    WithSpan::new_unchecked("BostonCream".into(), 6, 17),
                    Some(WithSpan::new_unchecked("Doughnut".into(), 20, 28)),
                    vec![]
                )])
            );
        }
        assert!(matches!(parse_str("class BostonCream < {}"), Err(SyntaxError::Expected(TokenKind::Identifier, WithSpan{span:_, value: Token::LeftBrace}))));
        assert!(matches!(parse_str("class BostonCream < Doughnut < BakedGood {}"), Err(SyntaxError::Expected(TokenKind::LeftBrace, WithSpan{span: _, value: Token::Less}))));
    }

    #[test]
    fn test_for() {
        fn block(what: Vec<Stmt>) -> Stmt {
            Stmt::Block(what)
        }
        fn var_i_zero(start: u32) -> Stmt {
            Stmt::Var(make_span_string("i", 8), Some(Box::new(ws(Expr::Number(0.), start..start+1))))
        }
        fn nil() -> Expr {
            Expr::Nil
        }
        fn while_stmt(e: WithSpan<Expr>, s: Stmt) -> Stmt {
            Stmt::While(Box::new(e), Box::new(s))
        }

        assert_eq!(
            parse_str("for(;;){}"),
            Ok(vec![while_stmt(ws(Expr::Boolean(true), 0..0), Stmt::Block(vec![])),])
        );
        assert_eq!(
            parse_str("for(var i=0;;){}"),
            Ok(vec![block(vec![
                var_i_zero(10),
                while_stmt(ws(Expr::Boolean(true), 0..0), Stmt::Block(vec![])),
            ])])
        );
        assert_eq!(
            parse_str("for(nil;nil;nil){}"),
            Ok(vec![block(vec![
                Stmt::Expression(Box::new(ws(nil(), 4..7))),
                while_stmt(
                    ws(Expr::Nil, 8..11),
                    Stmt::Block(vec![Stmt::Block(vec![]), Stmt::Expression(Box::new(ws(nil(), 12..15))),])
                ),
            ])])
        );
    }
}
use super::ast::*;
use super::token::*;
use crate::common::*;
use crate::parser::Parser;
use crate::position::Span;
use crate::position::WithSpan;

fn parse_program(it: &mut Parser) -> Result<Vec<WithSpan<Stmt>>, ()> {
    let mut statements = Vec::new();
    while !it.is_eof() {
        statements.push(parse_declaration(it)?);
    }

    Ok(statements)
}

fn parse_declaration(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    match it.peek() {
        TokenKind::Var => parse_var_declaration(it),
        TokenKind::Fun => parse_function_declaration(it),
        TokenKind::Class => parse_class_declaration(it),
        _ => parse_statement(it),
    }
}

fn parse_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    match it.peek() {
        TokenKind::Print => parse_print_statement(it),
        TokenKind::If => parse_if_statement(it),
        TokenKind::LeftBrace => parse_block_statement(it),
        TokenKind::While => parse_while_statement(it),
        TokenKind::Return => parse_return_statement(it),
        TokenKind::For => parse_for_statement(it),
        TokenKind::Import => parse_import_statement(it),
        _ => parse_expr_statement(it),
    }
}

fn parse_class_declaration(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = it.expect(TokenKind::Class)?;
    let name = expect_identifier(it)?;
    let superclass = if it.optionally(TokenKind::Less)? {
        let name = expect_identifier(it)?;
        Some(name.clone())
    } else {
        None
    };
    it.expect(TokenKind::LeftBrace)?;
    let mut functions: Vec<WithSpan<Stmt>> = vec![];
    while !it.check(TokenKind::RightBrace) {
        functions.push(parse_function(it)?);
    }
    let end_span = it.expect(TokenKind::RightBrace)?;

    Ok(WithSpan::new(Stmt::Class(name.clone(), superclass, functions), Span::union(begin_span, end_span)))
}

fn parse_function_declaration(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = it.expect(TokenKind::Fun)?;
    let fun = parse_function(it)?;

    let span = Span::union(begin_span, &fun);
    Ok(WithSpan::new(fun.value, span))
}

fn parse_function(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let name = expect_identifier(it)?;
    it.expect(TokenKind::LeftParen)?;
    let params = if !it.check(TokenKind::RightParen) {
        parse_params(it)?
    } else {
        Vec::new()
    };
    it.expect(TokenKind::RightParen)?;
    it.expect(TokenKind::LeftBrace)?;
    let mut body: Vec<WithSpan<Stmt>> = Vec::new();
    while !it.check(TokenKind::RightBrace) {
        body.push(parse_declaration(it)?);
    }
    let end_span = it.expect(TokenKind::RightBrace)?;
    Ok(WithSpan::new(Stmt::Function(name.clone(), params, body), Span::union(&name, end_span)))
}

fn parse_params(it: &mut Parser) -> Result<Vec<WithSpan<Identifier>>, ()> {
    let mut params: Vec<WithSpan<Identifier>> = Vec::new();
    params.push(expect_identifier(it)?);
    while it.check(TokenKind::Comma) {
        it.expect(TokenKind::Comma)?;
        params.push(expect_identifier(it)?);
    }
    Ok(params)
}

fn parse_var_declaration(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = it.expect(TokenKind::Var)?;
    let name = expect_identifier(it)?;
    let mut initializer = None;

    if it.optionally(TokenKind::Equal)? {
        initializer = Some(parse_expr(it)?);
    }

    let end_span = it.expect(TokenKind::Semicolon)?;

    Ok(WithSpan::new(Stmt::Var(name, initializer.map(Box::new)), Span::union(begin_span, end_span)))
}

fn parse_expr(it: &mut Parser) -> Result<WithSpan<Expr>, ()> {
    super::expr_parser::parse(it)
}

fn parse_for_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
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
        Some(expr) => {
            let span = expr.span;
            WithSpan::new(Stmt::Block(vec![body, WithSpan::new(Stmt::Expression(Box::new(expr)), span)]), span)
        },
        None => body,
    };
    let span = Span::union(&condition, &body);
    let body = WithSpan::new(Stmt::While(Box::new(condition), Box::new(body)), span);
    let body = match initializer {
        Some(stmt) => {
            let span = Span::union( &stmt, &body);
            WithSpan::new(Stmt::Block(vec![stmt, body]), span)
        },
        None => body,
    };

    Ok(body)
}

fn parse_import_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = it.expect(TokenKind::Import)?;
    let name = expect_string(it)?;
    let params = if it.check(TokenKind::For) {
        it.expect(TokenKind::For)?;
        Some(parse_params(it)?)
    } else {
        None
    };
    let end_span = it.expect(TokenKind::Semicolon)?;

    Ok(WithSpan::new(Stmt::Import(name, params), Span::union(begin_span, end_span)))
}

fn parse_return_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = it.expect(TokenKind::Return)?;
    let mut expr = None;
    if !it.check(TokenKind::Semicolon) {
        expr = Some(parse_expr(it)?);
    }
    let end_span = it.expect(TokenKind::Semicolon)?;
    Ok(WithSpan::new(Stmt::Return(expr.map(Box::new)), Span::union(begin_span, end_span)))
}

fn parse_expr_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let expr = parse_expr(it)?;
    let end_span = match it.expect2(TokenKind::Semicolon) {
        Some(token) => token,
        None => {
            return Ok(WithSpan::empty(Stmt::Error));
        },
    };

    let span = Span::union_span(expr.span, end_span);
    Ok(WithSpan::new(Stmt::Expression(Box::new(expr)), span))
}

fn parse_block_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = it.expect(TokenKind::LeftBrace)?;
    let mut statements: Vec<WithSpan<Stmt>> = Vec::new();
    while !it.check(TokenKind::RightBrace) {
        statements.push(parse_declaration(it)?);
    }
    let end_span = it.expect(TokenKind::RightBrace)?;
    Ok(WithSpan::new(Stmt::Block(statements), Span::union(begin_span, end_span)))
}

fn parse_while_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
   let begin_span =  it.expect(TokenKind::While)?;
    it.expect(TokenKind::LeftParen)?;
    let condition = parse_expr(it)?;
    it.expect(TokenKind::RightParen)?;
    let statement = parse_statement(it)?;
    let span = Span::union(begin_span, &statement);
    Ok(WithSpan::new(Stmt::While(Box::new(condition), Box::new(statement)), span))
}

fn parse_if_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_token = it.expect(TokenKind::If)?;
    it.expect(TokenKind::LeftParen)?;
    let condition = parse_expr(it)?;
    it.expect(TokenKind::RightParen)?;
    let if_stmt = parse_statement(it)?;
    let mut end_span = if_stmt.span;
    let mut else_stmt: Option<WithSpan<Stmt>> = None;

    if it.optionally(TokenKind::Else)? {
        let stmt = parse_statement(it)?;
        end_span = stmt.span;
        else_stmt = Some(stmt);
    }

    Ok(WithSpan::new(Stmt::If(
        Box::new(condition),
        Box::new(if_stmt),
        else_stmt.map(Box::new),
    ), Span::union_span(begin_token.span, end_span)))
}

fn parse_print_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_token = it.expect(TokenKind::Print)?;
    let expr = parse_expr(it)?;
    let end_token = it.expect(TokenKind::Semicolon)?;
    Ok( WithSpan::new(Stmt::Print(Box::new(expr)), Span::union(begin_token, end_token)) )
}

pub fn parse(it: &mut Parser) -> Result<Vec<WithSpan<Stmt>>, ()> {
    parse_program(it)
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use crate::position::Diagnostic;

    use super::super::tokenizer::*;
    use super::*;
    fn parse_str(data: &str) -> Result<Vec<WithSpan<Stmt>>, Vec<Diagnostic>> {
        let tokens = tokenize_with_context(data);
        let mut parser = crate::parser::Parser::new(&tokens);
        match parse(&mut parser) {
            Ok(ast) => Ok(ast),
            Err(_) => Err(parser.diagnostics().to_vec()),
        }
    }

    pub fn ws<T>(value: T, range: Range<u32>) -> WithSpan<T> {
        unsafe { WithSpan::new_unchecked(value, range.start, range.end) }
    }

    fn assert_errs(data: &str, errs: &[&str]) {
        let x = parse_str(data);
        assert!(x.is_err());
        let diagnostics = x.unwrap_err();
        for diag in diagnostics {
            assert!(errs.contains(&&diag.message.as_str()), "{}", diag.message);
        }
    }

    #[test]
    fn test_expr_stmt() {
        assert_eq!(
            parse_str("nil;"),
            Ok(vec![
                ws(Stmt::Expression(Box::new(ws(Expr::Nil, 0..3))), 0..4)
            ])
        );
        assert_eq!(
            parse_str("nil;nil;"),
            Ok(vec![
                ws(Stmt::Expression(Box::new(ws(Expr::Nil, 0..3))), 0..4),
                ws(Stmt::Expression(Box::new(ws(Expr::Nil, 4..7))), 4..8),
            ])
        );
    }

    #[test]
    fn test_print_stmt() {
        assert_eq!(
            parse_str("print nil;"),
            Ok(vec![
                ws(Stmt::Print(Box::new(ws(Expr::Nil, 6..9))), 0..10),
            ])
        );
    }

    fn make_span_string(string: &str, offset: u32) -> WithSpan<String> {
        unsafe { WithSpan::new_unchecked(string.into(), offset, offset+string.len() as u32) }
    }

    #[test]
    fn test_var_decl() {
        assert_eq!(
            parse_str("var beverage;"),
            Ok(vec![
                ws(Stmt::Var(make_span_string("beverage", 4), None), 0..13),
            ])
        );
        assert_eq!(
            parse_str("var beverage = nil;"),
            Ok(vec![
                ws(Stmt::Var(
                    make_span_string("beverage", 4),
                    Some(Box::new(ws(Expr::Nil, 15..18)))
                ), 0..19),
            ])
        );

        unsafe {
            assert_eq!(
                parse_str("var beverage = x = nil;"),
                Ok(vec![
                    ws(Stmt::Var(
                        make_span_string("beverage", 4),
                        Some(Box::new(ws(Expr::Assign(
                            WithSpan::new_unchecked("x".into(), 15, 16),
                            Box::new(ws(Expr::Nil, 19..22))
                        ), 15..22)))
                    ), 0..23),
                ])
            );
        }

        assert_errs("if (nil) var beverage = nil;", &["Unexpected 'var'"]);
    }

    #[test]
    fn test_if_stmt() {
        assert_eq!(
            parse_str("if(nil) print nil;"),
            Ok(vec![
                ws(Stmt::If(
                    Box::new(ws(Expr::Nil, 3..6)),
                    Box::new(ws(Stmt::Print(Box::new(ws(Expr::Nil, 14..17))), 8..18)),
                    None,
                ), 0..18),
            ])
        );
        assert_eq!(
            parse_str("if(nil) print nil; else print false;"),
            Ok(vec![
                ws(Stmt::If(
                    Box::new(ws(Expr::Nil, 3..6)),
                    Box::new(ws(Stmt::Print(Box::new(ws(Expr::Nil, 14..17))), 8..18)),
                    Some(Box::new(
                        ws(Stmt::Print(Box::new(ws(Expr::Boolean(false), 30..35))), 24..36),
                    )),
                ), 0..36),
            ])
        );
    }

    #[test]
    fn test_block_stmt() {
        assert_eq!(parse_str("{}"), Ok(vec![
            ws(Stmt::Block(vec![]), 0..2),
        ]));
        assert_eq!(
            parse_str("{nil;}"),
            Ok(vec![
                ws(Stmt::Block(vec![
                    ws(Stmt::Expression(Box::new(
                        ws(Expr::Nil, 1..4)
                    )), 1..5),
                ]), 0..6),
            ])
        );
        assert_eq!(
            parse_str("{nil;nil;}"),
            Ok(vec![
                ws(Stmt::Block(vec![
                    ws(Stmt::Expression(Box::new(ws(Expr::Nil, 1..4))), 1..5),
                    ws(Stmt::Expression(Box::new(ws(Expr::Nil, 5..8))), 5..9),
                ]), 0..10),
            ])
        );
    }

    #[test]
    fn test_while_stmt() {
        assert_eq!(
            parse_str("while(nil)false;"),
            Ok(vec![
                ws(Stmt::While(
                    Box::new(ws(Expr::Nil, 6..9)),
                    Box::new(ws(Stmt::Expression(Box::new(ws(Expr::Boolean(false), 10..15))), 10..16)),
                ), 0..16),
            ])
        );
    }

    #[test]
    fn test_return_stmt() {
        assert_eq!(parse_str("return;"), Ok(vec![
            ws(Stmt::Return(None), 0..7),
        ]));
        assert_eq!(
            parse_str("return nil;"),
            Ok(vec![
                ws(Stmt::Return(Some(Box::new(ws(Expr::Nil, 7..10)))), 0..11),
            ])
        );
    }

    #[test]
    fn test_import_stmt() {
        assert_eq!(parse_str("import \"mymodule\";"), Ok(vec![
            ws(Stmt::Import(
                ws("mymodule".into(), 7..17), 
                None
            ), 0..18),
        ]));

        assert_eq!(parse_str("import \"mymodule\" for message;"), Ok(vec![
            ws(Stmt::Import(
                ws("mymodule".into(), 7..17), 
                Some(vec![
                    ws("message".into(), 22..29),
                ])
            ), 0..30),
        ]));
    }

    #[test]
    fn test_function_stmt() {
        unsafe {
            assert_eq!(
                parse_str("fun test(){}"),
                Ok(vec![
                    ws(Stmt::Function(
                        WithSpan::new_unchecked("test".into(), 4, 8),
                        vec![],
                        vec![]
                    ), 0..12),
                ])
            );
            assert_eq!(
                parse_str("fun test(a){}"),
                Ok(vec![
                    ws(Stmt::Function(
                        WithSpan::new_unchecked("test".into(), 4, 8),
                        vec![WithSpan::new_unchecked("a".into(), 9, 10)],
                        vec![]
                    ), 0..13),
                ])
            );
            assert_eq!(
                parse_str("fun test(){nil;}"),
                Ok(vec![
                    ws(Stmt::Function(
                        WithSpan::new_unchecked("test".into(), 4, 8),
                        vec![],
                        vec![ws(Stmt::Expression(Box::new(ws(Expr::Nil, 11..14))), 11..15),]
                    ), 0..16),
                ])
            );
        }
    }

    #[test]
    fn test_class_stmt() {
        unsafe {
            assert_eq!(
                parse_str("class test{}"),
                Ok(vec![
                    ws(Stmt::Class(
                        WithSpan::new_unchecked("test".into(), 6, 10),
                        None,
                        vec![]
                    ), 0..12),
                ])
            );
            assert_eq!(
                parse_str("class test{a(){}}"),
                Ok(vec![
                    ws(Stmt::Class(
                        WithSpan::new_unchecked("test".into(), 6, 10),
                        None,
                        vec![ws(Stmt::Function(
                            WithSpan::new_unchecked("a".into(), 11, 12),
                            vec![],
                            vec![]
                        ), 11..16), ]
                    ), 0..17),
                ])
            );
        }
    }

    #[test]
    fn test_class_inheritance() {
        unsafe {
            assert_eq!(
                parse_str("class BostonCream < Doughnut {}"),
                Ok(vec![
                    ws(Stmt::Class(
                        WithSpan::new_unchecked("BostonCream".into(), 6, 17),
                        Some(WithSpan::new_unchecked("Doughnut".into(), 20, 28)),
                        vec![]
                    ), 0..31),
                ])
            );
        }
        assert_errs("class BostonCream < {}", &["Expected identifier got '{'"]);
        assert_errs("class BostonCream < Doughnut < BakedGood {}", &["Expected '{' got '<'"]);
    }

    #[test]
    fn test_for() {
        fn block(what: Vec<WithSpan<Stmt>>, r: Range<u32>) -> WithSpan<Stmt> {
            ws(Stmt::Block(what), r)
        }
        fn var_i_zero(start: u32, r: Range<u32>) -> WithSpan<Stmt> {
            ws(Stmt::Var(make_span_string("i", 8), Some(Box::new(ws(Expr::Number(0.), start..start+1)))), r)
        }
        fn nil() -> Expr {
            Expr::Nil
        }
        fn while_stmt(e: WithSpan<Expr>, s: WithSpan<Stmt>, r: Range<u32>) -> WithSpan<Stmt> {
            ws(Stmt::While(Box::new(e), Box::new(s)), r)
        }

        assert_eq!(
            parse_str("for(;;){}"),
            Ok(vec![
                while_stmt(ws(Expr::Boolean(true), 0..0), ws(Stmt::Block(vec![]), 7..9), 0..9),
            ])
        );
        assert_eq!(
            parse_str("for(var i=0;;){}"),
            Ok(vec![block(vec![
                var_i_zero(10, 4..12),
                while_stmt(ws(Expr::Boolean(true), 0..0), ws(Stmt::Block(vec![]), 14..16), 0..16),
            ], 0..16)])
        );
        assert_eq!(
            parse_str("for(nil;nil;nil){}"),
            Ok(vec![block(vec![
                ws(Stmt::Expression(Box::new(ws(nil(), 4..7))), 4..8),
                while_stmt(
                    ws(Expr::Nil, 8..11),
                    ws(Stmt::Block(vec![ws(Stmt::Block(vec![]), 16..18), ws(Stmt::Expression(Box::new(ws(nil(), 12..15))), 12..15), ]), 12..15),
                    8..15,
                ),
            ], 4..15)])
        );
    }
}
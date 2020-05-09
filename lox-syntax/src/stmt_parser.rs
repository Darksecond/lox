use super::ast::*;
use super::token::*;
use crate::position::WithSpan;
use crate::parser::Parser;
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
    let name = expect!(it, Token::Identifier(i) => i)?;
    let superclass = if it.optionally(TokenKind::Less)? {
        let name = expect!(it, Token::Identifier(i) => i)?;
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
    let name = expect!(it, Token::Identifier(i) => i)?;
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

fn parse_params(it: &mut Parser) -> Result<Vec<Identifier>, SyntaxError> {
    let mut params: Vec<Identifier> = Vec::new();
    params.push(expect!(it, Token::Identifier(i) => i.clone())?);
    while it.check(TokenKind::Comma) {
        it.expect(TokenKind::Comma)?;
        params.push(expect!(it, Token::Identifier(i) => i.clone())?);
    }
    Ok(params)
}

fn parse_var_declaration(it: &mut Parser) -> Result<Stmt, SyntaxError> {
    it.expect(TokenKind::Var)?;
    let name = expect_with_span!(it, Token::Identifier(i) => i.clone())?;
    let mut initializer: Option<Expr> = None;

    if it.optionally(TokenKind::Equal)? {
        initializer = Some(parse_expr(it)?);
    }

    it.expect(TokenKind::Semicolon)?;

    Ok(Stmt::Var(name, initializer.map(Box::new)))
}

fn parse_expr(it: &mut Parser) -> Result<Expr, SyntaxError> {
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
        Expr::Boolean(true)
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
    let mut expr: Option<Expr> = None;
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
    use super::super::tokenizer::*;
    use super::*;
    fn parse_str(data: &str) -> Result<Vec<Stmt>, SyntaxError> {
        let tokens = tokenize_with_context(data);
        let mut parser = crate::parser::Parser::new(&tokens);
        parse(&mut parser)
    }

    #[test]
    fn test_expr_stmt() {
        assert_eq!(
            parse_str("nil;"),
            Ok(vec![Stmt::Expression(Box::new(Expr::Nil)),])
        );
        assert_eq!(
            parse_str("nil;nil;"),
            Ok(vec![
                Stmt::Expression(Box::new(Expr::Nil)),
                Stmt::Expression(Box::new(Expr::Nil)),
            ])
        );
    }

    #[test]
    fn test_print_stmt() {
        assert_eq!(
            parse_str("print nil;"),
            Ok(vec![Stmt::Print(Box::new(Expr::Nil)),])
        );
    }

    fn make_span_string(string: &str, offset: u32) -> WithSpan<String> {
        use crate::position::{Span, BytePos};
        let start = BytePos(offset);
        let end = BytePos(string.len() as u32 + offset);
        WithSpan::new(string.to_string(), Span { start, end })
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
                Some(Box::new(Expr::Nil))
            ),])
        );

        assert_eq!(
            parse_str("var beverage = x = nil;"),
            Ok(vec![Stmt::Var(
                make_span_string("beverage", 4),
                Some(Box::new(Expr::Assign("x".into(), Box::new(Expr::Nil))))
            ),])
        );

        assert!(matches!(parse_str("if (nil) var beverage = nil;"), Err(SyntaxError::Unexpected(WithSpan{span:_,value: Token::Var}))));
    }

    #[test]
    fn test_if_stmt() {
        assert_eq!(
            parse_str("if(nil) print nil;"),
            Ok(vec![Stmt::If(
                Box::new(Expr::Nil),
                Box::new(Stmt::Print(Box::new(Expr::Nil))),
                None,
            ),])
        );
        assert_eq!(
            parse_str("if(nil) print nil; else print false;"),
            Ok(vec![Stmt::If(
                Box::new(Expr::Nil),
                Box::new(Stmt::Print(Box::new(Expr::Nil))),
                Some(Box::new(Stmt::Print(Box::new(Expr::Boolean(false))))),
            ),])
        );
    }

    #[test]
    fn test_block_stmt() {
        assert_eq!(parse_str("{}"), Ok(vec![Stmt::Block(vec![])]));
        assert_eq!(
            parse_str("{nil;}"),
            Ok(vec![Stmt::Block(vec![Stmt::Expression(Box::new(
                Expr::Nil
            )),])])
        );
        assert_eq!(
            parse_str("{nil;nil;}"),
            Ok(vec![Stmt::Block(vec![
                Stmt::Expression(Box::new(Expr::Nil)),
                Stmt::Expression(Box::new(Expr::Nil)),
            ])])
        );
    }

    #[test]
    fn test_while_stmt() {
        assert_eq!(
            parse_str("while(nil)false;"),
            Ok(vec![Stmt::While(
                Box::new(Expr::Nil),
                Box::new(Stmt::Expression(Box::new(Expr::Boolean(false)))),
            )])
        );
    }

    #[test]
    fn test_return_stmt() {
        assert_eq!(parse_str("return;"), Ok(vec![Stmt::Return(None),]));
        assert_eq!(
            parse_str("return nil;"),
            Ok(vec![Stmt::Return(Some(Box::new(Expr::Nil)))])
        );
    }

    #[test]
    fn test_function_stmt() {
        assert_eq!(
            parse_str("fun test(){}"),
            Ok(vec![Stmt::Function("test".into(), vec![], vec![]),])
        );
        assert_eq!(
            parse_str("fun test(a){}"),
            Ok(vec![Stmt::Function(
                "test".into(),
                vec!["a".into()],
                vec![]
            ),])
        );
        assert_eq!(
            parse_str("fun test(){nil;}"),
            Ok(vec![Stmt::Function(
                "test".into(),
                vec![],
                vec![Stmt::Expression(Box::new(Expr::Nil,)),]
            ),])
        );
    }

    #[test]
    fn test_class_stmt() {
        assert_eq!(
            parse_str("class test{}"),
            Ok(vec![Stmt::Class("test".into(), None, vec![]),])
        );
        assert_eq!(
            parse_str("class test{a(){}}"),
            Ok(vec![Stmt::Class(
                "test".into(),
                None,
                vec![Stmt::Function("a".into(), vec![], vec![])]
            )])
        );
    }

    #[test]
    fn test_class_inheritance() {
        assert_eq!(
            parse_str("class BostonCream < Doughnut {}"),
            Ok(vec![Stmt::Class(
                "BostonCream".into(),
                Some("Doughnut".into()),
                vec![]
            ),])
        );
        assert!(matches!(parse_str("class BostonCream < {}"), Err(SyntaxError::Unexpected(WithSpan{span:_,value: Token::LeftBrace}))));
        assert!(matches!(parse_str("class BostonCream < Doughnut < BakedGood {}"), Err(SyntaxError::Expected(TokenKind::LeftBrace, WithSpan{span: _, value: Token::Less}))));

    }

    #[test]
    fn test_for() {
        fn block(what: Vec<Stmt>) -> Stmt {
            Stmt::Block(what)
        }
        fn var_i_zero() -> Stmt {
            Stmt::Var(make_span_string("i", 8), Some(Box::new(Expr::Number(0.))))
        }
        fn nil() -> Expr {
            Expr::Nil
        }
        fn while_stmt(e: Expr, s: Stmt) -> Stmt {
            Stmt::While(Box::new(e), Box::new(s))
        }

        assert_eq!(
            parse_str("for(;;){}"),
            Ok(vec![while_stmt(Expr::Boolean(true), Stmt::Block(vec![])),])
        );
        assert_eq!(
            parse_str("for(var i=0;;){}"),
            Ok(vec![block(vec![
                var_i_zero(),
                while_stmt(Expr::Boolean(true), Stmt::Block(vec![])),
            ])])
        );
        assert_eq!(
            parse_str("for(nil;nil;nil){}"),
            Ok(vec![block(vec![
                Stmt::Expression(Box::new(nil())),
                while_stmt(
                    Expr::Nil,
                    Stmt::Block(vec![Stmt::Block(vec![]), Stmt::Expression(Box::new(nil())),])
                ),
            ])])
        );
    }
}

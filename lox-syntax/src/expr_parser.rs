use super::ast::*;
use super::token::*;
use crate::common::*;
use crate::parser::Parser;
use crate::position::{WithSpan, Span};

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd, Copy, Clone)]
enum Precedence {
    None,
    Assign, // =
    Or,
    And,
    Equality,   // == !=
    Comparison, // < <= > >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // ()
    Primary,
}

impl<'a> From<TokenKind> for Precedence {
    fn from(token: TokenKind) -> Precedence {
        match token {
            TokenKind::Equal => Precedence::Assign,
            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::BangEqual | TokenKind::EqualEqual => Precedence::Equality,
            TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash => Precedence::Factor,
            TokenKind::Bang => Precedence::Unary, // Minus is already specified, but I think this is only for infix ops
            TokenKind::LeftParen => Precedence::Call,
            TokenKind::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

fn parse_expr(it: &mut Parser, precedence: Precedence) -> Result<WithSpan<Expr>, ()> {
    let mut expr = parse_prefix(it)?;
    while !it.is_eof() {
        let next_precedence = Precedence::from(it.peek());
        if precedence >= next_precedence {
            break;
        }
        expr = parse_infix(it, expr)?;
    }
    Ok(expr)
}

fn parse_infix(it: &mut Parser, left: WithSpan<Expr>) -> Result<WithSpan<Expr>, ()> {
    match it.peek() {
        TokenKind::BangEqual
        | TokenKind::EqualEqual
        | TokenKind::Less
        | TokenKind::LessEqual
        | TokenKind::Greater
        | TokenKind::GreaterEqual
        | TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash => parse_binary(it, left),
        TokenKind::Or | TokenKind::And => parse_logical(it, left),
        TokenKind::Equal => parse_assign(it, left),
        TokenKind::LeftParen => parse_call(it, left),
        TokenKind::Dot => parse_get(it, left),
        _ => {
            it.error(&format!("Unexpected {}", it.peek_token().value), it.peek_token().span);
            Err(())
        },
    }
}

fn parse_prefix(it: &mut Parser) -> Result<WithSpan<Expr>, ()> {
    match it.peek() {
        TokenKind::Number
        | TokenKind::Nil
        | TokenKind::This
        | TokenKind::True
        | TokenKind::False
        | TokenKind::Identifier
        | TokenKind::Super
        | TokenKind::String => parse_primary(it),
        TokenKind::Bang | TokenKind::Minus => parse_unary(it),
        TokenKind::LeftParen => parse_grouping(it),
        _ => {
            it.error(&format!("Unexpected {}", it.peek_token().value), it.peek_token().span);
            Err(())
        },
    }
}

fn parse_get(it: &mut Parser, left: WithSpan<Expr>) -> Result<WithSpan<Expr>, ()> {
    it.expect(TokenKind::Dot)?;
    let tc = it.advance();
    match &tc.value {
        &Token::Identifier(ref i) => {
            let span = Span::union(&left, tc);
            Ok(WithSpan::new(Expr::Get(Box::new(left), WithSpan::new(i.clone(), tc.span)), span))
        },
        _ => {
            it.error(&format!("Expected identifier got {}", tc.value), tc.span);
            Err(())
        },
    }
}

fn parse_call(it: &mut Parser, left: WithSpan<Expr>) -> Result<WithSpan<Expr>, ()> {
    it.expect(TokenKind::LeftParen)?;
    let args = parse_arguments(it)?;
    let most_right = it.expect(TokenKind::RightParen)?;
    let span = Span::union(&left, most_right);
    Ok(WithSpan::new(Expr::Call(Box::new(left), args), span))
}

fn parse_arguments(it: &mut Parser) -> Result<Vec<WithSpan<Expr>>, ()> {
    let mut args = Vec::new();
    if !it.check(TokenKind::RightParen) {
        args.push(parse_expr(it, Precedence::None)?);
        while it.check(TokenKind::Comma) {
            it.expect(TokenKind::Comma)?;
            args.push(parse_expr(it, Precedence::None)?);
        }
    }
    Ok(args)
}

fn parse_assign(it: &mut Parser, left: WithSpan<Expr>) -> Result<WithSpan<Expr>, ()> {
    it.expect(TokenKind::Equal)?;
    let right = parse_expr(it, Precedence::None)?;
    let span = Span::union(&left, &right);
    match &left.value {
        Expr::Variable(i) => Ok(WithSpan::new(Expr::Assign(i.clone(), Box::new(right)), span)),
        Expr::Get(l, i) => Ok(WithSpan::new(Expr::Set(l.clone(), i.clone(), Box::new(right)), span)),
        _ => {
            it.error(&format!("Invalid left value"), left.span);
            Err(())
        },
    }
}

fn parse_logical(it: &mut Parser, left: WithSpan<Expr>) -> Result<WithSpan<Expr>, ()> {
    let precedence = Precedence::from(it.peek());
    let operator = parse_logical_op(it)?;
    let right = parse_expr(it, precedence)?;
    let span = Span::union(&left, &right);
    Ok(WithSpan::new(Expr::Logical(Box::new(left), operator, Box::new(right)), span))
}

fn parse_grouping(it: &mut Parser) -> Result<WithSpan<Expr>, ()> {
    let left_paren = it.expect(TokenKind::LeftParen)?;
    let expr = parse_expr(it, Precedence::None)?;
    let right_paren = it.expect(TokenKind::RightParen)?;

    let span = Span::union(left_paren, right_paren);
    Ok(WithSpan::new(Expr::Grouping(Box::new(expr)), span))
}

fn parse_binary(it: &mut Parser, left: WithSpan<Expr>) -> Result<WithSpan<Expr>, ()> {
    let precedence = Precedence::from(it.peek());
    let operator = parse_binary_op(it)?;
    let right = parse_expr(it, precedence)?;
    let span = Span::union(&left, &right);
    Ok(WithSpan::new(Expr::Binary(Box::new(left), operator, Box::new(right)), span))
}

fn parse_unary(it: &mut Parser) -> Result<WithSpan<Expr>, ()> {
    let operator = parse_unary_op(it)?;
    let right = parse_expr(it, Precedence::Unary)?;
    let span = Span::union(&operator, &right);
    Ok(WithSpan::new(Expr::Unary(operator, Box::new(right)), span))
}

fn parse_logical_op(it: &mut Parser) -> Result<WithSpan<LogicalOperator>, ()> {
    let tc = it.advance();
    let operator = match &tc.value {
        &Token::And => LogicalOperator::And,
        &Token::Or => LogicalOperator::Or,
        _ => {
            it.error(&format!("Expected logical operator got {}", tc.value), tc.span);
            return Err(())
        },
    };

    Ok(WithSpan::new(operator, tc.span))
}

fn parse_unary_op(it: &mut Parser) -> Result<WithSpan<UnaryOperator>, ()> {
    let tc = it.advance();
    match &tc.value {
        &Token::Bang => Ok(WithSpan::new(UnaryOperator::Bang, tc.span)),
        &Token::Minus => Ok(WithSpan::new(UnaryOperator::Minus, tc.span)),
        _ => {
            it.error(&format!("Expected unary operator got {}", tc.value), tc.span);
            Err(())
        }
    }
}

fn parse_binary_op(it: &mut Parser) -> Result<WithSpan<BinaryOperator>, ()> {
    let tc = it.advance();
    let operator = match &tc.value {
        &Token::BangEqual => BinaryOperator::BangEqual,
        &Token::EqualEqual => BinaryOperator::EqualEqual,
        &Token::Less => BinaryOperator::Less,
        &Token::LessEqual => BinaryOperator::LessEqual,
        &Token::Greater => BinaryOperator::Greater,
        &Token::GreaterEqual => BinaryOperator::GreaterEqual,
        &Token::Plus => BinaryOperator::Plus,
        &Token::Minus => BinaryOperator::Minus,
        &Token::Star => BinaryOperator::Star,
        &Token::Slash => BinaryOperator::Slash,
        _ => {
            it.error(&format!("Expected binary operator got {}", tc.value), tc.span);
            return Err(())
        },
    };

    Ok(WithSpan::new(operator, tc.span))
}

fn parse_primary(it: &mut Parser) -> Result<WithSpan<Expr>, ()> {
    let tc = it.advance();
    match &tc.value {
        &Token::Nil => Ok(WithSpan::new(Expr::Nil, tc.span)),
        &Token::This => Ok(WithSpan::new(Expr::This, tc.span)),
        &Token::Number(n) => Ok(WithSpan::new(Expr::Number(n), tc.span)),
        &Token::True => Ok(WithSpan::new(Expr::Boolean(true), tc.span)),
        &Token::False => Ok(WithSpan::new(Expr::Boolean(false), tc.span)),
        &Token::String(ref s) => Ok(WithSpan::new(Expr::String(s.clone()), tc.span)),
        &Token::Identifier(ref s) => Ok(WithSpan::new(Expr::Variable(WithSpan::new(s.clone(), tc.span)), tc.span)),
        &Token::Super => parse_super(it, &tc),
        _ => {
            it.error(&format!("Expected primary got {}", tc.value), tc.span);
            Err(())
        },
    }
}

fn parse_super(it: &mut Parser, keyword: &WithSpan<Token>) -> Result<WithSpan<Expr>, ()> {
    it.expect(TokenKind::Dot)?;
    let name = expect_identifier(it)?;
    let span = Span::union(keyword, &name);
    Ok(WithSpan::new(Expr::Super(name), span))
}

pub fn parse(it: &mut Parser) -> Result<WithSpan<Expr>, ()> {
    parse_expr(it, Precedence::None)
}

#[cfg(test)]
mod tests {
    use crate::position::Diagnostic;

    use super::*;
    fn parse_str(data: &str) -> Result<WithSpan<Expr>, Vec<Diagnostic>> {
        use super::super::tokenizer::*;

        let tokens = tokenize_with_context(data);
        let mut parser = crate::parser::Parser::new(&tokens);
        match parse(&mut parser) {
            Ok(e) => Ok(e),
            Err(_) => Err(parser.diagnostics().to_vec()),
        }
    }

    fn assert_errs(data: &str, errs: &[&str]) {
        let x = parse_str(data);
        assert!(x.is_err());
        let diagnostics = x.unwrap_err();
        for diag in diagnostics {
            assert!(errs.contains(&&diag.message.as_str()), "{}", diag.message);
        }
    }

    mod make {
        use super::*;
        use std::ops::Range;

        /// Make WithSpan
        pub fn ws<T>(value: T, range: Range<u32>) -> WithSpan<T> {
            unsafe { WithSpan::new_unchecked(value, range.start, range.end) }
        }

        /// Make Expr::Number
        pub fn n(value: f64) -> Expr {
            Expr::Number(value)
        }

        /// Make WithSpan<Expr::Number>
        pub fn wsn(value: f64, range: Range<u32>) -> WithSpan<Expr> {
            ws(n(value), range)
        }

        /// Make a Minus Number with span
        pub fn wsmn(value: f64, range: Range<u32>) -> WithSpan<Expr> {
            ws(Expr::Unary(ws(UnaryOperator::Minus, range.start..range.start+1), Box::new(ws(n(value), range.start+1..range.end))), range)
        }

        /// Make Expr::String
        pub fn s(value: &str) -> Expr {
            Expr::String(value.to_owned())
        }

        /// Make Expr::Variable
        pub fn v(value: &str, range: Range<u32>) -> Expr {
            Expr::Variable(ws(value.to_owned(), range))
        }

        /// Make WithSpan<Expr::Boolean>
        pub fn wsb(value: bool, range: Range<u32>) -> WithSpan<Expr> {
            ws(Expr::Boolean(value), range)
        }

        /// Make Expr::Unary
        pub fn uo(operator: UnaryOperator, operator_range: Range<u32>, expr: Expr, expr_range: Range<u32>) -> Expr {
            Expr::Unary(ws(operator, operator_range), Box::new(ws(expr, expr_range)))
        }

        /// Make WithSpan<Expr::Binary>
        pub fn wsbo(left: WithSpan<Expr>, op: WithSpan<BinaryOperator>, right: WithSpan<Expr>) -> WithSpan<Expr> {
            let span = Span::union(&left, &right);
            WithSpan::new(Expr::Binary(Box::new(left), op, Box::new(right)), span)
        }

        /// Make WithSpan<Expr::Logical>
        pub fn wslo(left: WithSpan<Expr>, op: WithSpan<LogicalOperator>, right: WithSpan<Expr>) -> WithSpan<Expr> {
            let span = Span::union(&left, &right);
            WithSpan::new(Expr::Logical(Box::new(left), op, Box::new(right)), span)
        }

        /// Make grouping with span
        pub fn wsg(expr: WithSpan<Expr>, range: Range<u32>) -> WithSpan<Expr> {
            ws(Expr::Grouping(Box::new(expr)), range)
        }

        /// Make assignment with span
        pub fn wsa(left: WithSpan<Identifier>, right: WithSpan<Expr>) -> WithSpan<Expr> {
            let span = Span::union(&left, &right);
            WithSpan::new(Expr::Assign(left, Box::new(right)), span)
        }

        /// WithSpan<Identifier>
        pub fn wsi(value: &str, range: Range<u32>) -> WithSpan<Identifier> {
            ws(value.into(), range)
        }

        pub fn wscall(left: WithSpan<Expr>, args: Vec<WithSpan<Expr>>, range: Range<u32>) -> WithSpan<Expr> {
            ws(Expr::Call(Box::new(left), args), range)
        }

        pub fn wsget(left: WithSpan<Expr>, right: WithSpan<Identifier>) -> WithSpan<Expr> {
            let span = Span::union(&left, &right);
            WithSpan::new(Expr::Get(Box::new(left), right), span)
        }

        pub fn wsset(left: WithSpan<Expr>, right: WithSpan<Identifier>, set: WithSpan<Expr>) -> WithSpan<Expr> {
            let span = Span::union(&left, &set);
            WithSpan::new(Expr::Set(Box::new(left), right, Box::new(set)), span)
        }
    }

    mod help {
        use super::*;
        use std::ops::Range;

        pub fn assert(expr: &str, expected: WithSpan<Expr>) {
            assert_eq!(parse_str(expr), Ok(expected));
        }

        pub fn assert2(expr: &str, expected: Expr, range: Range<u32>) {
            use super::make::ws;
            assert_eq!(parse_str(expr), Ok(ws(expected, range)));
        }

        pub fn simple_binary2(op: BinaryOperator, op_len: u32, start: u32) -> Expr {
            use super::make::*;

            let left = ws(n(1.0), 0+start..1+start);
            let op = ws(op, 1+start..1+start+op_len);
            let right = ws(n(2.0),1+start+op_len..2+start+op_len);

            Expr::Binary(Box::new(left), op, Box::new(right))
        }

        pub fn simple_binary(op: BinaryOperator, op_len: u32) -> Expr {
            simple_binary2(op, op_len, 0)
        }
    }

    #[test]
    fn test_primary() {
        use make::*;
        use help::assert;
        assert("nil", ws(Expr::Nil, 0..3));
        assert("1.0", ws(n(1.0), 0..3));
        assert("1", ws(n(1.0), 0..1));
        assert("true", ws(Expr::Boolean(true), 0..4));
        assert("false", ws(Expr::Boolean(false), 0..5));
        assert("\"iets\"", ws(s("iets"), 0..6));
        assert("iets", ws(v("iets", 0..4), 0..4));
        assert("this", ws(Expr::This, 0..4));
        assert("super.iets", ws(Expr::Super(ws("iets".into(), 6..10)), 0..10));
    }

    #[test]
    fn test_unary() {
        use make::*;
        use help::assert2;
        assert2("-nil", uo(UnaryOperator::Minus, 0..1, Expr::Nil, 1..4), 0..4);
        assert2("!nil", uo(UnaryOperator::Bang, 0..1, Expr::Nil, 1..4), 0..4);
        assert2("!!nil", uo(UnaryOperator::Bang, 0..1, uo(UnaryOperator::Bang, 1..2, Expr::Nil, 2..5), 1..5), 0..5);
        assert2("!-nil", uo(UnaryOperator::Bang, 0..1, uo(UnaryOperator::Minus, 1..2, Expr::Nil, 2..5), 1..5), 0..5);
        assert2("-!nil", uo(UnaryOperator::Minus, 0..1, uo(UnaryOperator::Bang, 1..2, Expr::Nil, 2..5), 1..5), 0..5);
    }

    #[test]
    fn test_binary() {
        use help::{assert2, simple_binary};
        assert2("1+2", simple_binary(BinaryOperator::Plus, 1), 0..3);
        assert2("1-2", simple_binary(BinaryOperator::Minus, 1), 0..3);
        assert2("1>2", simple_binary(BinaryOperator::Greater, 1), 0..3);
        assert2("1<2", simple_binary(BinaryOperator::Less, 1), 0..3);
        assert2("1*2", simple_binary(BinaryOperator::Star, 1), 0..3);
        assert2("1/2", simple_binary(BinaryOperator::Slash, 1), 0..3);

        assert2("1!=2", simple_binary(BinaryOperator::BangEqual, 2), 0..4);
        assert2("1==2", simple_binary(BinaryOperator::EqualEqual, 2), 0..4);
        assert2("1>=2", simple_binary(BinaryOperator::GreaterEqual, 2), 0..4);
        assert2("1<=2", simple_binary(BinaryOperator::LessEqual, 2), 0..4);
    }

    #[test]
    fn test_binary_precedence() {
        use help::assert;
        use make::*;

        let expr = wsbo(
            wsbo(
                wsn(1., 0..1),
                ws(BinaryOperator::Star, 1..2),
                wsn(2., 2..3)
            ),
            ws(BinaryOperator::Plus, 3..4),
            wsbo(
                wsn(3., 4..5),
                ws(BinaryOperator::Star, 5..6),
                wsn(4., 6..7)
            )
        );
        assert("1*2+3*4", expr);

        let expr = wsbo(
            wsmn(1., 0..2),
            ws(BinaryOperator::Star, 2..3),
            wsmn(2., 3..5)
        );
        assert("-1*-2", expr);
    }

    #[test]
    fn test_errors() {
        use help::{assert2, simple_binary};

        // Test infinite loops and extra tokens
        assert2("1+2 3", simple_binary(BinaryOperator::Plus, 1), 0..3);

        // assert!(matches!(parse_str("1+"), Err(SyntaxError::Unexpected(_))));
        assert_errs("1+", &["Unexpected <EOF>"]);
    }

    #[test]
    fn test_grouping() {
        use help::assert;
        use make::*;

        let expr = wsg(wsn(1., 1..2), 0..3);
        assert("(1)", expr);

        let expr = wsg(wsbo(
            wsn(1., 1..2),
            ws(BinaryOperator::Plus, 2..3),
            wsn(2., 3..4)
        ), 0..5);
        assert("(1+2)", expr);

        assert_errs("(1", &["Expected ')' got <EOF>"]);
        assert_errs("(1}", &["Expected ')' got '}'"]);
    }

    #[test]
    fn test_logical() {
        use help::assert;
        use make::*;

        let expr = wslo(
            wsb(true, 0..4),
            ws(LogicalOperator::Or, 5..7),
            wsb(false, 8..13)
        );
        assert("true or false", expr);

        let expr = wslo(
            wsb(true, 0..4),
            ws(LogicalOperator::And, 5..8),
            wsb(false, 9..14)
        );
        assert("true and false", expr);
    }

    #[test]
    fn test_logical_precedence() {
        use help::assert;
        use make::*;

        let left = wslo(
            wsn(1., 0..1),
            ws(LogicalOperator::And, 2..5),
            wsn(2., 6..7)
        );
        let right = wslo(
            wsn(3., 11..12),
            ws(LogicalOperator::And, 13..16),
            wsn(4., 17..18)
        );
        let expr = wslo(
            left,
            ws(LogicalOperator::Or, 8..10),
            right,
        );
        assert("1 and 2 or 3 and 4", expr);
    }

    #[test]
    fn test_assignment() {
        use help::{assert, simple_binary2};
        use make::*;

        let expr = wsa(wsi("a", 0..1), wsn(3., 2..3));
        assert("a=3", expr);
        let expr = wsa(wsi("a", 0..1), wsa(wsi("b", 2..3), wsn(3., 4..5)));
        assert("a=b=3", expr);
        let expr = wsa(wsi("a", 0..1), ws(simple_binary2(BinaryOperator::Plus, 1, 2), 2..5));
        assert("a=1+2", expr);

        assert_errs("a=", &["Unexpected <EOF>"]);
        assert_errs("3=3", &["Invalid left value"]);
    }

    #[test]
    fn test_call() {
        use help::assert;
        use make::*;
        
        let expr = wscall(
            ws(v("a", 0..1), 0..1),
            vec![],
            0..3
        );
        assert("a()", expr);

        let expr = wscall(
            ws(v("a", 0..1), 0..1),
            vec![
                wsn(3., 2..3)
            ],
            0..4
        );
        assert("a(3)", expr);

        let expr = wscall(
            ws(v("a", 0..1), 0..1),
            vec![
                wsn(3., 2..3),
                wsn(4., 4..5)
            ],
            0..6
        );
        assert("a(3,4)", expr);

        let expr = wscall(
            ws(v("a", 1..2), 1..2),
            vec![],
            1..4
        );
        let expr = ws(Expr::Unary(ws(UnaryOperator::Minus, 0..1), Box::new(expr)), 0..4);
        assert("-a()", expr);

        let left = wscall(
            ws(v("a", 0..1), 0..1),
            vec![],
            0..3
        );
        let right = wscall(
            ws(v("b", 4..5), 4..5),
            vec![],
            4..7
        );
        let expr = wsbo(left, ws(BinaryOperator::Plus, 3..4), right);
        assert("a()+b()", expr);

        assert_errs("a(3,)", &["Unexpected ')'"]);
    }

    #[test]
    fn test_get() {
        use help::assert;
        use make::*;

        let left = wsget(
            ws(v("a", 0..1), 0..1),
            wsi("b", 2..3)
        );
        let expr = wsget(
            left,
            wsi("c", 4..5)
        );
        assert("a.b.c", expr);
    }

    #[test]
    fn test_set() {
        use help::assert;
        use make::*;

        let expr = wsset(
            ws(v("a", 0..1), 0..1),
            wsi("b", 2..3),
            wsn(3., 4..5)
        );
        assert("a.b=3", expr);
    }
}
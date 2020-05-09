use super::ast::*;
use super::token::*;
use crate::parser::Parser;
use crate::SyntaxError;
use crate::position::WithSpan;

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

impl<'a> From<&'a Token> for Precedence {
    fn from(token: &Token) -> Precedence {
        match *token {
            Token::Equal => Precedence::Assign,
            Token::Or => Precedence::Or,
            Token::And => Precedence::And,
            Token::BangEqual | Token::EqualEqual => Precedence::Equality,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => {
                Precedence::Comparison
            }
            Token::Plus | Token::Minus => Precedence::Term,
            Token::Star | Token::Slash => Precedence::Factor,
            Token::Bang => Precedence::Unary, // Minus is already specified, but I think this is only for infix ops
            Token::LeftParen => Precedence::Call,
            Token::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

impl<'a> From<TokenKind> for Precedence {
    fn from(token: TokenKind) -> Precedence {
        match token {
            TokenKind::Equal => Precedence::Assign,
            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::BangEqual | TokenKind::EqualEqual => Precedence::Equality,
            TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => {
                Precedence::Comparison
            }
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash => Precedence::Factor,
            TokenKind::Bang => Precedence::Unary, // Minus is already specified, but I think this is only for infix ops
            TokenKind::LeftParen => Precedence::Call,
            TokenKind::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

fn parse_expr(it: &mut Parser, precedence: Precedence) -> Result<Expr, SyntaxError> {
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

fn parse_infix(it: &mut Parser, left: Expr) -> Result<Expr, SyntaxError> {
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
        _ => Err(SyntaxError::Unexpected(it.peek_token().clone())),
    }
}

fn parse_prefix(it: &mut Parser) -> Result<Expr, SyntaxError> {
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
        _ => Err(SyntaxError::Unexpected(it.peek_token().clone())),
    }
}

fn parse_get(it: &mut Parser, left: Expr) -> Result<Expr, SyntaxError> {
    it.expect(TokenKind::Dot)?;
    let tc = it.advance();
    match &tc.value {
        &Token::Identifier(ref i) => Ok(Expr::Get(Box::new(left), i.clone())),
        _ => Err(SyntaxError::Expected(TokenKind::Identifier, tc.clone())),
    }
}

fn parse_call(it: &mut Parser, left: Expr) -> Result<Expr, SyntaxError> {
    it.expect(TokenKind::LeftParen)?;
    let args = parse_arguments(it)?;
    it.expect(TokenKind::RightParen)?;
    Ok(Expr::Call(Box::new(left), args))
}

fn parse_arguments(it: &mut Parser) -> Result<Vec<Expr>, SyntaxError> {
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

fn parse_assign(it: &mut Parser, left: Expr) -> Result<Expr, SyntaxError> {
    it.expect(TokenKind::Equal)?;
    let right = parse_expr(it, Precedence::None)?;
    match left {
        Expr::Variable(i) => Ok(Expr::Assign(i, Box::new(right))),
        Expr::Get(l, i) => Ok(Expr::Set(l, i, Box::new(right))),
        e => Err(SyntaxError::InvalidLeftValue(WithSpan::empty(e.clone()))), //TODO
    }
}

fn parse_logical(it: &mut Parser, left: Expr) -> Result<Expr, SyntaxError> {
    let precedence = Precedence::from(it.peek());
    let operator = parse_logical_op(it)?;
    let right = parse_expr(it, precedence)?;
    Ok(Expr::Logical(Box::new(left), operator, Box::new(right)))
}

fn parse_grouping(it: &mut Parser) -> Result<Expr, SyntaxError> {
    it.expect(TokenKind::LeftParen)?;
    let expr = parse_expr(it, Precedence::None)?;
    it.expect(TokenKind::RightParen)?;
    Ok(Expr::Grouping(Box::new(expr)))
}

fn parse_binary(it: &mut Parser, left: Expr) -> Result<Expr, SyntaxError> {
    let precedence = Precedence::from(it.peek());
    let operator = parse_binary_op(it)?;
    let right = parse_expr(it, precedence)?;
    Ok(Expr::Binary(Box::new(left), operator, Box::new(right)))
}

fn parse_unary(it: &mut Parser) -> Result<Expr, SyntaxError> {
    let operator = parse_unary_op(it)?;
    let right = parse_expr(it, Precedence::Unary)?;
    Ok(Expr::Unary(operator, Box::new(right)))
}

fn parse_logical_op(it: &mut Parser) -> Result<LogicalOperator, SyntaxError> {
    let tc = it.advance();
    match &tc.value {
        &Token::And => Ok(LogicalOperator::And),
        &Token::Or => Ok(LogicalOperator::Or),
        _ => Err(SyntaxError::ExpectedUnaryOperator(tc.clone())),
    }
}

fn parse_unary_op(it: &mut Parser) -> Result<UnaryOperator, SyntaxError> {
    let tc = it.advance();
    match &tc.value {
        &Token::Bang => Ok(UnaryOperator::Bang),
        &Token::Minus => Ok(UnaryOperator::Minus),
        _ => Err(SyntaxError::ExpectedUnaryOperator(tc.clone())),
    }
}

fn parse_binary_op(it: &mut Parser) -> Result<BinaryOperator, SyntaxError> {
    let tc = it.advance();
    match &tc.value {
        &Token::BangEqual => Ok(BinaryOperator::BangEqual),
        &Token::EqualEqual => Ok(BinaryOperator::EqualEqual),
        &Token::Less => Ok(BinaryOperator::Less),
        &Token::LessEqual => Ok(BinaryOperator::LessEqual),
        &Token::Greater => Ok(BinaryOperator::Greater),
        &Token::GreaterEqual => Ok(BinaryOperator::GreaterEqual),
        &Token::Plus => Ok(BinaryOperator::Plus),
        &Token::Minus => Ok(BinaryOperator::Minus),
        &Token::Star => Ok(BinaryOperator::Star),
        &Token::Slash => Ok(BinaryOperator::Slash),
        _ => Err(SyntaxError::ExpectedBinaryOperator(tc.clone())),
    }
}

fn parse_primary(it: &mut Parser) -> Result<Expr, SyntaxError> {
    let tc = it.advance();
    match &tc.value {
        &Token::Nil => Ok(Expr::Nil),
        &Token::This => Ok(Expr::This),
        &Token::Number(n) => Ok(Expr::Number(n)),
        &Token::True => Ok(Expr::Boolean(true)),
        &Token::False => Ok(Expr::Boolean(false)),
        &Token::String(ref s) => Ok(Expr::String(s.clone())),
        &Token::Identifier(ref s) => Ok(Expr::Variable(s.clone())),
        &Token::Super => parse_super(it),
        _ => Err(SyntaxError::ExpectedPrimary(tc.clone())),
    }
}

fn parse_super(it: &mut Parser) -> Result<Expr, SyntaxError> {
    it.expect(TokenKind::Dot)?;
    let tc = it.advance();
    match &tc.value {
        &Token::Identifier(ref i) => Ok(Expr::Super(i.clone())),
        _ => Err(SyntaxError::Expected(TokenKind::Identifier, tc.clone())),
    }
}

pub fn parse(it: &mut Parser) -> Result<Expr, SyntaxError> {
    parse_expr(it, Precedence::None)
}

#[cfg(test)]
mod tests {
    use super::super::tokenizer::*;
    use super::*;
    fn parse_str(data: &str) -> Result<Expr, SyntaxError> {
        let tokens = tokenize_with_context(data);
        let mut parser = crate::parser::Parser::new(&tokens);
        parse(&mut parser)
    }

    mod make {
        use super::*;
        pub fn nr(value: f64) -> Expr {
            Expr::Number(value)
        }
        pub fn simple_binary(operator: BinaryOperator) -> Expr {
            let left = nr(1.);
            let right = nr(2.);
            Expr::Binary(Box::new(left), operator, Box::new(right))
        }
        pub fn binary(left: Expr, operator: BinaryOperator, right: Expr) -> Expr {
            Expr::Binary(Box::new(left), operator, Box::new(right))
        }
        pub fn minus_nr(value: f64) -> Expr {
            Expr::Unary(UnaryOperator::Minus, Box::new(nr(value)))
        }
    }

    #[test]
    fn test_primary() {
        assert_eq!(parse_str("nil"), Ok(Expr::Nil));
        assert_eq!(parse_str("1.0"), Ok(Expr::Number(1.0)));
        assert_eq!(parse_str("1"), Ok(Expr::Number(1.0)));
        assert_eq!(parse_str("true"), Ok(Expr::Boolean(true)));
        assert_eq!(parse_str("false"), Ok(Expr::Boolean(false)));
        assert_eq!(
            parse_str("\"test\""),
            Ok(Expr::String(String::from("test")))
        );
        assert_eq!(parse_str("test"), Ok(Expr::Variable("test".into())));
        assert_eq!(parse_str("this"), Ok(Expr::This));
        assert_eq!(parse_str("super.iets"), Ok(Expr::Super("iets".into())));
    }

    #[test]
    fn test_unary() {
        assert_eq!(
            parse_str("-nil"),
            Ok(Expr::Unary(UnaryOperator::Minus, Box::new(Expr::Nil)))
        );
        assert_eq!(
            parse_str("!nil"),
            Ok(Expr::Unary(UnaryOperator::Bang, Box::new(Expr::Nil)))
        );
        assert_eq!(
            parse_str("!!nil"),
            Ok(Expr::Unary(
                UnaryOperator::Bang,
                Box::new(Expr::Unary(UnaryOperator::Bang, Box::new(Expr::Nil)))
            ))
        );
        assert_eq!(
            parse_str("!-nil"),
            Ok(Expr::Unary(
                UnaryOperator::Bang,
                Box::new(Expr::Unary(UnaryOperator::Minus, Box::new(Expr::Nil)))
            ))
        );
        assert_eq!(
            parse_str("-!nil"),
            Ok(Expr::Unary(
                UnaryOperator::Minus,
                Box::new(Expr::Unary(UnaryOperator::Bang, Box::new(Expr::Nil)))
            ))
        );
    }

    #[test]
    fn test_binary() {
        assert_eq!(
            parse_str("1!=2"),
            Ok(make::simple_binary(BinaryOperator::BangEqual))
        );
        assert_eq!(
            parse_str("1==2"),
            Ok(make::simple_binary(BinaryOperator::EqualEqual))
        );
        assert_eq!(
            parse_str("1>2"),
            Ok(make::simple_binary(BinaryOperator::Greater))
        );
        assert_eq!(
            parse_str("1>=2"),
            Ok(make::simple_binary(BinaryOperator::GreaterEqual))
        );
        assert_eq!(
            parse_str("1<2"),
            Ok(make::simple_binary(BinaryOperator::Less))
        );
        assert_eq!(
            parse_str("1<=2"),
            Ok(make::simple_binary(BinaryOperator::LessEqual))
        );
        assert_eq!(
            parse_str("1+2"),
            Ok(make::simple_binary(BinaryOperator::Plus))
        );
        assert_eq!(
            parse_str("1-2"),
            Ok(make::simple_binary(BinaryOperator::Minus))
        );
        assert_eq!(
            parse_str("1*2"),
            Ok(make::simple_binary(BinaryOperator::Star))
        );
        assert_eq!(
            parse_str("1/2"),
            Ok(make::simple_binary(BinaryOperator::Slash))
        );
    }

    #[test]
    fn test_binary_precedence() {
        use self::make::*;
        assert_eq!(
            parse_str("1*2+3*4"),
            Ok(binary(
                binary(nr(1.), BinaryOperator::Star, nr(2.)),
                BinaryOperator::Plus,
                binary(nr(3.), BinaryOperator::Star, nr(4.))
            ))
        );
        assert_eq!(
            parse_str("-1*-2"),
            Ok(binary(minus_nr(1.), BinaryOperator::Star, minus_nr(2.)))
        );
    }

    #[test]
    fn test_errors() {
        // Test infinite loops and extra tokens
        assert_eq!(
            parse_str("1+2 3"),
            Ok(make::simple_binary(BinaryOperator::Plus))
        );
        assert!(matches!(parse_str("1+"), Err(SyntaxError::Unexpected(_))));
    }

    #[test]
    fn test_grouping() {
        use self::make::*;
        assert_eq!(parse_str("(1)"), Ok(Expr::Grouping(Box::new(make::nr(1.)))));
        assert_eq!(
            parse_str("((1))"),
            Ok(Expr::Grouping(Box::new(Expr::Grouping(Box::new(
                make::nr(1.)
            )))))
        );
        assert_eq!(
            parse_str("(1+2)*(1+2)"),
            Ok(binary(
                Expr::Grouping(Box::new(simple_binary(BinaryOperator::Plus))),
                BinaryOperator::Star,
                Expr::Grouping(Box::new(simple_binary(BinaryOperator::Plus))),
            ))
        );
        assert!(matches!(parse_str("(1"), Err(SyntaxError::Expected(TokenKind::RightParen, _))));
        assert!(matches!(parse_str("(1}"), Err(SyntaxError::Expected(TokenKind::RightParen, WithSpan{span: _, value: Token::RightBrace}))));
    }

    #[test]
    fn test_logical() {
        assert_eq!(
            parse_str("true or false"),
            Ok(Expr::Logical(
                Box::new(Expr::Boolean(true)),
                LogicalOperator::Or,
                Box::new(Expr::Boolean(false)),
            ))
        );
        assert_eq!(
            parse_str("true and false"),
            Ok(Expr::Logical(
                Box::new(Expr::Boolean(true)),
                LogicalOperator::And,
                Box::new(Expr::Boolean(false)),
            ))
        );
    }

    #[test]
    fn test_logical_precedence() {
        assert_eq!(
            parse_str("1 and 2 or 3 and 4"),
            Ok(Expr::Logical(
                Box::new(Expr::Logical(
                    Box::new(Expr::Number(1.)),
                    LogicalOperator::And,
                    Box::new(Expr::Number(2.)),
                )),
                LogicalOperator::Or,
                Box::new(Expr::Logical(
                    Box::new(Expr::Number(3.)),
                    LogicalOperator::And,
                    Box::new(Expr::Number(4.)),
                )),
            ))
        );
    }

    #[test]
    fn test_assignment() {
        assert_eq!(
            parse_str("a=3"),
            Ok(Expr::Assign("a".into(), Box::new(Expr::Number(3.))))
        );
        assert_eq!(
            parse_str("a=b=3"),
            Ok(Expr::Assign(
                "a".into(),
                Box::new(Expr::Assign("b".into(), Box::new(Expr::Number(3.))))
            ))
        );
        assert!(matches!(parse_str("a="), Err(SyntaxError::Unexpected(_))));
        assert!(matches!(parse_str("3=3"), Err(SyntaxError::InvalidLeftValue(WithSpan{span: _, value: Expr::Number(_)}))));

        assert_eq!(
            parse_str("a=1+2"),
            Ok(Expr::Assign(
                "a".into(),
                Box::new(make::simple_binary(BinaryOperator::Plus))
            ))
        );
    }

    #[test]
    fn test_call() {
        assert_eq!(
            parse_str("a()"),
            Ok(Expr::Call(Box::new(Expr::Variable("a".into())), vec![]))
        );

        assert_eq!(
            parse_str("a(3)"),
            Ok(Expr::Call(
                Box::new(Expr::Variable("a".into())),
                vec![Expr::Number(3.)]
            ))
        );
        assert_eq!(
            parse_str("a(3,4)"),
            Ok(Expr::Call(
                Box::new(Expr::Variable("a".into())),
                vec![Expr::Number(3.), Expr::Number(4.),]
            ))
        );

        assert_eq!(
            parse_str("-a(3)"),
            Ok(Expr::Unary(
                UnaryOperator::Minus,
                Box::new(Expr::Call(
                    Box::new(Expr::Variable("a".into())),
                    vec![Expr::Number(3.)]
                ))
            ))
        );

        assert_eq!(
            parse_str("a(3)+a(3)"),
            Ok(Expr::Binary(
                Box::new(Expr::Call(
                    Box::new(Expr::Variable("a".into())),
                    vec![Expr::Number(3.)]
                )),
                BinaryOperator::Plus,
                Box::new(Expr::Call(
                    Box::new(Expr::Variable("a".into())),
                    vec![Expr::Number(3.)]
                ))
            ))
        );

        assert!(matches!(parse_str("a(3,)"), Err(SyntaxError::Unexpected(WithSpan{span: _, value: Token::RightParen}))));
    }

    #[test]
    fn test_get() {
        assert_eq!(
            parse_str("a.b"),
            Ok(Expr::Get(Box::new(Expr::Variable("a".into())), "b".into(),))
        );

        assert_eq!(
            parse_str("a.b.c"),
            Ok(Expr::Get(
                Box::new(Expr::Get(Box::new(Expr::Variable("a".into())), "b".into(),)),
                "c".into(),
            ))
        );

        assert_eq!(
            parse_str("a.b(3).c"),
            Ok(Expr::Get(
                Box::new(Expr::Call(
                    Box::new(Expr::Get(Box::new(Expr::Variable("a".into())), "b".into())),
                    vec![Expr::Number(3.0)]
                )),
                "c".into()
            ))
        );
    }

    #[test]
    fn test_set() {
        assert_eq!(
            parse_str("a.b=3"),
            Ok(Expr::Set(
                Box::new(Expr::Variable("a".into())),
                "b".into(),
                Box::new(Expr::Number(3.))
            ))
        );
    }
}

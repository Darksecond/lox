use super::ast::*;
use super::common::*;
use super::token::*;
use super::tokenizer::TokenWithContext;
use std::iter::{Iterator, Peekable};

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

fn parse_expr<'a, It>(it: &mut Peekable<It>, precedence: Precedence) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    let mut expr = parse_prefix(it)?;
    while let Some(&token) = it.peek() {
        let next_precedence = Precedence::from(&token.token);
        if precedence >= next_precedence {
            break;
        }
        expr = parse_infix(it, expr)?;
    }
    Ok(expr)
}

fn parse_infix<'a, It>(it: &mut Peekable<It>, left: Expr) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match peek(it)? {
        &Token::BangEqual
        | &Token::EqualEqual
        | &Token::Less
        | &Token::LessEqual
        | &Token::Greater
        | &Token::GreaterEqual
        | &Token::Plus
        | &Token::Minus
        | &Token::Star
        | &Token::Slash => parse_binary(it, left),
        &Token::Or | &Token::And => parse_logical(it, left),
        &Token::Equal => parse_assign(it, left),
        &Token::LeftParen => parse_call(it, left),
        &Token::Dot => parse_get(it, left),
        t => Err(error(it, format!("unexpected token: {:?}", t)))
    }
}

fn parse_prefix<'a, It>(it: &mut Peekable<It>) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match peek(it)? {
        &Token::Number(_)
        | &Token::Nil
        | &Token::This
        | &Token::True
        | &Token::False
        | &Token::Identifier(_)
        | &Token::Super
        | &Token::String(_) => parse_primary(it),

        &Token::Bang | &Token::Minus => parse_unary(it),

        &Token::LeftParen => parse_grouping(it),
        t => Err(error(it, format!("unexpected token: {:?}", t)))
    }
}

fn parse_get<'a, It>(it: &mut Peekable<It>, left: Expr) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    expect(it, &Token::Dot)?;
    match next(it)? {
        &Token::Identifier(ref i) => Ok(Expr::Get(Box::new(left), i.clone())),
        t => Err(format!("unexpected token expected identifier: {:?}", t).into()),
    }
}

fn parse_call<'a, It>(it: &mut Peekable<It>, left: Expr) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    expect(it, &Token::LeftParen)?;
    let args = parse_arguments(it)?;
    expect(it, &Token::RightParen)?;
    Ok(Expr::Call(Box::new(left), args))
}

fn parse_arguments<'a, It>(it: &mut Peekable<It>) -> Result<Vec<Expr>, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    let mut args = Vec::new();
    if peek(it)? != &Token::RightParen {
        args.push(parse_expr(it, Precedence::None)?);
        while peek(it)? == &Token::Comma {
            expect(it, &Token::Comma)?;
            args.push(parse_expr(it, Precedence::None)?);
        }
    }
    Ok(args)
}

fn parse_assign<'a, It>(it: &mut Peekable<It>, left: Expr) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    expect(it, &Token::Equal)?;
    let right = parse_expr(it, Precedence::None)?;
    match left {
        Expr::Variable(i) => Ok(Expr::Assign(i, Box::new(right))),
        Expr::Get(l, i) => Ok(Expr::Set(l, i, Box::new(right))),
        e => Err(format!("invalid l-value: {:?}", e).into()),
    }
}

fn parse_logical<'a, It>(it: &mut Peekable<It>, left: Expr) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    let precedence = Precedence::from(peek(it)?);
    let operator = parse_logical_op(it)?;
    let right = parse_expr(it, precedence)?;
    Ok(Expr::Logical(Box::new(left), operator, Box::new(right)))
}

fn parse_grouping<'a, It>(it: &mut Peekable<It>) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    expect(it, &Token::LeftParen)?;
    let expr = parse_expr(it, Precedence::None)?;
    expect(it, &Token::RightParen)?;
    Ok(Expr::Grouping(Box::new(expr)))
}

fn parse_binary<'a, It>(it: &mut Peekable<It>, left: Expr) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    let precedence = Precedence::from(peek(it)?);
    let operator = parse_binary_op(it)?;
    let right = parse_expr(it, precedence)?;
    Ok(Expr::Binary(Box::new(left), operator, Box::new(right)))
}

fn parse_unary<'a, It>(it: &mut Peekable<It>) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    let operator = parse_unary_op(it)?;
    let right = parse_expr(it, Precedence::Unary)?;
    Ok(Expr::Unary(operator, Box::new(right)))
}

fn parse_logical_op<'a, It>(it: &mut Peekable<It>) -> Result<LogicalOperator, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match next(it)? {
        &Token::And => Ok(LogicalOperator::And),
        &Token::Or => Ok(LogicalOperator::Or),
        t => Err(format!("expected unary op got {:?}", t).into()),
    }
}

fn parse_unary_op<'a, It>(it: &mut Peekable<It>) -> Result<UnaryOperator, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match next(it)? {
        &Token::Bang => Ok(UnaryOperator::Bang),
        &Token::Minus => Ok(UnaryOperator::Minus),
        t => Err(format!("expected unary op got {:?}", t).into()),
    }
}

fn parse_binary_op<'a, It>(it: &mut Peekable<It>) -> Result<BinaryOperator, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match next(it)? {
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
        t => Err(format!("expected binary op got {:?}", t).into()),
    }
}

fn parse_primary<'a, It>(it: &mut Peekable<It>) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    match next(it)? {
        &Token::Nil => Ok(Expr::Nil),
        &Token::This => Ok(Expr::This),
        &Token::Number(n) => Ok(Expr::Number(n)),
        &Token::True => Ok(Expr::Boolean(true)),
        &Token::False => Ok(Expr::Boolean(false)),
        &Token::String(ref s) => Ok(Expr::String(s.clone())),
        &Token::Identifier(ref s) => Ok(Expr::Variable(s.clone())),
        &Token::Super => parse_super(it),
        t => Err(format!("expected primary got {:?}", t).into()),
    }
}

fn parse_super<'a, It>(it: &mut Peekable<It>) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    expect(it, &Token::Dot)?;
    match next(it)? {
        &Token::Identifier(ref i) => Ok(Expr::Super(i.clone())),
        t => Err(format!("expected identifier got {:?}", t).into()),
    }
}

pub fn parse<'a, It>(it: &mut Peekable<It>) -> Result<Expr, ParseError>
where
    It: Iterator<Item = &'a TokenWithContext>,
{
    parse_expr(it, Precedence::None)
}

#[cfg(test)]
mod tests {
    use super::super::tokenizer::*;
    use super::*;
    fn parse_str(data: &str) -> Result<Expr, String> {
        let tokens = tokenize_with_context(data);
        let mut it = tokens.as_slice().into_iter().peekable();
        parse(&mut it).map_err(|e| e.error)
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
        assert_eq!(parse_str("1+"), Err("No more tokens".into()));
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
        assert_eq!(parse_str("(1"), Err("No more tokens".into()));
        assert_eq!(
            parse_str("(1}"),
            Err("Expected RightParen got RightBrace".into())
        );
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
        assert_eq!(parse_str("a="), Err("No more tokens".into()));
        assert_eq!(parse_str("3=3"), Err("invalid l-value: Number(3.0)".into()));
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

        assert_eq!(
            parse_str("a(3,)"),
            Err("unexpected token: RightParen".into())
        );
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

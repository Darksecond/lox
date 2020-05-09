use crate::position::WithSpan;

pub type Identifier = String;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UnaryOperator {
    Bang,
    Minus,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BinaryOperator {
    Slash,
    Star,
    Plus,
    Minus,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    BangEqual,
    EqualEqual,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Grouping(Box<Expr>),
    Number(f64),
    Boolean(bool),
    Nil,
    This,
    Super(Identifier),
    String(String),
    Unary(UnaryOperator, Box<Expr>),
    Variable(Identifier),
    Logical(Box<Expr>, LogicalOperator, Box<Expr>),
    Assign(Identifier, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Get(Box<Expr>, Identifier),
    Set(Box<Expr>, Identifier, Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Var(WithSpan<Identifier>, Option<Box<Expr>>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Block(Vec<Stmt>),
    While(Box<Expr>, Box<Stmt>),
    Return(Option<Box<Expr>>),
    Function(WithSpan<Identifier>, Vec<WithSpan<Identifier>>, Vec<Stmt>),
    Class(WithSpan<Identifier>, Option<WithSpan<Identifier>>, Vec<Stmt>),
}

pub type Ast = Vec<Stmt>;
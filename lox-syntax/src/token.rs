use std::fmt::Display;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Import,

    // Other.
    Eof,
    UnterminatedString,
    Unknown(char),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Import,

    // Other.
    Eof,
    UnterminatedString,
    Unknown,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind: TokenKind = self.into();
        write!(f, "{}", kind)
    }
}

impl From<&crate::position::WithSpan<Token>> for TokenKind {
    fn from(token_with_span: &crate::position::WithSpan<Token>) -> Self {
        TokenKind::from(&token_with_span.value)
    }
}

impl From<&Token> for TokenKind {
    fn from(token: &Token) -> Self {
        match token {
            Token::LeftParen => TokenKind::LeftParen,
            Token::RightParen => TokenKind::RightParen,
            Token::LeftBrace => TokenKind::LeftBrace,
            Token::RightBrace => TokenKind::RightBrace,
            Token::Comma => TokenKind::Comma,
            Token::Dot => TokenKind::Dot,
            Token::Minus => TokenKind::Minus,
            Token::Plus => TokenKind::Plus,
            Token::Semicolon => TokenKind::Semicolon,
            Token::Slash => TokenKind::Slash,
            Token::Star => TokenKind::Star,
            Token::Bang => TokenKind::Bang,
            Token::BangEqual => TokenKind::BangEqual,
            Token::Equal => TokenKind::Equal,
            Token::EqualEqual => TokenKind::EqualEqual,
            Token::Greater => TokenKind::Greater,
            Token::GreaterEqual => TokenKind::GreaterEqual,
            Token::Less => TokenKind::Less,
            Token::LessEqual => TokenKind::LessEqual,
            Token::Identifier(_) => TokenKind::Identifier,
            Token::String(_) => TokenKind::String,
            Token::Number(_) => TokenKind::Number,
            Token::And => TokenKind::And,
            Token::Class => TokenKind::Class,
            Token::Else => TokenKind::Else,
            Token::False => TokenKind::False,
            Token::Fun => TokenKind::Fun,
            Token::For => TokenKind::For,
            Token::If => TokenKind::If,
            Token::Nil => TokenKind::Nil,
            Token::Or => TokenKind::Or,
            Token::Print => TokenKind::Print,
            Token::Return => TokenKind::Return,
            Token::Super => TokenKind::Super,
            Token::This => TokenKind::This,
            Token::True => TokenKind::True,
            Token::Var => TokenKind::Var,
            Token::While => TokenKind::While,
            Token::Import => TokenKind::Import,
            Token::Eof => TokenKind::Eof,
            Token::UnterminatedString => TokenKind::UnterminatedString,
            Token::Unknown(_) => TokenKind::Unknown,
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            TokenKind::LeftParen => "')'",
            TokenKind::RightParen => "')'",
            TokenKind::LeftBrace => "'{'",
            TokenKind::RightBrace => "'}'",
            TokenKind::Comma => "','",
            TokenKind::Dot => "'.'",
            TokenKind::Minus => "'-'",
            TokenKind::Plus => "'+'",
            TokenKind::Semicolon => "';'",
            TokenKind::Slash => "'/'",
            TokenKind::Star => "'*'",
            TokenKind::Bang => "'!'",
            TokenKind::BangEqual => "'!='",
            TokenKind::Equal => "'='",
            TokenKind::EqualEqual => "'=='",
            TokenKind::Greater => "'>'",
            TokenKind::GreaterEqual => "'>='",
            TokenKind::Less => "'<'",
            TokenKind::LessEqual => "'<='",
            TokenKind::Identifier => "identifier",
            TokenKind::String => "string",
            TokenKind::Number => "number",
            TokenKind::And => "'and'",
            TokenKind::Class => "'class'",
            TokenKind::Else => "'else'",
            TokenKind::False => "'false'",
            TokenKind::Fun => "'fun'",
            TokenKind::For => "'for'",
            TokenKind::If => "'if'",
            TokenKind::Nil => "nil",
            TokenKind::Or => "'or'",
            TokenKind::Print => "'print'",
            TokenKind::Return => "'return'",
            TokenKind::Super => "'super'",
            TokenKind::This => "'this'",
            TokenKind::True => "'true'",
            TokenKind::Var => "'var'",
            TokenKind::While => "'while'",
            TokenKind::Import => "'import'",
            TokenKind::Eof => "<EOF>",
            TokenKind::UnterminatedString => "<Unterminated String>",
            TokenKind::Unknown => "<Unknown>",
        })
    }
}
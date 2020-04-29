#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct BytePos(pub u32); //TODO Make non-public

impl BytePos {
    pub fn shift(self, ch: char) -> Self {
        BytePos(self.0 + ch.len_utf8() as u32)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position { line: 1, column: 1, }
    }
}

impl Position {
    fn increment_column(&mut self) {
        self.column += 1;
    }

    fn increment_line(&mut self) {
        self.column = 1;
        self.line += 1;
    }

    pub fn shift(mut self, ch: char) -> Self {
        if ch == '\n' {
            self.increment_line();
        } else {
            self.increment_column();
        }
        self
    }

    pub fn unshift(mut self) -> Self {
        self.column -= 1;
        if self.column <= 0 { panic!("Cannot unshift onto previous line"); }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

impl Span {
    pub fn union(a: Self, b: Self) -> Self {
        use std::cmp;

        Span {
            start: cmp::min(a.start, b.start),
            end: cmp::max(a.end, b.end),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct WithSpan<T> {
    pub value: T,
    pub span: Span,
}

impl<T> WithSpan<T> {
    pub fn new(value: T, span: Span) -> Self {
        WithSpan { value, span }
    }

    pub fn as_ref(&self) -> WithSpan<&T> {
        WithSpan {
            span: self.span,
            value: &self.value,
        }
    }
}
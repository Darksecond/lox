
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct BytePos(pub u32);

impl BytePos {
    pub fn shift(self, ch: char) -> Self {
        BytePos(self.0 + ch.len_utf8() as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

impl Span {
    pub unsafe fn new_unchecked(start: u32, end: u32) -> Self {
        Span {
            start: BytePos(start),
            end: BytePos(end),
        }
    }

    pub const fn empty() -> Self {
        Span {
            start: BytePos(0),
            end: BytePos(0),
        }
    }

    pub fn union_span(a: Self, b: Self) -> Self {
        use std::cmp;

        Span {
            start: cmp::min(a.start, b.start),
            end: cmp::max(a.end, b.end),
        }
    }

    pub fn union<A, B>(a: &WithSpan<A>, b: &WithSpan<B>) -> Self {
        Self::union_span(a.into(), b.into())
    }
}

impl<T> From<WithSpan<T>> for Span {
    fn from(with_span: WithSpan<T>) -> Span {
        with_span.span
    }
}

impl<T> From<&WithSpan<T>> for Span {
    fn from(with_span: &WithSpan<T>) -> Span {
        with_span.span
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct WithSpan<T> {
    pub value: T,
    pub span: Span,
}

impl<T> WithSpan<T> {
    pub const fn new(value: T, span: Span) -> Self {
        WithSpan { value, span }
    }

    pub const fn empty(value: T) -> Self {
        Self {
            value,
            span: Span {
                start: BytePos(0),
                end: BytePos(0),
            },
        }
    }

    pub const unsafe fn new_unchecked(value: T, start: u32, end: u32) -> Self {
        Self {
            value,
            span: Span {
                start: BytePos(start),
                end: BytePos(end),
            },
        }
    }

    //TODO Move to AsRef trait impl?
    pub const fn as_ref(&self) -> WithSpan<&T> {
        WithSpan {
            span: self.span,
            value: &self.value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub span: Span,
    pub message: String,
}

pub struct LineOffsets {
    offsets: Vec<u32>,
    len: u32,
}

/// Helper struct to convert BytePos into line numbers.
/// 
/// # Examples
/// ```
/// use lox_syntax::position::{LineOffsets, BytePos};
/// let offsets = LineOffsets::new("abc\ndef");
/// assert_eq!(offsets.line(BytePos(0)), 1);
/// assert_eq!(offsets.line(BytePos(1)), 1);
/// assert_eq!(offsets.line(BytePos(4)), 2);
/// assert_eq!(offsets.line(BytePos(3)), 1);
/// assert_eq!(offsets.line(BytePos(7)), 2);
/// ```
impl LineOffsets {
    pub fn new(data: &str) -> Self {
        let mut offsets = vec![0];
        let len = data.len() as u32;

        for (i, val) in data.bytes().enumerate() {
            if val == b'\n' {
                offsets.push((i+1) as u32);
            }
        }

        Self {
            offsets,
            len,
        }
    }

    pub fn line(&self, pos: BytePos) -> usize {
        let offset = pos.0;

        assert!(offset <= self.len);

        match self.offsets.binary_search(&offset) {
            Ok(line) => line + 1,
            Err(line) => line,
        }
    }
}
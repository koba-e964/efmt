use crate::format::{self, Format, Formatter};
use crate::parse::{self, Parse, TokenStream};
use crate::span::{Position, Span};
use erl_tokenize::values::{Keyword, Symbol};

// Note that the `Parse` trait for `Token` is implemented in the `parse` module.
#[derive(Debug, Clone, Span, Format)]
pub enum Token {
    Atom(AtomToken),
    Char(CharToken),
    Float(FloatToken),
    Integer(IntegerToken),
    Keyword(KeywordToken),
    String(StringToken),
    Symbol(SymbolToken),
    Variable(VariableToken),
}

impl Token {
    pub fn set_span(&mut self, span: &impl Span) {
        let (start, end) = match self {
            Self::Atom(x) => (&mut x.start, &mut x.end),
            Self::Char(x) => (&mut x.start, &mut x.end),
            Self::Float(x) => (&mut x.start, &mut x.end),
            Self::Integer(x) => (&mut x.start, &mut x.end),
            Self::Keyword(x) => (&mut x.start, &mut x.end),
            Self::String(x) => (&mut x.start, &mut x.end),
            Self::Symbol(x) => (&mut x.start, &mut x.end),
            Self::Variable(x) => (&mut x.start, &mut x.end),
        };
        *start = span.start_position();
        *end = span.end_position();
    }
}

impl Parse for Token {
    fn parse(ts: &mut TokenStream) -> parse::Result<Self> {
        if let Some(token) = ts.next().transpose()? {
            Ok(token)
        } else {
            let position = ts.current_position();
            Err(parse::Error::UnexpectedEof { position })
        }
    }
}

macro_rules! impl_traits {
    ($name:ident, $variant:ident, $is_primitive:expr) => {
        impl Span for $name {
            fn start_position(&self) -> Position {
                self.start
            }

            fn end_position(&self) -> Position {
                self.end
            }
        }

        impl Parse for $name {
            fn parse(ts: &mut TokenStream) -> parse::Result<Self> {
                match ts.parse()? {
                    Token::$variant(token) => Ok(token),
                    token => Err(parse::Error::unexpected_token(ts, token)),
                }
            }
        }

        impl Format for $name {
            fn format(&self, fmt: &mut Formatter) -> format::Result<()> {
                fmt.write_text(self)
            }

            fn is_primitive(&self) -> bool {
                $is_primitive
            }
        }

        impl From<$name> for Token {
            fn from(x: $name) -> Self {
                Self::$variant(x)
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct AtomToken {
    value: String,
    start: Position,
    end: Position,
}

impl AtomToken {
    pub fn new(value: &str, start: Position, end: Position) -> Self {
        Self {
            value: value.to_owned(),
            start,
            end,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl_traits!(AtomToken, Atom, true);

#[derive(Debug, Clone)]
pub struct CharToken {
    start: Position,
    end: Position,
}

impl CharToken {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl_traits!(CharToken, Char, true);

#[derive(Debug, Clone)]
pub struct FloatToken {
    start: Position,
    end: Position,
}

impl FloatToken {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl_traits!(FloatToken, Float, true);

#[derive(Debug, Clone)]
pub struct IntegerToken {
    start: Position,
    end: Position,
}

impl IntegerToken {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl_traits!(IntegerToken, Integer, true);

#[derive(Debug, Clone)]
pub struct KeywordToken {
    value: Keyword,
    start: Position,
    end: Position,
}

impl KeywordToken {
    pub fn new(value: Keyword, start: Position, end: Position) -> Self {
        Self { value, start, end }
    }

    pub fn value(&self) -> Keyword {
        self.value
    }
}

impl_traits!(KeywordToken, Keyword, false);

#[derive(Debug, Clone)]
pub struct StringToken {
    value: String,
    start: Position,
    end: Position,
}

impl StringToken {
    pub fn new(value: &str, start: Position, end: Position) -> Self {
        Self {
            value: value.to_owned(),
            start,
            end,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl_traits!(StringToken, String, true);

#[derive(Debug, Clone)]
pub struct SymbolToken {
    value: Symbol,
    start: Position,
    end: Position,
}

impl SymbolToken {
    pub fn new(value: Symbol, start: Position, end: Position) -> Self {
        Self { value, start, end }
    }

    pub fn value(&self) -> Symbol {
        self.value
    }
}

impl_traits!(SymbolToken, Symbol, false);

#[derive(Debug, Clone)]
pub struct VariableToken {
    value: String,
    start: Position,
    end: Position,
}

impl VariableToken {
    pub fn new(value: &str, start: Position, end: Position) -> Self {
        Self {
            value: value.to_owned(),
            start,
            end,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl_traits!(VariableToken, Variable, true);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentKind {
    Post,
    Trailing,
}

#[derive(Debug, Clone)]
pub struct CommentToken {
    start: Position,
    end: Position,
    kind: CommentKind,
}

impl CommentToken {
    pub fn new(kind: CommentKind, start: Position, end: Position) -> Self {
        Self { start, end, kind }
    }

    pub fn kind(&self) -> CommentKind {
        self.kind
    }
}

impl Span for CommentToken {
    fn start_position(&self) -> Position {
        self.start
    }

    fn end_position(&self) -> Position {
        self.end
    }
}

#[derive(Debug, Clone)]
pub struct WhitespaceToken {
    start: Position,
    end: Position,
}

impl WhitespaceToken {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl Span for WhitespaceToken {
    fn start_position(&self) -> Position {
        self.start
    }

    fn end_position(&self) -> Position {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_text;

    #[test]
    fn atom_works() {
        assert!(matches!(parse_text("foo").unwrap(), Token::Atom(_)));
    }

    #[test]
    fn char_works() {
        assert!(matches!(parse_text("$a").unwrap(), Token::Char(_)));
    }

    #[test]
    fn float_works() {
        assert!(matches!(parse_text("12.3").unwrap(), Token::Float(_)));
    }

    #[test]
    fn integer_works() {
        assert!(matches!(parse_text("12").unwrap(), Token::Integer(_)));
    }

    #[test]
    fn keyword_works() {
        assert!(matches!(parse_text("case").unwrap(), Token::Keyword(_)));
    }

    #[test]
    fn string_works() {
        assert!(matches!(parse_text("\"foo\"").unwrap(), Token::String(_)));
    }

    #[test]
    fn symbol_works() {
        assert!(matches!(parse_text("-").unwrap(), Token::Symbol(_)));
    }

    #[test]
    fn variable_works() {
        assert!(matches!(parse_text("Foo").unwrap(), Token::Variable(_)));
    }
}

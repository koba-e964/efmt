use erl_tokenize::Position;

pub use erl_tokenize::tokens::{
    AtomToken, CharToken, CommentToken, FloatToken, IntegerToken, KeywordToken, StringToken,
    SymbolToken, VariableToken, WhitespaceToken,
};
pub use erl_tokenize::values::{Keyword, Symbol};
pub use erl_tokenize::{LexicalToken, Token};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TokenPosition {
    token_index: usize, // TODO: delete
    text_position: Position,
}

impl TokenPosition {
    pub fn new(token_index: usize, text_position: Position) -> Self {
        Self {
            token_index,
            text_position,
        }
    }

    pub fn token_index(&self) -> usize {
        self.token_index
    }

    pub fn text_position(&self) -> Position {
        self.text_position.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenRegion {
    start: TokenPosition,
    end: TokenPosition,
}

impl TokenRegion {
    pub fn new(start: TokenPosition, end: TokenPosition) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> &TokenPosition {
        &self.start
    }

    pub fn end(&self) -> &TokenPosition {
        &self.end
    }
}

pub trait Region {
    fn region(&self) -> &TokenRegion;
}

impl Region for TokenRegion {
    fn region(&self) -> &TokenRegion {
        self
    }
}

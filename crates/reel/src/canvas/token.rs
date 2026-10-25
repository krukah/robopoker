use crate::*;

/// What one chance reveal looks like: a white tile of inked fragments —
/// a card face for poker, a bare rank for Kuhn.
#[derive(Debug, Clone, PartialEq)]
pub struct Token(Vec<Rune>);

impl Token {
    pub fn new(runes: Vec<Rune>) -> Self {
        Self(runes)
    }

    pub fn runes(&self) -> &[Rune] {
        &self.0
    }
}

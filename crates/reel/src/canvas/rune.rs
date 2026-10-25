use crate::*;

/// One inked fragment of a chance token: a rank, a suit glyph, a Kuhn card —
/// text plus its ink class and font role.
#[derive(Debug, Clone, PartialEq)]
pub struct Rune {
    text: String,
    tint: Tint,
    face: Face,
}

impl Rune {
    pub fn new(text: impl Into<String>, tint: Tint, face: Face) -> Self {
        Self {
            text: text.into(),
            tint,
            face,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn tint(&self) -> Tint {
        self.tint
    }

    pub fn face(&self) -> Face {
        self.face
    }
}

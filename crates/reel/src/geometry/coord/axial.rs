use crate::*;

/// A point in abstract (layer, slot) space — everything upstream of
/// [`crate::Orient`] lives here, and only the projection turns it
/// into screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axial {
    along: Along,
    cross: Cross,
}

impl Axial {
    pub fn new(along: Along, cross: Cross) -> Self {
        Self { along, cross }
    }

    pub fn along(&self) -> Along {
        self.along
    }

    pub fn cross(&self) -> Cross {
        self.cross
    }
}

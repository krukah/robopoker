use pokerkit::*;

/// Button-anchored set of players still contesting the pot.
///
/// Bit `i` is set iff the player at offset `i` from the dealer button has not
/// folded. Anchoring at the button makes the set rotation-invariant — the same
/// relative configuration maps to one `Live` regardless of absolute seats — and
/// folded players simply vanish from the mask.
#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Live(u16);

impl Live {
    /// Number of players still live.
    pub const fn count(&self) -> usize {
        self.0.count_ones() as usize
    }
    /// Whether the player at `offset` from the button is live.
    pub const fn contains(&self, offset: Position) -> bool {
        self.0 & (1 << offset) != 0
    }
    /// Live offsets from the button, ascending (button = 0).
    pub fn seats(&self) -> impl Iterator<Item = Position> + '_ {
        (0..u16::BITS as Position).filter(|&offset| self.contains(offset))
    }
}

impl FromIterator<Position> for Live {
    fn from_iter<T>(offsets: T) -> Self
    where
        T: IntoIterator<Item = Position>,
    {
        Self(offsets.into_iter().fold(0u16, |mask, offset| mask | (1 << offset)))
    }
}

impl From<Live> for u16 {
    fn from(live: Live) -> Self {
        live.0
    }
}

impl From<u16> for Live {
    fn from(mask: u16) -> Self {
        Self(mask)
    }
}

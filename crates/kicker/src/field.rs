use super::*;
use pokerkit::*;

/// Number of `Field` payload slots, derived purely from the compile-time
/// [`pokerkit::N`]: **0 at heads-up, 1 multiway**. Because the field is a
/// fixed-size array of this length, at N=2 `Field` is a zero-sized type — the
/// info-set key and every encounters-map entry pay nothing for multiway state
/// the heads-up build can never use. No `cfg`, no per-crate feature: the type
/// specializes on `N` itself, so it can't drift out of sync with the build.
const SLOTS: usize = (N > 2) as usize;

/// The field at a decision node: the live set plus whose turn it is, both
/// anchored at the dealer button. `hero` is always a member of `live`.
///
/// **Build-specialized on player count via `SLOTS`.** At heads-up the live
/// set is invariant — any fold ends the hand — so the field carries no
/// information and is a zero-sized `[_; 0]`: an N=2 info-set key is
/// byte-identical to a build with no multiway support. Multiway it holds one
/// `(live, hero)` payload — the cross-street state the current-street `Path`
/// drops on every `Draw`, separating a 3-way flop from a 4-way flop and, with
/// the button, pinning down the acting position.
#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Field([(Live, Position); SLOTS]);

const _: () = assert!(N > 2 || std::mem::size_of::<Field>() == 0, "heads-up Field must be zero-sized");

impl Field {
    /// Constructs a field. Multiway asserts the acting player is live; at
    /// heads-up the payload is discarded (zero slots).
    pub const fn new(live: Live, hero: Position) -> Self {
        debug_assert!(SLOTS == 0 || live.contains(hero), "hero must be live to act");
        Self([(live, hero); SLOTS])
    }
    /// The button-anchored live set (all players heads-up).
    pub fn live(&self) -> Live {
        self.0.first().map_or_else(|| (0..N).collect(), |&(live, _)| live)
    }
    /// The acting player's offset from the button (0 heads-up).
    pub fn hero(&self) -> Position {
        self.0.first().map_or(0, |&(_, hero)| hero)
    }
    /// Number of players still live.
    pub fn count(&self) -> usize {
        self.0.first().map_or(N, |&(live, _)| live.count())
    }
}

impl From<Field> for i32 {
    fn from(field: Field) -> Self {
        field
            .0
            .first()
            .map_or(0, |&(live, hero)| (hero as i32) << 16 | i32::from(u16::from(live)))
    }
}

impl From<i32> for Field {
    fn from(bits: i32) -> Self {
        Self::new(Live::from(bits as u16), (bits >> 16) as Position)
    }
}

// SQL codec: an `INT` column via the packed i32 representation.
#[cfg(feature = "sql")]
pokerkit::codec!(Field as i32, |f: &Field| i32::from(*f), Field::from);

impl<const P: usize> GameN<P> {
    /// The button-anchored [`Field`] at the current decision node.
    ///
    /// Meaningful only at a `Turn::Choice` node — the acting seat pins `hero`.
    /// At a chance/terminal node (no actor) `hero` falls back to the lowest
    /// live seat, so the invariant `hero ∈ live` holds and callers on a
    /// degenerate node never panic; such fields are never used for lookup.
    /// At heads-up the payload is zero-sized, so the compiler folds this to a
    /// bare default with no work.
    pub fn field(&self) -> Field {
        if SLOTS == 0 {
            return Field::default();
        }
        let dealer = self.dealer().position();
        let live = self
            .seats()
            .into_iter()
            .enumerate()
            .filter(|(_, seat)| seat.state() != State::Folding)
            .map(|(seat, _)| (seat + P - dealer) % P)
            .collect::<Live>();
        let hero = match self.turn() {
            Turn::Choice(seat) => (seat + P - dealer) % P,
            _ => live.seats().next().unwrap_or_default(),
        };
        Field::new(live, hero)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_roundtrips_u16() {
        let live = [0, 2, 5].into_iter().collect::<Live>();
        assert_eq!(live, Live::from(u16::from(live)));
        assert_eq!(live.count(), 3);
        assert!(live.contains(0));
        assert!(!live.contains(1));
        assert!(live.contains(5));
    }

    #[test]
    fn live_seats_ascending() {
        let live = [5, 0, 2].into_iter().collect::<Live>();
        assert_eq!(live.seats().collect::<Vec<_>>(), vec![0, 2, 5]);
    }

    #[test]
    fn field_roundtrips_i32() {
        // Heads-up build: Field is a ZST and always packs to 0.
        if N == 2 {
            assert_eq!(i32::from(Field::default()), 0);
            assert_eq!(std::mem::size_of::<Field>(), 0);
            return;
        }
        let field = Field::new([0, 3, 4].into_iter().collect::<Live>(), 3);
        assert_eq!(field, Field::from(i32::from(field)));
        assert_eq!(field.hero(), 3);
        assert_eq!(field.count(), 3);
    }

    #[test]
    fn field_multiway_absorbs_folds() {
        // Field payload only exists in a multiway build; heads-up it is a ZST.
        if N == 2 {
            return;
        }
        let game = FunTable::root();
        let dealer = game.dealer().position();
        let offset = (game.turn().position() + 6 - dealer) % 6;
        let field = game.apply(Action::Fold).field();
        assert_eq!(field.count(), 5);
        assert!(!field.live().contains(offset), "folded seat absorbed");
    }
}

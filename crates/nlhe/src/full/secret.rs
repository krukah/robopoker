use deuce::*;
use kicker::*;
use mccfr::*;
use monge::Support;
use neural::*;
use rosetta::Feature;

/// The actor's cards in deal order, [canonical](Arrangement::canonical). A newtype only because
/// `mccfr` and `kicker` own the trait and the type.
///
/// The board sits here, not in [`FullPublic`](crate::FullPublic), because its canonical suits
/// depend on the hole cards.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FullSecret(Arrangement);

impl FullSecret {
    /// `cards` must already be [canonical](Arrangement::canonical).
    pub fn new(cards: Arrangement) -> Self {
        Self(cards)
    }

    pub fn cards(&self) -> Arrangement {
        self.0
    }
}

impl Support for FullSecret {}
impl CfrSecret for FullSecret {}

/// Each street's cards three ways (as a 52-bit set, counted by rank, counted by suit), then
/// `rosetta`'s predicates: made hand against the board, kicker, draws, hole shape, texture.
impl Block for FullSecret {
    const WIDTH: usize = 4 * (Self::DECK + Self::RANKS + Self::SUITS) as usize + Self::PREDICATES;

    fn values(self) -> impl Iterator<Item = f32> {
        self.deck()
            .chain(self.ranks())
            .chain(self.suits())
            .chain(self.predicates())
    }
}

impl FullSecret {
    const DECK: u8 = 52;
    const RANKS: u8 = 13;
    const SUITS: u8 = 4;
    const PREDICATES: usize =
        Feature::MADE.len() + Feature::KICKS.len() + Feature::DRAWS.len() + Feature::SHAPE.len() + Feature::BOARD.len();

    fn deck(self) -> impl Iterator<Item = f32> {
        self.0
            .streets()
            .into_iter()
            .flat_map(|hand| (0..Self::DECK).map(move |card| f32::from(hand.contains(&Card::from(card)))))
    }

    fn ranks(self) -> impl Iterator<Item = f32> {
        self.0.streets().into_iter().flat_map(|hand| {
            (0..Self::RANKS).map(move |rank| hand.filter(|card| u8::from(card.rank()) == rank).count() as f32 / 4.)
        })
    }

    fn suits(self) -> impl Iterator<Item = f32> {
        self.0
            .streets()
            .into_iter()
            .flat_map(|hand| Suit::all().map(|suit| hand.of(&suit).size() as f32 / 4.))
    }

    fn predicates(self) -> impl Iterator<Item = f32> {
        let features = Feature::of(&self.0.observation());
        Feature::all()
            .into_iter()
            .map(move |feature| f32::from(features.contains(&feature)))
    }
}

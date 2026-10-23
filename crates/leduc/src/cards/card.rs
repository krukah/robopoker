use crate::*;
/// A card in the 6-card Leduc deck: {J, Q, K} x {Spades, Hearts}.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    pub const ALL: [Card; 6] = [
        Card {
            rank: Rank::J,
            suit: Suit::Spades,
        },
        Card {
            rank: Rank::J,
            suit: Suit::Hearts,
        },
        Card {
            rank: Rank::Q,
            suit: Suit::Spades,
        },
        Card {
            rank: Rank::Q,
            suit: Suit::Hearts,
        },
        Card {
            rank: Rank::K,
            suit: Suit::Spades,
        },
        Card {
            rank: Rank::K,
            suit: Suit::Hearts,
        },
    ];
    pub fn rank(self) -> Rank {
        self.rank
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.rank(),
            match self.suit() {
                Suit::Spades => 0,
                Suit::Hearts => 1,
            }
        )
    }
}

impl Card {
    pub fn suit(self) -> Suit {
        self.suit
    }
}

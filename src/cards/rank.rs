use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Rank {
    #[default]
    Two = 0,
    Three = 1,
    Four = 2,
    Five = 3,
    Six = 4,
    Seven = 5,
    Eight = 6,
    Nine = 7,
    Ten = 8,
    Jack = 9,
    Queen = 10,
    King = 11,
    Ace = 12,
}

impl Rank {
    pub fn mask(n: u32) -> u32 {
        n & 0b00000000000000000001111111111111
    }
    pub const MAX: Self = Rank::Ace;
    pub const MIN: Self = Rank::Two;
}

// u8 isomorphism
impl From<u8> for Rank {
    fn from(n: u8) -> Rank {
        match n {
            0 => Rank::Two,
            1 => Rank::Three,
            2 => Rank::Four,
            3 => Rank::Five,
            4 => Rank::Six,
            5 => Rank::Seven,
            6 => Rank::Eight,
            7 => Rank::Nine,
            8 => Rank::Ten,
            9 => Rank::Jack,
            10 => Rank::Queen,
            11 => Rank::King,
            12 => Rank::Ace,
            _ => panic!("Invalid rank"),
        }
    }
}
impl From<Rank> for u8 {
    fn from(r: Rank) -> u8 {
        r as u8
    }
}

/// u32 isomorphism.
/// with this we get the highest rank in a union of cards in u32 representation
/// xxxxxxxxxxxxxxx xxxx 0000Txxxxxxxx
impl From<u32> for Rank {
    fn from(n: u32) -> Rank {
        let msb = (32 - Rank::mask(n).leading_zeros() - 1) as u8;
        Rank::from(msb)
    }
}
impl From<Rank> for u32 {
    fn from(r: Rank) -> u32 {
        1 << u8::from(r)
    }
}
impl Display for Rank {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match self {
                Rank::Two => "2",
                Rank::Three => "3",
                Rank::Four => "4",
                Rank::Five => "5",
                Rank::Six => "6",
                Rank::Seven => "7",
                Rank::Eight => "8",
                Rank::Nine => "9",
                Rank::Ten => "T",
                Rank::Jack => "J",
                Rank::Queen => "Q",
                Rank::King => "K",
                Rank::Ace => "A",
            }
        )
    }
}

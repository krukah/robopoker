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

/// u8 isomorphism
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

/// u16 isomorphism
///
/// With 13 ranks we only need 13 bits
impl From<u16> for Rank {
    fn from(n: u16) -> Rank {
        const MASK: u16 = 0b1111111111111;
        let msb = (16 - 1 - (n & MASK).leading_zeros()) as u8;
        Rank::from(msb)
    }
}
impl From<Rank> for u16 {
    fn from(r: Rank) -> u16 {
        1 << u8::from(r)
    }
}

impl std::fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

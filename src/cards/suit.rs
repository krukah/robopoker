#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    #[default]
    Club = 0,
    Diamond = 1,
    Heart = 2,
    Spade = 3,
}

impl Suit {
    pub const MAX: Self = Suit::Spade;
    pub const MIN: Self = Suit::Club;
}

impl From<u8> for Suit {
    fn from(n: u8) -> Suit {
        match n {
            0 => Suit::Club,
            1 => Suit::Diamond,
            2 => Suit::Heart,
            3 => Suit::Spade,
            _ => panic!("Invalid suit"),
        }
    }
}
impl From<Suit> for u8 {
    fn from(s: Suit) -> u8 {
        s as u8
    }
}

// xxxxxxxxxxxxxxx cdhs xxxxxxxxxxxxx
impl From<u32> for Suit {
    fn from(n: u32) -> Suit {
        Suit::from((n >> 13).trailing_zeros() as u8)
    }
}
impl From<Suit> for u32 {
    fn from(s: Suit) -> u32 {
        1 << (13 + u8::from(s))
    }
}

impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Suit::Club => "c",
                Suit::Diamond => "d",
                Suit::Heart => "h",
                Suit::Spade => "s",
            }
        )
    }
}

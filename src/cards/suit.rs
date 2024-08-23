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

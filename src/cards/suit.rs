#[derive(Debug, Clone, Copy)]
pub enum Suit {
    Club = 0,
    Diamond = 1,
    Heart = 2,
    Spade = 3,
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

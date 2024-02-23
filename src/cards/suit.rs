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
impl Display for Suit {
    fn fmt(&self, f: &mut Formatter) -> Result {
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
use std::fmt::{Display, Formatter, Result};

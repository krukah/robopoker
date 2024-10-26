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
    const fn mask() -> u16 {
        0b1111111111111
    }
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
            _ => panic!("Invalid rank u8: {}", n),
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
        let msb = (16 - 1 - (n & Self::mask()).leading_zeros()) as u8;
        Rank::from(msb)
    }
}
impl From<Rank> for u16 {
    fn from(r: Rank) -> u16 {
        1 << u8::from(r)
    }
}

/// u64 injection
impl From<Rank> for u64 {
    fn from(r: Rank) -> u64 {
        0xF << (u8::from(r) * 4)
    }
}

/// str isomorphism
impl From<&str> for Rank {
    fn from(s: &str) -> Self {
        match s {
            "2" => Rank::Two,
            "3" => Rank::Three,
            "4" => Rank::Four,
            "5" => Rank::Five,
            "6" => Rank::Six,
            "7" => Rank::Seven,
            "8" => Rank::Eight,
            "9" => Rank::Nine,
            "T" => Rank::Ten,
            "J" => Rank::Jack,
            "Q" => Rank::Queen,
            "K" => Rank::King,
            "A" => Rank::Ace,
            _ => panic!("Invalid rank str: {}", s),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bijective_u8() {
        let rank = Rank::Five;
        assert!(rank == Rank::from(u8::from(rank)));
    }

    #[test]
    fn bijective_u16() {
        let rank = Rank::Five;
        assert!(rank == Rank::from(u16::from(rank)));
    }

    #[test]
    fn injective_u64() {
        assert!(u64::from(Rank::Five) == 0b1111000000000000);
    }
}

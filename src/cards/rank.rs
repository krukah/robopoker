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
    pub fn lo(bits: u64) -> Self {
        let rank = (00 + 0 + bits.trailing_zeros()) as u8 / 4;
        Rank::from(rank)
    }
    pub fn hi(bits: u64) -> Self {
        let rank = (64 - 1 - bits.leading_zeros()) as u8 / 4;
        Rank::from(rank)
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
impl TryFrom<&str> for Rank {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.trim().to_uppercase().as_str() {
            "2" => Ok(Rank::Two),
            "3" => Ok(Rank::Three),
            "4" => Ok(Rank::Four),
            "5" => Ok(Rank::Five),
            "6" => Ok(Rank::Six),
            "7" => Ok(Rank::Seven),
            "8" => Ok(Rank::Eight),
            "9" => Ok(Rank::Nine),
            "T" => Ok(Rank::Ten),
            "J" => Ok(Rank::Jack),
            "Q" => Ok(Rank::Queen),
            "K" => Ok(Rank::King),
            "A" => Ok(Rank::Ace),
            _ => Err(format!("invalid rank str: {}", s)),
        }
    }
}

impl std::fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Rank::Two => write!(f, "2"),
            Rank::Three => write!(f, "3"),
            Rank::Four => write!(f, "4"),
            Rank::Five => write!(f, "5"),
            Rank::Six => write!(f, "6"),
            Rank::Seven => write!(f, "7"),
            Rank::Eight => write!(f, "8"),
            Rank::Nine => write!(f, "9"),
            Rank::Ten => write!(f, "T"),
            Rank::Jack => write!(f, "J"),
            Rank::Queen => write!(f, "Q"),
            Rank::King => write!(f, "K"),
            Rank::Ace => write!(f, "A"),
        }
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

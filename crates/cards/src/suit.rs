/// Card suit: clubs, diamonds, hearts, spades.
///
/// Suits are strategically interchangeable in poker — only the pattern of
/// suit matches matters, not which specific suits are involved. This symmetry
/// is exploited by [`Permutation`] to reduce the abstraction space.
///
/// The ordering (C < D < H < S) is arbitrary but consistent, used for
/// canonical form selection during isomorphism computation.
///
/// [`Permutation`]: super::permutation::Permutation
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    #[default]
    C = 0,
    D = 1,
    H = 2,
    S = 3,
}

impl Suit {
    /// All four suits in canonical order.
    pub const fn all() -> [Suit; 4] {
        [Suit::C, Suit::D, Suit::H, Suit::S]
    }
    /// Unicode suit symbol for display.
    pub fn ascii(&self) -> char {
        match self {
            Suit::C => '♣',
            Suit::D => '♦',
            Suit::H => '♥',
            Suit::S => '♠',
        }
    }
}

/// u8 isomorphism
impl From<u8> for Suit {
    fn from(n: u8) -> Suit {
        match n {
            0 => Suit::C,
            1 => Suit::D,
            2 => Suit::H,
            3 => Suit::S,
            _ => unreachable!("invalid suit"),
        }
    }
}
impl From<Suit> for u8 {
    fn from(s: Suit) -> u8 {
        s as u8
    }
}

/// u64 representation
impl From<Suit> for u64 {
    fn from(s: Suit) -> u64 {
        match s {
            Suit::C => 0x0001111111111111,
            Suit::D => 0x0002222222222222,
            Suit::H => 0x0004444444444444,
            Suit::S => 0x0008888888888888,
        }
    }
}

/// str isomorphism
impl TryFrom<&str> for Suit {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.trim().to_lowercase().as_str() {
            "c" | "♣" => Ok(Suit::C),
            "d" | "♦" => Ok(Suit::D),
            "h" | "♥" => Ok(Suit::H),
            "s" | "♠" => Ok(Suit::S),
            _ => Err(format!("invalid suit str: {}", s)),
        }
    }
}

impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Suit::C => write!(f, "c"),
            Suit::D => write!(f, "d"),
            Suit::H => write!(f, "h"),
            Suit::S => write!(f, "s"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bijective_u8() {
        let suit = Suit::D;
        assert!(suit == Suit::from(u8::from(suit)));
    }

    #[test]
    fn injective_u64() {
        assert!(u64::from(Suit::C) == 0b0001000100010001000100010001000100010001000100010001);
        assert!(u64::from(Suit::D) == 0b0010001000100010001000100010001000100010001000100010);
        assert!(u64::from(Suit::H) == 0b0100010001000100010001000100010001000100010001000100);
        assert!(u64::from(Suit::S) == 0b1000100010001000100010001000100010001000100010001000);
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    #[default]
    C = 0,
    D = 1,
    H = 2,
    S = 3,
}

impl Suit {
    pub const fn all() -> [Suit; 4] {
        [Suit::C, Suit::D, Suit::H, Suit::S]
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
            _ => panic!("Invalid suit"),
        }
    }
}
impl From<Suit> for u8 {
    fn from(s: Suit) -> u8 {
        s as u8
    }
}

/// u64 injection
impl From<Suit> for u64 {
    fn from(s: Suit) -> u64 {
        (0..13).fold(0, |acc, _| (acc << 4) | (1 << s as u64))
    }
}

/// str isomorphism
impl From<&str> for Suit {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "c" | "♣" => Suit::C,
            "d" | "♦" => Suit::D,
            "h" | "♥" => Suit::H,
            "s" | "♠" => Suit::S,
            _ => panic!("Invalid suit string: {}", s),
        }
    }
}

impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Suit::C => "c",
                Suit::D => "d",
                Suit::H => "h",
                Suit::S => "s",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bijective_u8() {
        let suit = Suit::C;
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

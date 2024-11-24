#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Street {
    Pref,
    Flop,
    Turn,
    Rive,
}

impl Street {
    pub const fn all() -> &'static [Self] {
        &[Self::Pref, Self::Flop, Self::Turn, Self::Rive]
    }
    pub const fn next(&self) -> Self {
        match self {
            Self::Pref => Self::Flop,
            Self::Flop => Self::Turn,
            Self::Turn => Self::Rive,
            Self::Rive => panic!("terminal"),
        }
    }
    pub const fn prev(&self) -> Self {
        match self {
            Self::Pref => Self::Pref, // format!("{} <- {}", self.street.prev(), self.street)
            Self::Flop => Self::Pref,
            Self::Turn => Self::Flop,
            Self::Rive => Self::Turn,
        }
    }
    pub const fn n_observed(&self) -> usize {
        match self {
            Self::Pref => 0,
            Self::Flop => 3,
            Self::Turn => 4,
            Self::Rive => 5,
        }
    }
    pub const fn n_revealed(&self) -> usize {
        match self {
            Self::Pref => 3,
            Self::Flop => 1,
            Self::Turn => 1,
            Self::Rive => panic!("terminal"),
        }
    }
    #[cfg(not(feature = "shortdeck"))]
    pub const fn n_isomorphisms(&self) -> usize {
        match self {
            Self::Pref => 0_________630,
            Self::Flop => 0___3_769_920,
            Self::Turn => 0__29_216_880,
            Self::Rive => 0_175_301_280,
        }
    }
    #[cfg(feature = "shortdeck")]
    pub const fn n_isomorphisms(&self) -> usize {
        // TODO
        // pencil paper math, combinatorics. still learning how to count 25 years later
        match self {
            Self::Pref => 0_________630,
            Self::Flop => 0___3_769_920,
            Self::Turn => 0__29_216_880,
            Self::Rive => 0_175_301_280,
        }
    }
    #[cfg(not(feature = "shortdeck"))]
    pub const fn n_observations(&self) -> usize {
        match self {
            Self::Pref => 0_______1_326,
            Self::Flop => 0__25_989_600,
            Self::Turn => 0_305_377_800,
            Self::Rive => 2_809_475_760,
        }
    }
    #[cfg(feature = "shortdeck")]
    pub const fn n_observations(&self) -> usize {
        match self {
            Self::Pref => 0_________630,
            Self::Flop => 0___3_769_920,
            Self::Turn => 0__29_216_880,
            Self::Rive => 0_175_301_280,
        }
    }
}

impl std::fmt::Display for Street {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Pref => write!(f, "preflop"),
            Self::Flop => write!(f, "flop"),
            Self::Turn => write!(f, "turn"),
            Self::Rive => write!(f, "river"),
        }
    }
}

impl crate::Arbitrary for Street {
    fn random() -> Self {
        use rand::Rng;
        match rand::thread_rng().gen_range(0..4) {
            0 => Self::Pref,
            1 => Self::Flop,
            2 => Self::Turn,
            _ => Self::Rive,
        }
    }
}

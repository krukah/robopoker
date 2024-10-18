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
            Self::Pref => panic!("initial"),
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
    pub const fn n_observations(&self) -> usize {
        match self {
            Self::Pref => 0_______1_326,
            Self::Flop => 0__25_989_600,
            Self::Turn => 0_305_377_800,
            Self::Rive => 2_809_475_760,
        }
    }
    pub const fn n_isomorphisms(&self) -> usize {
        match self {
            Self::Pref => 0_________169,
            Self::Flop => 0___1_286_792,
            Self::Turn => 0__13_960_050, // loses reveal order information; ~4x smaller
            Self::Rive => 0_123_156_254, // loses reveal order information; ~20x smaller
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

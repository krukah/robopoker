#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Street {
    Pref,
    Flop,
    Turn,
    Rive,
}

impl Street {
    pub fn all() -> &'static [Street] {
        &[Street::Pref, Street::Flop, Street::Turn, Street::Rive]
    }
    pub fn next(&self) -> Street {
        match self {
            Street::Pref => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::Rive,
            Street::Rive => unreachable!("terminal"),
        }
    }
    pub fn prev(&self) -> Street {
        match self {
            Street::Pref => unreachable!("initial"),
            Street::Flop => Street::Pref,
            Street::Turn => Street::Flop,
            Street::Rive => Street::Turn,
        }
    }
    pub fn n_observed(&self) -> usize {
        match self {
            Street::Pref => 0,
            Street::Flop => 3,
            Street::Turn => 4,
            Street::Rive => 5,
        }
    }
    pub fn n_revealed(&self) -> usize {
        match self {
            Street::Pref => 3,
            Street::Flop => 1,
            Street::Turn => 1,
            Street::Rive => unreachable!("terminal"),
        }
    }
    pub fn n_observations(&self) -> usize {
        match self {
            Street::Pref => 0_______1_326,
            Street::Flop => 0__25_989_600,
            Street::Turn => 0_305_377_800,
            Street::Rive => 2_809_475_760,
        }
    }
    pub fn n_isomorphisms(&self) -> usize {
        match self {
            Street::Pref => 0_________169,
            Street::Flop => 0___1_286_792,
            Street::Turn => 0__55_190_538,
            Street::Rive => 2_428_287_420,
        }
    }
}

impl std::fmt::Display for Street {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Street::Pref => write!(f, "Preflop"),
            Street::Flop => write!(f, "Flop"),
            Street::Turn => write!(f, "Turn"),
            Street::Rive => write!(f, "River"),
        }
    }
}

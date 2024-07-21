#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Street {
    Pref,
    Flop,
    Turn,
    Rive,
    Show,
}

impl Street {
    #[allow(unused)]
    pub fn next(&self) -> Street {
        match self {
            Street::Pref => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::Rive,
            Street::Rive => Street::Show,
            Street::Show => panic!("no next street"),
        }
    }

    pub fn prev(&self) -> Street {
        match self {
            Street::Pref => panic!("no previous street"),
            Street::Flop => Street::Pref,
            Street::Turn => Street::Flop,
            Street::Rive => Street::Turn,
            Street::Show => Street::Rive,
        }
    }
}

impl std::fmt::Display for Street {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Street::Pref => write!(f, "Pre Flop"),
            Street::Flop => write!(f, "Flop"),
            Street::Turn => write!(f, "Turn"),
            Street::Rive => write!(f, "River"),
            Street::Show => write!(f, "Showdown"),
        }
    }
}

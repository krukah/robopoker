#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Club = 0,
    Diamond = 1,
    Heart = 2,
    Spade = 3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Suitedness {
    Suited,
    Offsuit,
    All,
    Specific(u8, u8),
}

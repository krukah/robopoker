pub enum Action {
    None,
    Fold,
    Check,
    Call,
    Bet(u32),
    Raise(u32),
    AllIn(u32),
    Deal(Card),
}

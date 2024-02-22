use crate::cards::card::Card;

pub trait Actor {
    fn act(&self) -> Action;
}

#[derive(Debug, Clone)]
pub enum Action {
    Draw(Card),
    Check,
    Fold,
    Call(u32),
    Open(u32),
    Raise(u32),
    Shove(u32),
}

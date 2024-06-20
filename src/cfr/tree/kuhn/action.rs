use crate::cards::card::Card;
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
struct Deal(Card, Card);

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum Action {
    Raise,
    Call,
    Check,
    Fold,
    Chance(Deal),
}

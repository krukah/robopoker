use crate::*;

/// Game tree node: which phase of the hand are we in?
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Node {
    Start,
    Dealt,
    R1(Spot),
    Deal(Spot),
    R2(Card, Spot, Spot),
    Over(Outcome),
}

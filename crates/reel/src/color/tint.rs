/// The ink class of a token fragment, resolved to paint by a Palette —
/// black suits vs red suits vs the terminal gold, generically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tint {
    Dark,
    Red,
    Gold,
}

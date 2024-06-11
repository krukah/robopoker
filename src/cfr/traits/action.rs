pub(crate) trait Action: Sized + Eq + Copy + std::hash::Hash {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum E {
    RK,
    PA,
    SC,
}

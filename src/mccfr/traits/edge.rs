/// the edge is fully abstracted. it is basically a marker trait
pub trait Edge:
    Copy
    + Clone
    + PartialEq
    + Eq
    + PartialOrd // can be ignored
    + Ord // can be ignored
    + Send
    + Sync
    + crate::transport::support::Support
    + std::hash::Hash // can be ignored
    + std::fmt::Debug
{
}

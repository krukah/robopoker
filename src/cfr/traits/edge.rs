/// the edge is fully abstracted. it is basically a marker trait
pub trait Edge:
    Copy + Clone + PartialEq + Eq + crate::transport::support::Support + std::fmt::Debug
{
}

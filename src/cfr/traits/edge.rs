/// the edge is fully abstracted. it is basically a marker trait
pub trait Edge:
    Copy + Clone + PartialEq + Eq + Send + Sync + crate::transport::support::Support + std::fmt::Debug
{
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Edge {
    R,
    P,
    S,
}

impl crate::transport::support::Support for Edge {}
impl crate::mccfr::traits::edge::Edge for Edge {}

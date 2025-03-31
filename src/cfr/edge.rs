/// the edge is fully abstracted. it is basically a marker trait
pub trait Edge: Copy + Clone + PartialEq + Eq + crate::transport::support::Support {}
impl Edge for crate::gameplay::action::Action {}
impl Edge for crate::gameplay::edge::Edge {}
impl crate::transport::support::Support for crate::gameplay::action::Action {}
impl crate::transport::support::Support for crate::gameplay::edge::Edge {}

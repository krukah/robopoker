use monge::Support;

/// The part of an information set visible only to the acting player — hole cards
/// in poker.
pub trait CfrSecret
where
    Self: Support,
    Self: Send + Sync,
    Self: Copy + Clone,
    Self: PartialEq + Eq,
    Self: PartialOrd + Ord,
    Self: std::fmt::Debug,
    Self: std::hash::Hash,
{
}
/// Unit secret for games with no private information.
impl CfrSecret for () {}

use rbp_transport::Support;

/// A representation of private information in an information set.
///
/// Private information is observable only by the acting player.
/// In poker, this is the player's hole cards. Different representations
/// offer different tradeoffs:
///
/// - **Exact**: Lossless but larger state space
/// - **Abstracted**: Lossy but tractable for full-game solving
///
/// # Requirements
///
/// Types implementing this trait must be:
/// - `Support` — Can serve as distribution support
/// - `Copy` + `Clone` — Cheap to duplicate
/// - `Hash` + `Eq` — Usable as hash map keys
/// - `Ord` — Sortable for deterministic iteration
/// - `Debug` — Printable for debugging
/// - `Send` + `Sync` — Safe for parallel CFR
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

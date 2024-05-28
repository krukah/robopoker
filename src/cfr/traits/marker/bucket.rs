use std::hash::Hash;

#[allow(dead_code)]
/// The hashable representation of "what you know" at a node. Used to index the regret and strategy tables, in-mem and on-disk.
pub(crate) trait Bucket: Hash + Eq {}

use std::hash::Hash;

pub(crate) trait Signature: Hash + Eq {}

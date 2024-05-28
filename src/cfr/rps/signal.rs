use crate::cfr::traits::marker::signature::Signature;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) struct RpsSignal {}

impl Signature for RpsSignal {}

use crate::cfr::training::marker::signature::Signature;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) struct RPSSignal {}

impl Signature for RPSSignal {}

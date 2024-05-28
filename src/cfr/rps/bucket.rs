use crate::cfr::traits::marker::bucket::Bucket;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) struct RpsBucket {}

impl Bucket for RpsBucket {}

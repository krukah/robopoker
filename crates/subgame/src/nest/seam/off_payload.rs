/// The marker bundle an off-tree payload needs to ride through the CFR machinery: copyable, a
/// valid hashmap/btree key, debuggable, thread-safe. For NLHE this is `Chips` (an integer). It
/// is **not** a [`mccfr::CfrEdge`] — the payload is raw data, not a game action (the action is the
/// enclosing [`NestEdge`](crate::NestEdge), which *is* a `CfrEdge`). The blanket impl means
/// every qualifying type satisfies it with nothing to write.
pub trait OffPayload: Copy + Eq + Ord + std::hash::Hash + std::fmt::Debug + Send + Sync {}

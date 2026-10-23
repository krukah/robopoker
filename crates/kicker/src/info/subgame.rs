use super::*;

/// The subgame path type: current-street action history, width pinned by the
/// compile-time player count ([`pokerkit::WORDS`]): one u64 word at heads-up
/// (byte-identical to the legacy `Path`), four words (48 edges) multiway,
/// where a single street can legally exceed the 12-edge single-word capacity.
pub type Subgame = Path<{ pokerkit::WORDS }>;

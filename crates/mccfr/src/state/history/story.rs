//! A growing descent story emitted during a biased rollout.
use crate::*;

/// A descent story growing during a biased rollout at a subgame frontier.
/// A [`Trail`] tagged [`Grown`] — distinct from the upstream solver's own
/// path so the compiler refuses to splice it in as a [`Prefix`].
pub type Story<G> = Trail<G, Grown>;

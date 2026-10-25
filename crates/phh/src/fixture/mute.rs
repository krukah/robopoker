use crate::*;
use kicker::*;

/// A strategy that says nothing, ever — every decision is a miss.
pub struct Mute;

impl Oracle for Mute {
    fn policy(&self, _: &Witness) -> Policy {
        Policy::default()
    }
}

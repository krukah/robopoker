use crate::cards::hand::Hand;
use crate::evaluation::strength::Strength;
use std::cmp::Ordering;

/// Representation of private cards
/// might optimize this into less memory
///  u16      if order does not matter
/// [Card; 2] if order matters
/// in either case, we need impl From<Hold> for Hand to preserve contract
/// this eventual mapping to Hand(u64) then feels like maybe the Hole optimization is futile
/// haven't reasoned about it enough to tell if worth it
type Hole = Hand;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Observation {
    secret: Hole,
    public: Hand,
}

impl Observation {
    /// this is only available for terminal observations
    pub fn equity(&self) -> f32 {
        let this = self.secret;
        let this = Strength::from(self.public | this);
        let theirs = self.theirs();
        let n = theirs.len();
        theirs
            .into_iter()
            .map(|that| Strength::from(self.public | that))
            .map(|that| match &this.cmp(&that) {
                Ordering::Less => 0,
                Ordering::Equal => 1,
                Ordering::Greater => 2,
            })
            .sum::<u32>() as f32
            / n as f32
            / 2 as f32
    }

    /// this is only available for terminal observations
    fn theirs(&self) -> Vec<Hole> {
        todo!("terminal: generate all possible villain hands")
    }
}

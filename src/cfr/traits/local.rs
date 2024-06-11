use crate::Utility;

use super::action::E;
use super::bucket::B;
use super::player::C;

pub(crate) trait Local<A, B, C>
where
    Self: Sized,
    A: super::action::Action,
    B: super::bucket::Bucket,
    C: super::player::Player,
{
    fn root() -> Self;
    fn bucket(&self) -> &B;
    fn player(&self) -> &C;
    fn spawn(&self) -> Vec<(Self, A)>;
    fn payoff(&self, player: &C) -> crate::Utility;
}

pub(crate) struct L(usize);

impl L {
    /// root of tree is a static method
    pub fn root() -> Self {
        Self(0)
    }
    /// abstraction
    pub fn bucket(&self) -> &B {
        match self.0 {
            00 => &B::P1,
            01..=03 => &B::P2,
            04..=12 => &B::Ignore,
            _ => unreachable!(),
        }
    }
    /// attribution
    pub fn player(&self) -> &C {
        match self.0 {
            00 => &C::P1,
            01..=03 => &C::P2,
            04..=12 => &C::Chance,
            _ => unreachable!(),
        }
    }
    /// local tree generation
    pub fn spawn(&self) -> Vec<(Self, E)> {
        match self.0 {
            // P1 moves
            00 => vec![(Self(01), E::RK), (Self(02), E::PA), (Self(03), E::SC)],
            // P2 moves
            01 => vec![(Self(04), E::RK), (Self(05), E::PA), (Self(06), E::SC)],
            02 => vec![(Self(07), E::RK), (Self(08), E::PA), (Self(09), E::SC)],
            03 => vec![(Self(10), E::RK), (Self(11), E::PA), (Self(12), E::SC)],
            // terminal nodes
            04..=12 => Vec::new(),
            //
            _ => unreachable!(),
        }
    }
    /// utility to player
    pub fn payoff(&self, player: &C) -> Utility {
        const LO_STAKES: Utility = 1.;
        const HI_STAKES: Utility = 5.; // we can modify payoffs to verify convergence
        let direction = match player {
            C::P1 => 0. + 1.,
            C::P2 => 0. - 1.,
            C::Chance => unreachable!("payoff should not be queried for chance"),
        };
        let payoff = match self.0 {
            07 => 0. + LO_STAKES, // P > R
            05 => 0. - LO_STAKES, // R < P
            06 => 0. + HI_STAKES, // R > S
            11 => 0. + HI_STAKES, // S > P
            10 => 0. - HI_STAKES, // S < R
            09 => 0. - HI_STAKES, // P < S
            04 | 08 | 12 => 0.0,
            00..=03 | _ => unreachable!("eval at terminal node, depth > 1"),
        };
        direction * payoff
    }
}

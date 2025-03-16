use super::edge::Edge;
use super::edges::Decision;
use super::node::Node;
use super::nodes::Position;
use super::policy::Policy;
use super::profile::Profile;
use super::turn::Turn;
use crate::transport::density::Density;
use crate::Probability;
use crate::Utility;

/// This trait is used to evaluate the Utility of
/// a given < Strategy | Tree > "inner product", which
/// i'm calling marginal_value here for whatever reason.
/// The &Tree is embedded in the &Node, and the &Policy
/// are embedded in the &Profile. We only walk the Tree
/// and so don't need ownership of anything other than our profile reference.
/// So consider this a wrapper around Profile.
pub trait Trainer {
    fn train<P>(&self) -> P
    where
        P: Profile;

    fn profile<P>(&self) -> &P
    where
        P: Profile;

    /// Historically,
    /// upon visiting any Node in this Infoset,
    /// how much cumulative Utility have we missed out on
    /// for not having followed this Edge?
    fn regret<D, E>(&self, info: &D, edge: &E) -> Utility
    where
        E: Edge,
        D: Decision<E>;

    /// Update myself
    fn update_regret<P, D, E>(&mut self, info: &D, update: &P)
    where
        E: Edge,
        D: Decision<E>,
        P: Density<Support = E>;

    /// Update myself
    fn update_policy<P, D, E>(&mut self, info: &D, update: &P)
    where
        E: Edge,
        D: Decision<E>,
        P: Density<Support = E>;

    /// Using our current regret Profile,
    /// compute a new strategy vector
    /// by following a given Edge
    /// proportionally to how much regret we felt
    /// for not having followed that Edge in the past.
    fn policy_vector<N, P, I, D, E>(&self, infoset: I) -> P
    where
        N: Node,
        E: Edge,
        I: Position<N>,
        D: Decision<E>,
        P: Policy<E>;

    /*



     * default implemenation


    */

    fn step<Y, X, E, N, P, I, T>(&mut self, infoset: I)
    where
        E: Edge,
        N: Node,
        T: Turn,
        Y: Profile,
        X: Decision<E>,
        I: Position<N>,
        P: Policy<E>,
    {
        let ref prof = self.profile::<Y>();
        let regret = prof.regret_vector::<N, P, I, X, E, T>(infoset.clone());
        let policy = self.policy_vector::<N, P, I, X, E>(infoset.clone());
        self.update_regret::<P, X, E>(&infoset.decision::<E, X>(), &regret);
        self.update_policy::<P, X, E>(&infoset.decision::<E, X>(), &policy);
    }

    /// select policy for a single Edge at this InfoSet
    fn policy<N, I, J, E>(&self, infoset: I, edge: &E) -> Probability
    where
        N: Node,
        E: Edge,
        I: Position<N>,
        J: Decision<E>,
    {
        let info = infoset.decision::<E, J>();
        let regrets = info
            .map(|edge| (edge, self.regret(info, &edge)))
            .collect::<Vec<(E, Utility)>>();
        let denominator = regrets
            .iter()
            .map(|(_, r)| r.max(crate::POLICY_MIN))
            .sum::<Utility>();
        regrets
            .into_iter()
            .map(|(a, r)| (a, r.max(crate::POLICY_MIN)))
            .map(|(a, r)| (a, r / denominator))
            .inspect(|(a, p)| log::trace!("{:16} ~ {:>5.03}", format!("{:?}", a), p))
            .inspect(|(_, p)| assert!(*p >= 0.))
            .inspect(|(_, p)| assert!(*p <= 1.))
            .find(|(a, _)| a == edge)
            .map(|(_, p)| p)
            .unwrap_or(0.)
    }
}

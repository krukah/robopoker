use super::edge::Edge;
use super::edges::EdgeSet;
use super::node::Node;
use super::nodes::NodeSet;
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
    fn regret<I, E>(&self, info: &I, edge: &E) -> Utility
    where
        E: Edge,
        I: EdgeSet<E>;

    /// Update myself
    fn update_regret<P, I, E>(&mut self, info: &I, update: &P)
    where
        E: Edge,
        I: EdgeSet<E>,
        P: Density<Support = E>;

    /// Update myself
    fn update_policy<P, I, E>(&mut self, info: &I, update: &P)
    where
        E: Edge,
        I: EdgeSet<E>,
        P: Density<Support = E>;

    /*



     * default implemenation


    */

    fn step<Y, X, E, N, P, I, T>(&mut self, infoset: I)
    where
        E: Edge,
        N: Node,
        T: Turn,
        Y: Profile,
        X: EdgeSet<E>,
        I: NodeSet<N>,
        P: Density<Support = E> + From<Vec<(E, Utility)>>, // Policy trait ??
    {
        let policy = self.policy_vector::<N, P, I, X, E>(infoset.clone());
        let regret = self
            .profile::<Y>()
            .regret_vector::<N, P, I, X, E, T>(infoset.clone());
        self.update_regret::<P, X, E>(&infoset.info::<E, X>(), &regret);
        self.update_policy::<P, X, E>(&infoset.info::<E, X>(), &policy);
    }

    /// Using our current regret Profile,
    /// compute a new strategy vector
    /// by following a given Edge
    /// proportionally to how much regret we felt
    /// for not having followed that Edge in the past.
    fn policy_vector<N, P, I, J, E>(&self, infoset: I) -> P
    where
        N: Node,
        E: Edge,
        I: NodeSet<N>,
        J: EdgeSet<E>,
        P: Density<Support = E> + From<Vec<(E, Probability)>>,
    {
        let info = infoset.info::<E, J>();
        let regrets = info
            .map(|edge| (edge, self.regret(info, &edge)))
            .map(|(a, r)| (a, r.max(crate::POLICY_MIN)))
            .collect::<Vec<(E, Utility)>>();
        let denom = regrets.iter().map(|(_, r)| r).sum::<Utility>();
        let policy = regrets
            .into_iter()
            .map(|(a, r)| (a, r / denom))
            .inspect(|(a, p)| log::trace!("{:16} ~ {:>5.03}", format!("{:?}", a), p))
            .inspect(|(_, p)| assert!(*p >= 0.))
            .inspect(|(_, p)| assert!(*p <= 1.))
            .collect::<Vec<(E, Probability)>>();
        policy.into()
    }
}

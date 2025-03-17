use super::edge::Edge;
use super::edge::EdgeSet;
use super::node::Node;
use super::node::NodeSet;
use super::policy::Policy;
use super::profile::Profile;
use super::turn::Turn;
use crate::transport::density::Density;
use crate::Utility;

/// This trait is used to evaluate the Utility of
/// a given < Strategy | Tree > "inner product", which
/// i'm calling marginal_value here for whatever reason.
/// The &Tree is embedded in the &Node, and the &Policy
/// are embedded in the &Profile. We only walk the Tree
/// and so don't need ownership of anything other than our profile reference.
/// So consider this a wrapper around Profile.
pub trait Trainer {
    // type E: Edge;
    // type D: EdgeSet<Self::E>;
    // type N: Node;
    // type I: NodeSet<Self::N>;
    // type P: Policy<Self::E>;
    // type T: Tree;
    // type W: Turn;

    fn train<P>(&self) -> P
    where
        P: Profile,
    {
        todo!("seed, recursively expandm, return profile after some number of iterations")
    }
    fn profile<P>(&self) -> &P
    where
        P: Profile,
    {
        todo!("reference profile")
    }
    /// Historically,
    /// upon visiting any Node in this Infoset,
    /// how much cumulative Utility have we missed out on
    /// for not having followed this Edge?
    fn regret<D, E>(&self, info: &D, edge: &E) -> Utility
    where
        E: Edge,
        D: EdgeSet<E>,
    {
        todo!("lookup historical regret value")
    }
    /// Update myself
    fn update_regret<P, D, E>(&mut self, info: &D, update: &P)
    where
        E: Edge,
        D: EdgeSet<E>,
        P: Policy<E>,
    {
        todo!("update regret memory, applying discount factor.")
    }
    /// Update myself
    fn update_policy<P, D, E>(&mut self, info: &D, update: &P)
    where
        E: Edge,
        D: EdgeSet<E>,
        P: Policy<E>,
    {
        todo!("update policy memory, applying discount factor.")
    }

    /// take one iteration of CFR. not parallelized yet.
    /// need to think about whether this should actually return a
    /// set of Counterfactual results from which we can apply
    /// updates in a single-threaded manner, after generating them
    /// in parallel.
    /// Iterator<Item = (D, P, R)>
    /// where
    ///     D: DecisionSet<E>,
    ///     P: Policy<E>,
    ///     R: Regret<E>
    fn counterfactual<Y, D, E, N, P, I, T>(&mut self, infoset: I) -> (D, P, P)
    where
        E: Edge,
        N: Node,
        T: Turn,
        Y: Profile,
        D: EdgeSet<E>,
        I: NodeSet<N>,
        P: Policy<E>,
    {
        let ref prof = self.profile::<Y>();
        let regret = prof.regret_vector::<N, P, I, D, E, T>(infoset.clone());
        let policy = self.policy_vector::<N, P, I, D, E>(infoset.clone());
        let bucket = infoset.decision::<E, D>().clone();
        (bucket, policy, regret)
    }

    /// Using our current regret Profile,
    /// compute a new strategy vector
    /// by following a given Edge
    /// proportionally to how much regret we felt
    /// for not having followed that Edge in the past.
    fn policy_vector<N, P, I, D, E>(&self, infoset: I) -> P
    where
        N: Node,
        E: Edge,
        I: NodeSet<N>,
        D: EdgeSet<E>,
        P: Policy<E>,
    {
        let info = infoset.decision::<E, D>();
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
            .collect::<P>()
    }
}

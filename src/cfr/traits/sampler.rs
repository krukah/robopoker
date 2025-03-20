use super::encoder::Encoder;
use super::leaf::Leaf;
use super::node::Node;
use super::profile::Profile;
use super::tree::Tree;
use super::turn::Playee;

/// The Sampler will encapsulate all the massive objects
/// that we need to generate Trees in memory
/// (namely, Encoder + Profile). It will also handle
/// the mutability constraints of profile witnessing.
pub trait Sampler {
    /// Just grow a tree from the ground up
    fn sample<T>(&self) -> T
    where
        T: Tree,
    {
        todo!("get default tree. get game root. lookup info abstraction in table. root DecisionSet. empty history. insert into tree.")
    }

    /// To support different sampling schemes, we need
    /// to assign a Player to be the "traverser" of the
    /// Tree that we are harvesting.
    fn walker<W>(&self) -> &W
    where
        W: Playee,
    {
        todo!("iteration % 2..if we bind W: From<usize> we can implement without generics by iteration() -> usize")
    }

    /// Roughly static reference to the massive lookup table
    /// of Observation -> Abstraction
    fn encoder<E>(&self) -> &E
    where
        E: Encoder,
    {
        todo!("reference encoder")
    }

    /// We'll probably need to have a Profile
    /// to correctly sample Node, Edge pairs
    /// based on different sampling schemes
    fn profile<P>(&self) -> &P
    where
        P: Profile,
    {
        todo!("reference profile")
    }

    /// Encapsulation of [external, internal, probing]
    /// sampling strategies. In practice, we will use
    /// a Profile to sample different forks from this
    /// Node according to existing policy.
    ///
    /// This might need to be mutable if we
    /// include Profile::witness() within here
    fn expand<N, I, L>(&self, node: N) -> I
    where
        N: Node,
        L: Leaf,
        I: Iterator<Item = L>,
    {
        todo!("if chance, touch any (uniform). if walker, touch all. else, touch one (policy-weighted).")
    }
}

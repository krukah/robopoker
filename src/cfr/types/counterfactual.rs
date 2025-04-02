/// this is pre-implemented. it is a wrapper around
/// different edge-indexed distributions of regret and policy
/// at a given Info point.
///
/// this is the smallest unit of information that can be used
/// to update a Profile. two densities over decision space.
pub type Counterfactual<E, I> = (I, super::policy::Policy<E>, super::policy::Policy<E>);

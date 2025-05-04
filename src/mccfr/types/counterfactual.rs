/// this is pre-implemented. it is a wrapper around
/// different edge-indexed distributions of regret and policy
/// at a given Info point.
///
/// this is the smallest unit of information that can be used
/// to update a Profile. two densities over decision space.
///
/// there is a very real possibility of mistkaing the first
/// and second fields, since they have identical types.
/// try to not do this.
pub type Counterfactual<E, I> = (I, super::policy::Policy<E>, super::policy::Policy<E>);

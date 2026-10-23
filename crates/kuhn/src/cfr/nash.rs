use crate::*;
use pokerkit::*;

/// This variant's unique equilibrium, derived in the README: five pure actions, and six mixed
/// ones that are all fractions of 31. Each target carries the tolerance the tabular tests use.
pub struct KuhnNash;

impl KuhnNash {
    /// `(rank, history, edge, target, tolerance)`.
    #[rustfmt::skip]
    pub fn targets() -> [(Rank, History, KuhnEdge, Probability, Probability); 12] {
        [
            (Rank::J, History::Bet,      KuhnEdge::Fold,  1.,        0.05),
            (Rank::J, History::CheckBet, KuhnEdge::Fold,  1.,        0.05),
            (Rank::K, History::Bet,      KuhnEdge::Call,  1.,        0.05),
            (Rank::K, History::CheckBet, KuhnEdge::Call,  1.,        0.05),
            (Rank::K, History::Check,    KuhnEdge::Bet,   1.,        0.05),
            (Rank::Q, History::Open,     KuhnEdge::Check, 1.,        0.15),
            (Rank::J, History::Open,     KuhnEdge::Bet,   9. / 31.,  0.05),
            (Rank::K, History::Open,     KuhnEdge::Bet,   27. / 31., 0.05),
            (Rank::Q, History::Bet,      KuhnEdge::Call,  17. / 31., 0.08),
            (Rank::Q, History::CheckBet, KuhnEdge::Call,  23. / 31., 0.05),
            (Rank::J, History::Check,    KuhnEdge::Bet,   9. / 31.,  0.05),
            (Rank::Q, History::Check,    KuhnEdge::Bet,   8. / 31.,  0.18),
        ]
    }

    /// Every pure target `policy` misses by `tolerance` or more, described. Exploitability is flat
    /// around the mixed targets, so only the pure ones pin a near-equilibrium policy.
    pub fn strays<P>(policy: P, tolerance: Probability) -> Vec<String>
    where
        P: Fn(&KuhnInfo, &KuhnEdge) -> Probability,
    {
        Self::targets()
            .into_iter()
            .filter(|(.., target, _)| *target == 1.)
            .map(|(rank, node, edge, ..)| (rank, node, edge, policy(&Info::kuhn_info(true, rank, node), &edge)))
            .filter(|(.., p)| 1. - p >= tolerance)
            .map(|(rank, node, edge, p)| format!("{rank:?}|{node:?} {edge:?}: {p:.3} not within {tolerance:.3} of 1"))
            .collect()
    }

    /// Every target `policy` misses by more than its tolerance times `slack`, described.
    pub fn misses<P>(policy: P, slack: Probability) -> Vec<String>
    where
        P: Fn(&KuhnInfo, &KuhnEdge) -> Probability,
    {
        Self::targets()
            .into_iter()
            .map(|(rank, node, edge, target, tolerance)| {
                (rank, node, edge, target, tolerance * slack, policy(&Info::kuhn_info(true, rank, node), &edge))
            })
            .filter(|(.., target, tolerance, p)| (p - target).abs() >= *tolerance)
            .map(|(rank, node, edge, target, tolerance, p)| {
                format!("{rank:?}|{node:?} {edge:?}: {p:.3} not within ±{tolerance:.3} of {target:.3}")
            })
            .collect()
    }
}

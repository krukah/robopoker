use rbp_core::*;

/// Blueprint's weighted average EV: sum of sigma(a) * v(a).
pub fn strategy_ev(policy: &[(Probability, Utility)]) -> Utility {
    let total: Probability = policy.iter().map(|(w, _)| w).sum();
    if total <= 0.0 {
        return 0.0;
    }
    policy.iter().map(|(w, v)| (w / total) * v).sum()
}

/// EV of the specific action taken. Falls back to strategy_ev if action not found.
pub fn observed_ev(policy: &[(Probability, Utility)], idx: Option<usize>) -> Utility {
    idx.and_then(|i| policy.get(i))
        .map_or_else(|| strategy_ev(policy), |(_, v)| *v)
}

/// Action-node correction: removes variance from which action was chosen.
/// Positive when observed action was worse than average; negative when better.
pub fn action_correction(policy: &[(Probability, Utility)], idx: Option<usize>) -> Utility {
    strategy_ev(policy) - observed_ev(policy, idx)
}

/// Chance-node correction: removes variance from which cards were dealt.
/// avg_baseline: deal-count-weighted average of per-bucket baselines.
/// obs_baseline: baseline of the bucket the actual deal mapped to.
pub fn chance_correction(avg_baseline: Utility, obs_baseline: Utility) -> Utility {
    avg_baseline - obs_baseline
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    const N: usize = 100;

    fn random_policy(rng: &mut impl Rng) -> Vec<(Probability, Utility)> {
        let n = rng.random_range(2..=5);
        (0..n)
            .map(|_| (rng.random_range(0.01..10.0), rng.random_range(-100.0..100.0)))
            .collect()
    }
    #[test]
    fn uniform_ev_yields_zero_correction_for_any_action() {
        let mut rng = rand::rng();
        for _ in 0..N {
            let ev = rng.random_range(-100.0..100.0);
            let n = rng.random_range(2..=5);
            let policy: Vec<(Probability, Utility)> = (0..n).map(|_| (rng.random_range(0.01..10.0), ev)).collect();
            for i in 0..n {
                assert!((action_correction(&policy, Some(i))).abs() < 1e-4);
            }
        }
    }
    #[test]
    fn correction_sums_to_zero_under_policy() {
        let mut rng = rand::rng();
        for _ in 0..N {
            let policy = random_policy(&mut rng);
            let total: Probability = policy.iter().map(|(w, _)| w).sum();
            let weighted_sum: Utility = policy
                .iter()
                .enumerate()
                .map(|(i, (w, _))| (w / total) * action_correction(&policy, Some(i)))
                .sum();
            assert!(weighted_sum.abs() < 1e-3, "weighted correction sum {weighted_sum} != 0");
        }
    }
    #[test]
    fn best_action_gets_negative_correction() {
        let mut rng = rand::rng();
        for _ in 0..N {
            let policy = random_policy(&mut rng);
            let best = policy
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.1.partial_cmp(&b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();
            assert!(action_correction(&policy, Some(best)) <= 1e-6);
        }
    }
    #[test]
    fn worst_action_gets_positive_correction() {
        let mut rng = rand::rng();
        for _ in 0..N {
            let policy = random_policy(&mut rng);
            let worst = policy
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.1.partial_cmp(&b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();
            assert!(action_correction(&policy, Some(worst)) >= -1e-6);
        }
    }
    #[test]
    fn missing_idx_yields_zero_correction() {
        let mut rng = rand::rng();
        for _ in 0..N {
            let policy = random_policy(&mut rng);
            assert_eq!(action_correction(&policy, None), 0.0);
            assert_eq!(action_correction(&policy, Some(999)), 0.0);
        }
    }
    #[test]
    fn chance_correction_is_antisymmetric() {
        let mut rng = rand::rng();
        for _ in 0..N {
            let a: Utility = rng.random_range(-100.0..100.0);
            let b: Utility = rng.random_range(-100.0..100.0);
            assert!((chance_correction(a, b) + chance_correction(b, a)).abs() < 1e-6);
        }
    }
    #[test]
    fn chance_correction_is_zero_when_equal() {
        let mut rng = rand::rng();
        for _ in 0..N {
            let v: Utility = rng.random_range(-100.0..100.0);
            assert_eq!(chance_correction(v, v), 0.0);
        }
    }
    #[test]
    fn empty_policy_yields_zero() {
        let policy: Vec<(Probability, Utility)> = vec![];
        assert_eq!(strategy_ev(&policy), 0.0);
        assert_eq!(action_correction(&policy, Some(0)), 0.0);
        assert_eq!(action_correction(&policy, None), 0.0);
    }
    #[test]
    fn zero_weight_policy_yields_zero_strategy_ev() {
        let mut rng = rand::rng();
        for _ in 0..N {
            let n = rng.random_range(2..=5);
            let policy: Vec<(Probability, Utility)> = (0..n).map(|_| (0.0, rng.random_range(-100.0..100.0))).collect();
            assert_eq!(strategy_ev(&policy), 0.0);
        }
    }
}

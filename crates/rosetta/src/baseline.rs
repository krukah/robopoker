//! What a street looks like on average, so a bucket can be described by how
//! it differs.
use crate::*;
use pokerkit::*;
use std::collections::BTreeMap;

/// The population-weighted feature frequencies of a whole street.
///
/// Raw frequency is a poor guide to what makes a bucket interesting: nearly
/// every flop is two-tone and nearly every river board has three ranks inside
/// a five-span, so "two-tone board" and "connected board" win frequency
/// contests while saying nothing. **Lift** — how much more often a bucket
/// shows a feature than its street does — says what is actually distinctive.
/// A bucket that is 72% paired on a street that is 17% paired is a bucket
/// about paired boards.
///
/// Every bucket's sample stands in for its whole population, so the
/// share-weighted mixture is what a random isomorphism on the street looks
/// like — computed from the profiles already in hand, at no extra query.
pub struct Baseline(BTreeMap<Feature, Probability>);

impl Baseline {
    /// How much more often a bucket carries a feature than its street does,
    /// in frequency points.
    pub fn lift(&self, feature: Feature, frequency: Probability) -> Probability {
        frequency - self.frequency(feature)
    }

    /// The street's own rate for a feature.
    pub fn frequency(&self, feature: Feature) -> Probability {
        self.0.get(&feature).copied().unwrap_or_default()
    }

    /// The most distinctive feature of a bucket within a group: the one whose
    /// frequency most exceeds the street's, among those common enough inside
    /// the bucket to be worth saying at all. Ties go to the earlier feature,
    /// which is the more salient one by declaration order.
    pub fn distinguishes(
        &self,
        profile: &Profile,
        group: &[Feature],
        floor: Probability,
    ) -> Option<(Feature, Probability)> {
        group
            .iter()
            .copied()
            .map(|feature| (feature, profile.frequency(feature)))
            .filter(|(_, frequency)| *frequency >= floor)
            .map(|(feature, frequency)| (feature, self.lift(feature, frequency)))
            .fold(None, |best, (feature, lift)| match best {
                Some((_, held)) if held >= lift => best,
                _ => Some((feature, lift)),
            })
    }

    /// Everything a bucket does unusually often, most surprising first —
    /// the description's one-line answer to "what is this bucket *for*".
    pub fn distinctions(
        &self,
        profile: &Profile,
        floor: Probability,
        lift: Probability,
    ) -> Vec<(Feature, Probability)> {
        let mut ranked = Feature::all()
            .iter()
            .copied()
            .map(|feature| (feature, profile.frequency(feature)))
            .filter(|(_, frequency)| *frequency >= floor)
            .map(|(feature, frequency)| (feature, self.lift(feature, frequency)))
            .filter(|(_, over)| *over >= lift)
            .collect::<Vec<_>>();
        ranked.sort_by(|(_, a), (_, b)| b.total_cmp(a));
        ranked
    }
}

impl From<&[Profile]> for Baseline {
    fn from(profiles: &[Profile]) -> Self {
        Self(
            Feature::all()
                .iter()
                .copied()
                .map(|feature| {
                    (
                        feature,
                        profiles
                            .iter()
                            .map(|profile| profile.census().share() * profile.frequency(feature))
                            .sum(),
                    )
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deuce::*;
    use kicker::*;

    fn profile(index: usize, share: Probability, board: &str) -> Profile {
        Profile::from((
            Census::from((Abstraction::from((Street::Flop, index)), 100, share, 0.5, 0.1)),
            ["AsKh", "QsJh", "9s8h"]
                .iter()
                .map(|hole| format!("{hole} ~ {board}"))
                .map(|obs| Observation::try_from(obs.as_str()).expect("parses"))
                .collect::<Vec<Observation>>(),
        ))
    }

    /// Half the street is monotone, so being monotone is worth +50 points to
    /// the bucket that always is and -50 to the one that never is.
    #[test]
    fn lift_is_a_bucket_against_its_street() {
        let street = vec![profile(0, 0.5, "2d 7d 5d"), profile(1, 0.5, "2d 7c 5s")];
        let baseline = Baseline::from(street.as_slice());
        assert!((baseline.frequency(Feature::MonotoneBoard) - 0.5).abs() < 1e-6);
        assert!((baseline.lift(Feature::MonotoneBoard, 1.0) - 0.5).abs() < 1e-6);
        assert!((baseline.lift(Feature::MonotoneBoard, 0.0) + 0.5).abs() < 1e-6);
    }

    /// Both buckets are 100% low-board; only one is monotone. Frequency alone
    /// cannot tell them apart, which is the whole reason lift exists.
    #[test]
    fn distinguishes_prefers_the_surprising_over_the_universal() {
        let street = vec![profile(0, 0.5, "2d 7d 5d"), profile(1, 0.5, "2d 7c 5s")];
        let baseline = Baseline::from(street.as_slice());
        assert_eq!(
            Some(Feature::MonotoneBoard),
            baseline
                .distinguishes(&street[0], &Feature::BOARD, 0.5)
                .map(|(feature, _)| feature)
        );
        assert!(baseline.frequency(Feature::LowBoard) > baseline.frequency(Feature::MonotoneBoard));
    }
}

//! What kind of hand a bucket holds — the head of its name.
use crate::*;
use pokerkit::*;

/// The made-hand claim a bucket's name leads with.
///
/// A bucket does not always agree on one class. It can agree that hero paired
/// *something* without agreeing which pair, or that hero beat one pair without
/// agreeing how — and both of those are worth saying, because they separate a
/// value bucket from a bluff bucket. So the head of a name is one of these
/// five things, not a string, and everything downstream can ask what it is:
/// a kicker qualifies [`Class::Pairs`] and nothing else.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Class {
    /// One made-hand class the bucket agrees on outright.
    Sole(Feature),
    /// Some pair, no agreement on which.
    Pairs,
    /// Two pair or better, no agreement on which.
    Strong,
    /// Hero connected, no agreement beyond that.
    Made,
    /// No dominant structure at all — a bucket worth looking at twice.
    Mixed,
}

impl Class {
    /// A class this common speaks for the whole bucket on its own.
    const DECISIVE: Probability = 0.55;
    /// A family the bucket agrees on this often names it even when no single
    /// member of that family does.
    const PREVALENT: Probability = 0.65;
    /// Below decisive, the modal class still names the bucket from here up.
    const MODAL: Probability = 0.40;
    /// Every one-pair class. Several at once still means one pair.
    const PAIRS: [Feature; 5] = [
        Feature::Overpair,
        Feature::TopPair,
        Feature::MidPair,
        Feature::LowPair,
        Feature::Underpair,
    ];
    /// Every class that beats one pair. Several at once still means a big hand.
    const STRONG: [Feature; 6] = [
        Feature::Monster,
        Feature::Boat,
        Feature::Flush,
        Feature::Straight,
        Feature::Trips,
        Feature::TwoPair,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Sole(feature) => feature.label(),
            Self::Pairs => "some pair",
            Self::Strong => "two pair or better",
            Self::Made => "one pair or better",
            Self::Mixed => "mixed holdings",
        }
    }

    /// Whether the class names a pair — the only class a kicker can qualify.
    /// Asking the class is what keeps this answer in step with the cascade
    /// that produced it; re-deriving it from the profile is how the two drift.
    pub fn pairs(&self) -> bool {
        match self {
            Self::Sole(feature) => Self::PAIRS.contains(feature),
            Self::Pairs => true,
            _ => false,
        }
    }

    /// Whether the modal class belongs to a family the bucket agrees on, even
    /// though no single member of that family won outright.
    fn family(profile: &Profile, group: &[Feature], modal: Feature) -> bool {
        group.contains(&modal) && profile.total(group) >= Self::DECISIVE
    }

    /// How often hero has anything at all.
    fn connects(profile: &Profile) -> Probability {
        profile.total(&Self::PAIRS) + profile.total(&Self::STRONG)
    }
}

impl From<&Profile> for Class {
    fn from(profile: &Profile) -> Self {
        match profile.dominant(&Feature::MADE) {
            Some((feature, p)) if p >= Self::DECISIVE => Self::Sole(feature),
            Some((feature, _)) if Self::family(profile, &Self::PAIRS, feature) => Self::Pairs,
            Some((feature, _)) if Self::family(profile, &Self::STRONG, feature) => Self::Strong,
            Some(_) if Self::connects(profile) >= Self::PREVALENT => Self::Made,
            Some((feature, p)) if p >= Self::MODAL => Self::Sole(feature),
            _ => Self::Mixed,
        }
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deuce::*;
    use kicker::*;

    fn profile(hands: &[&str]) -> Profile {
        Profile::from((
            Census::from((Abstraction::from((Street::Flop, 0)), hands.len(), 1.0, 0.5, 0.1)),
            hands
                .iter()
                .map(|obs| Observation::try_from(*obs).expect("parses"))
                .collect::<Vec<Observation>>(),
        ))
    }

    /// Half top pair, half middle pair: no class wins, but the bucket is
    /// unanimous that hero paired something.
    #[test]
    fn a_split_between_pair_classes_is_still_a_pair() {
        let class = Class::from(&profile(&["As2h ~ Ad7c9s", "Ks2h ~ Kd7c9s", "7s2h ~ Ad7c9s", "7s3h ~ Kd7c9s"]));
        assert_eq!(Class::Pairs, class);
        assert!(class.pairs());
    }

    /// One class outright, and it is a pair, so a kicker still qualifies it.
    #[test]
    fn a_decisive_pair_names_itself() {
        let class = Class::from(&profile(&["As2h ~ Ad7c9s", "Ks2h ~ Kd7c9s", "Qs3h ~ Qd7c9s", "As4h ~ Ad7c8s"]));
        assert_eq!(Class::Sole(Feature::TopPair), class);
        assert!(class.pairs());
    }

    /// Nothing paired, so nothing a kicker can attach to.
    #[test]
    fn a_bucket_that_missed_takes_no_kicker() {
        let class = Class::from(&profile(&["5s4h ~ Ad7c9s", "6s3h ~ Kd7c9s", "5s2h ~ Qd7c9s", "6s4h ~ Ad7c8s"]));
        assert_eq!(Class::Sole(Feature::NoPair), class);
        assert!(!class.pairs());
    }
}

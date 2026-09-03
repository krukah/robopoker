use super::evaluator::Evaluator;
use super::hand::Hand;
use super::kicks::Kickers;
use super::ranking::Ranking;

/// A fully-evaluated hand strength for comparison.
///
/// Combines a [`Ranking`] (hand category like flush or two pair) with
/// [`Kickers`] (tie-breaking cards). Ordering is lexicographic: ranking
/// first, then kickers.
///
/// Constructed from a [`Hand`] by running the [`Evaluator`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Strength {
    value: Ranking,
    pub kicks: Kickers,
}

impl From<Hand> for Strength {
    fn from(hand: Hand) -> Self {
        Self::from(Evaluator::from(hand))
    }
}

impl From<Evaluator> for Strength {
    fn from(e: Evaluator) -> Self {
        let value = e.find_ranking();
        let kicks = e.find_kickers(value);
        Self::from((value, kicks))
    }
}

impl Strength {
    pub fn ranking(&self) -> Ranking {
        self.value
    }
}
impl From<(Ranking, Kickers)> for Strength {
    fn from((value, kicks): (Ranking, Kickers)) -> Self {
        Self { value, kicks }
    }
}

impl std::fmt::Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:<18}{:>5}", self.value, self.kicks)
    }
}

#[cfg(test)]
#[cfg(not(feature = "shortdeck"))]
mod tests {
    use super::*;
    use crate::rank::Rank;

    fn strength(cards: &str) -> Strength {
        Strength::from(Hand::try_from(cards).expect("parses"))
    }

    /// The hand ranking ladder, weakest to strongest. Each example is built
    /// from the lowest ranks that make it, so the ordering can only come from
    /// the category and never from the cards.
    ///
    /// `Ranking` derives `Ord`, which reads variant *declaration position*, and
    /// the enum is written out twice under opposing `cfg`s. Walking the whole
    /// ladder is what catches the two blocks drifting apart.
    #[rustfmt::skip]
    const LADDER: [(&str, &str); 9] = [
        ("high card",      "As Kh Qd Jc 9s"),
        ("one pair",       "2s 2h Kd Qc Js"),
        ("two pair",       "2s 2h 3d 3c Js"),
        ("trips",          "2s 2h 2d Qc Js"),
        ("straight",       "2s 3h 4d 5c 6s"),
        ("flush",          "2s 4s 6s 8s Ts"),
        ("full house",     "2s 2h 2d 3c 3s"),
        ("quads",          "2s 2h 2d 2c 3s"),
        ("straight flush", "2s 3s 4s 5s 6s"),
    ];

    #[test]
    fn ranking_ladder_is_ordered() {
        LADDER.windows(2).for_each(|pair| {
            let ((weak, lo), (strong, hi)) = (pair[0], pair[1]);
            assert!(
                strength(hi) > strength(lo),
                "{strong} ({hi}) should beat {weak} ({lo}): {:?} vs {:?}",
                strength(hi),
                strength(lo)
            );
        });
    }

    /// Pluribus hand `41b/173`: sevens full lost a 21,400-chip pot to a flush.
    #[test]
    fn full_house_beats_flush() {
        assert!(strength("7c7h 2s5s5c7sKs") > strength("QsQh 2s5s5c7sKs"));
    }

    /// Pluribus hand `103b/164`: two flushes off a four-club board, differing
    /// only in their second card.
    #[test]
    fn flushes_compare_below_the_high_card() {
        assert!(strength("As8c 7dJc5c4c3c") > strength("Ks6c 7dJc5c4c3c"));
    }

    /// A flush's kickers have to come from the flush suit. Drawn from the whole
    /// hand, the offsuit king and queen would stand in for the nine and the
    /// weaker flush would win.
    #[test]
    fn flush_kickers_come_from_the_flush_suit() {
        assert!(strength("As Ts 4s 3s 2s 7h 8d") > strength("As 9s 4s 3s 2s Kh Qd"));
    }

    #[test]
    fn identical_flushes_tie() {
        assert_eq!(strength("As8c 7dJc5c4c3c"), strength("As8c 2dJc5c4c3c"));
    }

    /// The other two conformance failures, kept whole so the corpus and the
    /// suite name the same hands.
    #[test]
    fn pluribus_showdowns_resolve() {
        // 116/61: fives beat threes, both playing a flush on a four-heart board.
        assert!(strength("5d5h Jh9h4h6h2s") > strength("3h3c Jh9h4h6h2s"));
        // 34/67: ace-ten of hearts beats pocket sevens, likewise.
        assert!(strength("ThAd 4hAc9hJhQh") > strength("7h7d 4hAc9hJhQh"));
    }

    #[test]
    fn flush_carries_its_lower_four_cards_as_kickers() {
        let flush = strength("As Ks Qs Js 9s");
        assert_eq!(flush.ranking(), Ranking::Flush(Rank::Ace));
        assert_eq!(flush.kicks, Kickers::from(vec![Rank::King, Rank::Queen, Rank::Jack, Rank::Nine]));
    }
}

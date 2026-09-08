//! The mechanistic vocabulary — one predicate per file-level concept.
use deuce::*;

/// One mechanically-decidable predicate about a player's view of a hand.
///
/// Every feature is a pure function of the cards: no equity, no strategy, no
/// clustering. A bucket is interpreted by which predicates its sampled members
/// share, so the vocabulary is deliberately small, exhaustive within each
/// group, and stable across re-clustering — the same cards always yield the
/// same features, which is what makes a [`Gloss`](crate::Gloss) reproducible.
///
/// Declaration order is display order and groups the vocabulary: made hands
/// first (exactly one holds per observation), then kicker quality, draws, hero
/// shape, board texture.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Feature {
    Monster,
    Boat,
    Flush,
    Straight,
    Trips,
    TwoPair,
    Overpair,
    TopPair,
    MidPair,
    LowPair,
    Underpair,
    PlaysBoard,
    NoPair,
    TopKicker,
    GoodKicker,
    WeakKicker,
    NutFlushDraw,
    FlushDraw,
    BackdoorFlush,
    OpenEnded,
    Gutshot,
    PocketPair,
    Suited,
    Overcards,
    AceHigh,
    KingHigh,
    QueenHigh,
    JackHigh,
    TripsBoard,
    PairedBoard,
    MonotoneBoard,
    TwoToneBoard,
    RainbowBoard,
    ConnectedBoard,
    HighBoard,
    MidBoard,
    LowBoard,
}

impl Feature {
    /// Made-hand classes. Exactly one holds for any observation with a board.
    pub const MADE: [Self; 13] = [
        Self::Monster,
        Self::Boat,
        Self::Flush,
        Self::Straight,
        Self::Trips,
        Self::TwoPair,
        Self::Overpair,
        Self::TopPair,
        Self::MidPair,
        Self::LowPair,
        Self::Underpair,
        Self::PlaysBoard,
        Self::NoPair,
    ];
    /// Kicker quality. One holds whenever hero's pair has a kicker at all.
    pub const KICKS: [Self; 3] = [Self::TopKicker, Self::GoodKicker, Self::WeakKicker];
    /// Draws hero holds a card in. Several may hold at once.
    pub const DRAWS: [Self; 5] = [
        Self::NutFlushDraw,
        Self::FlushDraw,
        Self::BackdoorFlush,
        Self::OpenEnded,
        Self::Gutshot,
    ];
    /// Hero's two cards, independent of the board.
    pub const SHAPE: [Self; 7] = [
        Self::PocketPair,
        Self::Suited,
        Self::Overcards,
        Self::AceHigh,
        Self::KingHigh,
        Self::QueenHigh,
        Self::JackHigh,
    ];
    /// What an unpaired hand is worth at showdown, strongest first. Exactly
    /// one of the four card ranks can hold, and `Overcards` may hold with it.
    pub const HIGHS: [Self; 5] = [
        Self::AceHigh,
        Self::KingHigh,
        Self::QueenHigh,
        Self::JackHigh,
        Self::Overcards,
    ];
    /// Board texture. Suitedness and height are each exactly one.
    pub const BOARD: [Self; 9] = [
        Self::TripsBoard,
        Self::PairedBoard,
        Self::MonotoneBoard,
        Self::TwoToneBoard,
        Self::RainbowBoard,
        Self::ConnectedBoard,
        Self::HighBoard,
        Self::MidBoard,
        Self::LowBoard,
    ];

    /// Every feature in the vocabulary, group by group. The order is the
    /// declaration order, which is also display order and tie-break order.
    pub fn all() -> Vec<Self> {
        std::iter::empty::<Self>()
            .chain(Self::MADE)
            .chain(Self::KICKS)
            .chain(Self::DRAWS)
            .chain(Self::SHAPE)
            .chain(Self::BOARD)
            .collect()
    }

    /// Every predicate that holds for one observation.
    pub fn of(obs: &Observation) -> Vec<Self> {
        std::iter::empty::<Self>()
            .chain(Self::made(obs))
            .chain(Self::draws(obs))
            .chain(Self::shape(obs))
            .chain(Self::board(obs))
            .collect()
    }

    /// Human-readable label, lowercase so it composes into a sentence.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Monster => "quads or better",
            Self::Boat => "full house",
            Self::Flush => "flush",
            Self::Straight => "straight",
            Self::Trips => "trips",
            Self::TwoPair => "two pair",
            Self::Overpair => "overpair",
            Self::TopPair => "top pair",
            Self::MidPair => "middle pair",
            Self::LowPair => "bottom pair",
            Self::Underpair => "underpair",
            Self::PlaysBoard => "plays the board",
            Self::NoPair => "no pair",
            Self::TopKicker => "top kicker",
            Self::GoodKicker => "good kicker",
            Self::WeakKicker => "weak kicker",
            Self::NutFlushDraw => "nut flush draw",
            Self::FlushDraw => "flush draw",
            Self::BackdoorFlush => "backdoor flush",
            Self::OpenEnded => "open-ended draw",
            Self::Gutshot => "gutshot",
            Self::PocketPair => "pocket pair",
            Self::Suited => "suited",
            Self::Overcards => "overcards",
            Self::AceHigh => "ace high",
            Self::KingHigh => "king high",
            Self::QueenHigh => "queen high",
            Self::JackHigh => "jack high",
            Self::TripsBoard => "trips board",
            Self::PairedBoard => "paired board",
            Self::MonotoneBoard => "monotone board",
            Self::TwoToneBoard => "two-tone board",
            Self::RainbowBoard => "rainbow board",
            Self::ConnectedBoard => "connected board",
            Self::HighBoard => "high board",
            Self::MidBoard => "middle board",
            Self::LowBoard => "low board",
        }
    }
}

// classification
impl Feature {
    /// The single made-hand class, plus kicker quality when the class has one.
    /// A pair that lives entirely on the board is not hero's pair — hero plays
    /// the board and reads as [`Feature::NoPair`].
    fn made(obs: &Observation) -> Vec<Self> {
        if obs.public().size() < 3 {
            return Vec::new();
        }
        if Self::mirrors(obs) {
            return vec![Self::PlaysBoard];
        }
        match Strength::from(Hand::from(*obs)).ranking() {
            Ranking::StraightFlush(_) | Ranking::FourOAK(_) => vec![Self::Monster],
            Ranking::FullHouse(_, _) => vec![Self::Boat],
            Ranking::Flush(_) => vec![Self::Flush],
            Ranking::Straight(_) => vec![Self::Straight],
            Ranking::ThreeOAK(r) if Self::holds(obs, r) => vec![Self::Trips],
            Ranking::TwoPair(hi, lo) => match (Self::holds(obs, hi), Self::holds(obs, lo)) {
                (true, true) => vec![Self::TwoPair],
                (true, false) => Self::pair(obs, hi),
                (false, true) => Self::pair(obs, lo),
                (false, false) => vec![Self::NoPair],
            },
            Ranking::OnePair(r) if Self::holds(obs, r) => Self::pair(obs, r),
            _ => vec![Self::NoPair],
        }
    }

    /// Whether hero's best five cards are the five on the board. Only the
    /// river can answer it — an unfinished board has no five-card strength of
    /// its own — and only there does it matter, since a hand that has stopped
    /// improving has stopped being hero's hand.
    fn mirrors(obs: &Observation) -> bool {
        obs.street() == Street::Rive && Strength::from(*obs.public()) == Strength::from(Hand::from(*obs))
    }

    /// Where a hero-held pair of `rank` sits against the board, and — when
    /// hero's other card is not part of the pair — how good that kicker is.
    fn pair(obs: &Observation, rank: Rank) -> Vec<Self> {
        let hi = obs.public().max_rank().expect("board dealt");
        let lo = obs.public().min_rank().expect("board dealt");
        let ranks = Self::ranks(obs.pocket());
        if ranks[0] == ranks[1] {
            vec![Self::pocket(rank, hi, lo)]
        } else {
            vec![Self::against(rank, hi, lo), Self::kicker(Self::other(&ranks, rank), hi)]
        }
    }

    /// A pocket pair reads against the whole board, not against one card.
    fn pocket(rank: Rank, hi: Rank, lo: Rank) -> Self {
        if rank > hi {
            Self::Overpair
        } else if rank < lo {
            Self::Underpair
        } else {
            Self::MidPair
        }
    }

    /// A board pair reads against the card it matched.
    fn against(rank: Rank, hi: Rank, lo: Rank) -> Self {
        if rank == hi {
            Self::TopPair
        } else if rank == lo {
            Self::LowPair
        } else {
            Self::MidPair
        }
    }

    /// Top kicker is an ace, or a king behind an ace-high board.
    fn kicker(kick: Rank, hi: Rank) -> Self {
        if kick == Rank::Ace || (kick == Rank::King && hi == Rank::Ace) {
            Self::TopKicker
        } else if kick >= Rank::Ten {
            Self::GoodKicker
        } else {
            Self::WeakKicker
        }
    }

    /// Flush draws hero has a card in: four to a flush, or three on the flop.
    /// A completed flush is a made hand and yields no draw. "Nut" means hero
    /// holds the ace of the suit, without checking for board blockers.
    fn flush(obs: &Observation) -> Vec<Self> {
        Suit::all()
            .iter()
            .map(|suit| (Hand::from(*obs).of(suit), obs.pocket().of(suit)))
            .filter(|(_, mine)| mine.size() > 0)
            .flat_map(|(all, mine)| match all.size() {
                4 if mine.max_rank() == Some(Rank::Ace) => vec![Self::FlushDraw, Self::NutFlushDraw],
                4 => vec![Self::FlushDraw],
                3 if obs.street() == Street::Flop => vec![Self::BackdoorFlush],
                _ => Vec::new(),
            })
            .collect()
    }

    /// Straight draws hero actually improves: completing ranks the board does
    /// not already have on its own. Two or more is open-ended (or a double
    /// gutshot), one is a gutshot, none is nothing.
    fn straight(obs: &Observation) -> Vec<Self> {
        let full = Hand::from(*obs).ranks();
        if Self::runs(full) {
            return Vec::new();
        }
        match Self::outs(full).saturating_sub(Self::outs(obs.public().ranks())) {
            0 => Vec::new(),
            1 => vec![Self::Gutshot],
            _ => vec![Self::OpenEnded],
        }
    }

    /// Every draw hero holds a card in. The river has none: every card is
    /// out, so four to a flush is four to a flush forever.
    fn draws(obs: &Observation) -> Vec<Self> {
        if obs.public().size() < 3 || obs.street() == Street::Rive {
            return Vec::new();
        }
        std::iter::empty::<Self>()
            .chain(Self::flush(obs))
            .chain(Self::straight(obs))
            .collect()
    }

    /// What hero's two cards are, and whether they both beat the board.
    fn shape(obs: &Observation) -> Vec<Self> {
        let ranks = Self::ranks(obs.pocket());
        let suited = Suit::all().iter().any(|suit| obs.pocket().of(suit).size() == 2);
        std::iter::empty::<Self>()
            .chain((ranks[0] == ranks[1]).then_some(Self::PocketPair))
            .chain(suited.then_some(Self::Suited))
            .chain(Self::overcards(obs))
            .chain(Self::highcard(obs))
            .collect()
    }

    /// What hero's unpaired, unimproved hand is worth at showdown, read off
    /// its top card. Ace-high beats king-high beats queen-high, and below a
    /// jack the top card stops mattering — the hand is air and its equity
    /// says so. A pocket pair of aces is an overpair, not ace-high, and a
    /// hand that paired the board is named by its pair.
    fn highcard(obs: &Observation) -> Option<Self> {
        let ranks = Self::ranks(obs.pocket());
        if ranks[0] == ranks[1] || obs.pocket().ranks() & obs.public().ranks() != 0 {
            return None;
        }
        match ranks[1] {
            Rank::Ace => Some(Self::AceHigh),
            Rank::King => Some(Self::KingHigh),
            Rank::Queen => Some(Self::QueenHigh),
            Rank::Jack => Some(Self::JackHigh),
            _ => None,
        }
    }

    /// Two unpaired hole cards, both above every board card. A pocket pair
    /// over the board is an overpair — a made hand, not two live cards — so
    /// it is deliberately excluded.
    fn overcards(obs: &Observation) -> Option<Self> {
        let ranks = Self::ranks(obs.pocket());
        obs.public()
            .max_rank()
            .filter(|_| ranks[0] != ranks[1])
            .filter(|hi| ranks[0] > *hi)
            .map(|_| Self::Overcards)
    }

    /// Board texture: suitedness, height, pairing, connectivity.
    fn board(obs: &Observation) -> Vec<Self> {
        let board = *obs.public();
        if board.size() < 3 {
            return Vec::new();
        }
        std::iter::empty::<Self>()
            .chain(Some(Self::suited(&board)))
            .chain(Some(Self::height(&board)))
            .chain((board.ranks().count_ones() as usize != board.size()).then_some(Self::PairedBoard))
            .chain(Self::tripsy(&board).then_some(Self::TripsBoard))
            .chain(Self::connected(&board).then_some(Self::ConnectedBoard))
            .collect()
    }

    /// Three or more of one rank on the board — the texture that makes a full
    /// house ordinary and a bucket's equity say almost nothing about hero.
    fn tripsy(board: &Hand) -> bool {
        Vec::<Card>::from(*board)
            .iter()
            .map(Card::rank)
            .fold(std::collections::BTreeMap::<Rank, usize>::new(), |mut counts, rank| {
                *counts.entry(rank).or_default() += 1;
                counts
            })
            .values()
            .any(|&n| n >= 3)
    }

    /// Three-plus of a suit reads monotone even on the turn, where it is the
    /// same strategic object: a live one-card flush.
    fn suited(board: &Hand) -> Self {
        match Suit::all()
            .iter()
            .map(|suit| board.of(suit).size())
            .max()
            .unwrap_or_default()
        {
            0 | 1 => Self::RainbowBoard,
            2 => Self::TwoToneBoard,
            _ => Self::MonotoneBoard,
        }
    }

    /// Height is the top board card: queen-plus is high, ten or jack is
    /// middle, nine and below is low.
    fn height(board: &Hand) -> Self {
        match board.max_rank().expect("board dealt") {
            r if r >= Rank::Queen => Self::HighBoard,
            r if r >= Rank::Ten => Self::MidBoard,
            _ => Self::LowBoard,
        }
    }

    /// Three board ranks inside a five-rank window — the span a straight needs.
    fn connected(board: &Hand) -> bool {
        let ranks = Self::spread(board.ranks());
        match ranks.len() {
            0 | 1 => false,
            2 => ranks[1] - ranks[0] <= 4,
            _ => ranks.windows(3).any(|w| w[2] - w[0] <= 4),
        }
    }

    /// Distinct ranks present, ascending.
    fn spread(ranks: u16) -> Vec<u8> {
        (0..13u8).filter(|i| ranks & (1 << i) != 0).collect()
    }

    /// Whether a rank mask already contains five in a row, wheel included.
    fn runs(ranks: u16) -> bool {
        (0..9).any(|i| (ranks >> i) & 0b11111 == 0b11111) || ranks & 0b1000000001111 == 0b1000000001111
    }

    /// How many absent ranks would complete a straight.
    fn outs(ranks: u16) -> usize {
        (0..13)
            .filter(|i| ranks & (1u16 << i) == 0)
            .filter(|i| Self::runs(ranks | (1u16 << i)))
            .count()
    }

    /// Whether hero's two cards include a given rank.
    fn holds(obs: &Observation, rank: Rank) -> bool {
        obs.pocket().ranks() & u16::from(rank) != 0
    }

    /// Hero's two ranks, ascending.
    fn ranks(hand: &Hand) -> Vec<Rank> {
        Vec::<Card>::from(*hand).iter().map(Card::rank).collect()
    }

    /// The hole rank that is not the paired one.
    fn other(ranks: &[Rank], rank: Rank) -> Rank {
        if ranks[0] == rank { ranks[1] } else { ranks[0] }
    }
}

impl std::fmt::Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(obs: &str) -> Vec<Feature> {
        Feature::of(&Observation::try_from(obs).expect("parses"))
    }

    #[test]
    fn top_pair_top_kicker() {
        let features = read("AsKh ~ Kd7c2s");
        assert!(features.contains(&Feature::TopPair));
        assert!(features.contains(&Feature::TopKicker));
    }

    #[test]
    fn overpair_carries_no_kicker() {
        let features = read("AsAh ~ Kd7c2s");
        assert!(features.contains(&Feature::Overpair));
        assert!(features.contains(&Feature::PocketPair));
        assert!(Feature::KICKS.iter().all(|kick| !features.contains(kick)));
    }

    #[test]
    fn pocket_pair_below_the_board_is_an_underpair() {
        assert!(read("3s3h ~ Kd7c4s").contains(&Feature::Underpair));
    }

    #[test]
    fn bottom_pair_weak_kicker() {
        let features = read("2h5s ~ Kd7c2s");
        assert!(features.contains(&Feature::LowPair));
        assert!(features.contains(&Feature::WeakKicker));
    }

    #[test]
    fn a_pair_on_the_board_is_not_heros_pair() {
        let features = read("9s4h ~ KdKc2s");
        assert!(features.contains(&Feature::NoPair));
        assert!(features.contains(&Feature::PairedBoard));
    }

    #[test]
    fn a_set_reads_as_trips() {
        assert!(read("7s7h ~ Kd7c2s").contains(&Feature::Trips));
    }

    #[test]
    fn both_cards_pairing_is_two_pair() {
        assert!(read("Ks7h ~ Kd7c2s").contains(&Feature::TwoPair));
    }

    #[test]
    fn the_ace_of_the_suit_makes_a_flush_draw_the_nuts() {
        let features = read("AsQs ~ 2s7s9h");
        assert!(features.contains(&Feature::FlushDraw));
        assert!(features.contains(&Feature::NutFlushDraw));
        assert!(features.contains(&Feature::Suited));
    }

    #[test]
    fn three_to_a_flush_is_a_backdoor() {
        assert!(read("AsQs ~ 2s7h9d").contains(&Feature::BackdoorFlush));
    }

    #[test]
    fn two_completing_ranks_is_open_ended() {
        assert!(read("8s9h ~ 2cTd7s").contains(&Feature::OpenEnded));
    }

    #[test]
    fn one_completing_rank_is_a_gutshot() {
        assert!(read("8s9h ~ 2cJd7s").contains(&Feature::Gutshot));
    }

    #[test]
    fn the_wheel_counts_as_a_straight() {
        assert!(read("As2h ~ 3d4c5s").contains(&Feature::Straight));
    }

    #[test]
    fn a_board_hero_cannot_beat_plays_itself() {
        assert!(read("2s3h ~ TdJdQcKsAh").contains(&Feature::PlaysBoard));
    }

    #[test]
    fn the_river_has_no_draws() {
        let features = read("AsKs ~ 2s7s9h4dJc");
        assert!(Feature::DRAWS.iter().all(|draw| !features.contains(draw)));
    }

    #[test]
    fn ace_high_needs_an_ace_and_no_pair() {
        assert!(read("AsQh ~ 2d7c9s").contains(&Feature::AceHigh));
        assert!(!read("AsQh ~ 2d7cQs").contains(&Feature::AceHigh));
    }

    #[test]
    fn a_pocket_pair_over_the_board_is_an_overpair_not_overcards() {
        let features = read("AsAh ~ 2d7c9s");
        assert!(features.contains(&Feature::Overpair));
        assert!(!features.contains(&Feature::Overcards));
        assert!(!features.contains(&Feature::AceHigh));
    }

    #[test]
    fn three_of_a_rank_on_the_board_is_a_trips_board() {
        let features = read("AsKh ~ 2d2c2s");
        assert!(features.contains(&Feature::TripsBoard));
        assert!(features.contains(&Feature::PairedBoard));
        assert!(!read("AsKh ~ 2d2c9s").contains(&Feature::TripsBoard));
    }

    #[test]
    fn high_card_reads_the_top_unpaired_card() {
        assert!(read("AsQh ~ 2d7c9s").contains(&Feature::AceHigh));
        assert!(read("KsQh ~ 2d7c9s").contains(&Feature::KingHigh));
        assert!(read("QsTh ~ 2d7c9s").contains(&Feature::QueenHigh));
        assert!(read("Js8h ~ 2d7c9s").contains(&Feature::JackHigh));
        assert!(Feature::HIGHS.iter().all(|high| !read("Ts8h ~ 2d7c9s").contains(high)));
    }

    #[test]
    fn a_hand_that_paired_the_board_is_named_by_its_pair() {
        let features = read("As9h ~ 2d7c9s");
        assert!(features.contains(&Feature::TopPair));
        assert!(!features.contains(&Feature::AceHigh));
    }

    #[test]
    fn board_texture_reads_off_the_board_alone() {
        assert!(read("AsKh ~ 2d7dTd").contains(&Feature::MonotoneBoard));
        assert!(read("AsKh ~ 2d7cTs").contains(&Feature::RainbowBoard));
        assert!(read("AsKh ~ 7d8c9s").contains(&Feature::ConnectedBoard));
        assert!(read("AsKh ~ 2d7cQs").contains(&Feature::HighBoard));
        assert!(read("AsKh ~ 2d7cTs").contains(&Feature::MidBoard));
        assert!(read("AsKh ~ 2d7c9s").contains(&Feature::LowBoard));
    }

    #[test]
    fn exactly_one_made_class_holds() {
        [
            "AsKh ~ Kd7c2s",
            "9s4h ~ KdKc2s",
            "7s7h ~ Kd7c2s",
            "As2h ~ 3d4c5s",
            "2s3h ~ TdJdQcKsAh",
        ]
        .iter()
        .map(|obs| read(obs))
        .for_each(|features| {
            assert_eq!(1, Feature::MADE.iter().filter(|made| features.contains(made)).count());
        });
    }
}

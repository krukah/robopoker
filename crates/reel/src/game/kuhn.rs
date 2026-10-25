use super::*;
use ::kuhn::KuhnEdge;
use ::kuhn::KuhnGame;
use ::kuhn::KuhnTurn;
use ::kuhn::Rank;
use mccfr::CfrGame;

/// The genericity proof: a Kuhn reel from one rigged deal and three real
/// decisions — check · bet · call — with zero Kuhn-specific drawing code
/// beyond this impl.
impl Scenic for KuhnGame {
    fn demo() -> Demo<Self> {
        let root = KuhnGame::all_roots()
            .find(|game| game.hole_rank(0) == Rank::K && game.hole_rank(1) == Rank::Q)
            .expect("king-queen deal");
        let open = vec![KuhnEdge::Check, KuhnEdge::Bet];
        let respond = vec![KuhnEdge::Fold, KuhnEdge::Call];
        Demo::new(
            KuhnTurn::Player(0),
            vec![
                Step::new(root, open.clone(), Slot::new(0)),
                Step::new(root.apply(KuhnEdge::Check), open, Slot::new(1)),
                Step::new(root.apply(KuhnEdge::Check).apply(KuhnEdge::Bet), respond, Slot::new(1)),
            ],
        )
    }

    fn reveals(&self, hero: Self::T) -> Vec<Token> {
        let seat = match hero {
            KuhnTurn::Player(seat) => seat,
            _ => 0,
        };
        vec![Token::new(vec![Rune::new(
            format!("{:?}", self.hole_rank(seat)),
            Tint::Dark,
            Face::Mono,
        )])]
    }

    fn phase(&self) -> String {
        "DEAL".into()
    }

    fn badge(turn: Self::T) -> String {
        match turn {
            KuhnTurn::Player(seat) => format!("P{}", seat + 1),
            _ => "?".into(),
        }
    }

    fn deed(edge: &Self::E) -> Deed {
        match edge {
            KuhnEdge::Deal(_) => Deed::Draw,
            KuhnEdge::Check => Deed::Check,
            KuhnEdge::Bet => Deed::Open,
            KuhnEdge::Call => Deed::Call,
            KuhnEdge::Fold => Deed::Fold,
        }
    }

    fn glyph(edge: &Self::E) -> String {
        match edge {
            KuhnEdge::Deal(card) => card.to_string(),
            KuhnEdge::Check => "•".into(),
            KuhnEdge::Bet => "+".into(),
            KuhnEdge::Call => "=".into(),
            KuhnEdge::Fold => "-".into(),
        }
    }
}

impl Atlas for KuhnGame {
    fn grid() -> (usize, usize) {
        (3, 1)
    }

    fn cell(col: usize, _row: usize) -> String {
        ["J", "Q", "K"][col].into()
    }

    fn weight(col: usize, _row: usize, phase: usize, hero: bool) -> pokerkit::Probability {
        if hero {
            f32::from(u8::from(col == 2))
        } else {
            [1.0, 0.75, 0.75][col.min(2)] - 0.2 * phase.min(2) as pokerkit::Probability * [1.0, 0.0, -0.5][col]
        }
    }

    fn spot() -> (usize, usize) {
        (2, 0)
    }

    /// Kuhn's one-phase story: a broad prior sharpening toward the king.
    fn curves(phase: usize) -> (Vec<f64>, Vec<f64>) {
        let sharpen = 0.5 + 0.5 * phase.min(1) as f64;
        let series = (0..crate::BUCKETS)
            .map(|k| {
                let x = k as f64 / 31.0;
                (-((x - 0.5 - 0.35 * sharpen).powi(2)) / (2.0 * (0.2 - 0.1 * sharpen).powi(2))).exp()
            })
            .collect::<Vec<f64>>();
        (series.clone(), series)
    }
}

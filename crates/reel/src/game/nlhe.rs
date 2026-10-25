use super::*;
use ::nlhe::NlheEdge;
use ::nlhe::NlheGame;
use ::nlhe::NlheTurn;
use deuce::*;
use kicker::Action;
use kicker::Edge;
use kicker::Odds;
use kicker::Recall;
use kicker::Turn;
use kicker::Witness;
use pokerkit::Probability;

/// The demo's chosen line: 3bb open · call, check · c-bet · call,
/// check · bet · raise · call, bet · jam · fold — onto J♠T♠3♦ K♠ 7♦.
const LINE: [Edge; 12] = [
    Edge::Open(3),
    Edge::Call,
    Edge::Check,
    Edge::Raise(Odds::new(1, 1)),
    Edge::Call,
    Edge::Check,
    Edge::Raise(Odds::new(1, 1)),
    Edge::Raise(Odds::new(2, 1)),
    Edge::Call,
    Edge::Raise(Odds::new(1, 1)),
    Edge::Shove,
    Edge::Fold,
];

/// The demo's rigged deal: hero holds A♠Q♠ on J♠T♠3♦ K♠ 7♦.
const DEAL: &str = "As Qs Js Ts 3d Ks 7d";

impl Scenic for NlheGame {
    fn demo() -> Demo<Self> {
        let cards = DEAL
            .split_whitespace()
            .map(|c| Card::try_from(c).expect("valid card literal"))
            .collect::<Vec<Card>>();
        let witness =
            LINE.iter()
                .fold(Witness::from((Turn::Choice(0), kicker::Arrangement::from(cards))), |witness, edge| {
                    let action = witness.head().snap(witness.head().actionize(*edge));
                    if witness.can_push(&action) { witness.push(action) } else { witness }
                });
        let states = witness.states();
        let steps = witness
            .actions()
            .iter()
            .enumerate()
            .filter(|(_, action)| action.is_choice())
            .map(|(i, _)| {
                let game = states[i];
                let aggro = witness
                    .actions()
                    .iter()
                    .take(i)
                    .rev()
                    .copied()
                    .take_while(Action::is_choice)
                    .filter(|action| matches!(action, Action::Raise(_) | Action::Shove(_)))
                    .count();
                let menu = Nlhe::arrange(game.choices(aggro).into_iter().collect());
                let pick = Slot::new(
                    menu.iter()
                        .position(|e| *e == witness.history()[i])
                        .expect("taken edge on menu"),
                );
                Step::new(NlheGame::from(game), menu.into_iter().map(NlheEdge::from).collect(), pick)
            })
            .collect();
        Nlhe::settle(&witness)
            .into_iter()
            .fold(Demo::new(NlheTurn::from(Turn::Choice(0)), steps), Demo::settled)
    }

    fn reveals(&self, hero: Self::T) -> Vec<Token> {
        let street = self.street();
        let mut cards = if street == Street::Pref {
            let seat = match Turn::from(hero) {
                Turn::Choice(seat) => seat,
                _ => 0,
            };
            (*self.sweat_at(seat).pocket()).collect::<Vec<Card>>()
        } else {
            kicker::Game::from(*self)
                .board()
                .cards()
                .skip(match street {
                    Street::Turn => Street::Flop.n_revealed(),
                    Street::Rive => Street::Flop.n_revealed() + Street::Turn.n_revealed(),
                    _ => 0,
                })
                .take(street.n_revealed())
                .collect::<Vec<Card>>()
        };
        cards.sort_by_key(|card| std::cmp::Reverse(card.rank()));
        cards.into_iter().map(Nlhe::token).collect()
    }

    fn phase(&self) -> String {
        self.street().label().to_uppercase()
    }

    fn badge(turn: Self::T) -> String {
        match Turn::from(turn) {
            Turn::Choice(0) => "BTN".into(),
            Turn::Choice(_) => "BB".into(),
            _ => "?".into(),
        }
    }

    fn deed(edge: &Self::E) -> Deed {
        match Edge::from(*edge) {
            Edge::Draw => Deed::Draw,
            Edge::Fold => Deed::Fold,
            Edge::Check => Deed::Check,
            Edge::Call => Deed::Call,
            Edge::Open(_) => Deed::Open,
            Edge::Raise(odds) => Deed::Raise(Probability::from(odds)),
            Edge::Shove => Deed::Shove,
        }
    }

    fn glyph(edge: &Self::E) -> String {
        match Edge::from(*edge) {
            Edge::Draw => "?".into(),
            Edge::Fold => "-".into(),
            Edge::Check => "•".into(),
            Edge::Call => "=".into(),
            Edge::Shove => "!".into(),
            Edge::Open(n) => format!("{n}bb"),
            Edge::Raise(odds) => format!("{}:{}", odds.numer(), odds.denom()),
        }
    }
}

/// Rank order of the range grid, ace-high first — the classic 169 layout.
const RANKS: &str = "AKQJT98765432";

/// Per-phase strength thresholds: hero stays wide, villain collapses.
const HERO: [f64; 4] = [0.26, 0.32, 0.38, 0.45];

/// Villain per-phase thresholds before board-awareness.
const VILLAIN: [f64; 4] = [0.33, 0.42, 0.52, 0.62];

/// Ranks on the demo board (K, J, T, 7, 3 as 14-down indices).
const BOARD: [usize; 5] = [13, 11, 10, 7, 3];

/// How much the board shapes villain's range per phase.
const MIX: [f64; 4] = [0.0, 0.55, 0.8, 1.0];

impl Atlas for NlheGame {
    fn grid() -> (usize, usize) {
        (13, 13)
    }

    fn cell(col: usize, row: usize) -> String {
        let (hi, lo) = (row.min(col), row.max(col));
        format!("{}{}", &RANKS[hi..=hi], &RANKS[lo..=lo])
    }

    fn weight(col: usize, row: usize, phase: usize, hero: bool) -> Probability {
        let strong = Nlhe::strength(row, col);
        let bar = if hero { HERO[phase] } else { VILLAIN[phase] };
        let kept = Nlhe::smoothstep(strong, bar - 0.1, bar + 0.1);
        if hero {
            kept as Probability
        } else {
            let mix = MIX[phase];
            (kept * ((1.0 - mix) + mix * (0.28 + 0.72 * Nlhe::affinity(row, col)))) as Probability
        }
    }

    fn spot() -> (usize, usize) {
        (2, 0)
    }

    /// The demo's per-phase equity story: broad preflop, bimodal draw on the
    /// flop, spiking to the nuts by the river.
    fn curves(phase: usize) -> (Vec<f64>, Vec<f64>) {
        let obs = [
            Nlhe::bump(0.55, 0.16, 0.7),
            Nlhe::mix(Nlhe::bump(0.34, 0.11, 0.6), Nlhe::bump(0.74, 0.1, 0.85)),
            Nlhe::mix(Nlhe::bump(0.93, 0.05, 1.0), Nlhe::bump(0.55, 0.09, 0.18)),
            Nlhe::bump(0.965, 0.035, 1.0),
        ];
        let abs = [
            Nlhe::bump(0.53, 0.2, 0.55),
            Nlhe::mix(Nlhe::bump(0.36, 0.14, 0.5), Nlhe::bump(0.72, 0.13, 0.62)),
            Nlhe::mix(Nlhe::bump(0.9, 0.07, 0.8), Nlhe::bump(0.52, 0.12, 0.22)),
            Nlhe::bump(0.94, 0.05, 0.82),
        ];
        (obs[phase].clone(), abs[phase].clone())
    }
}

/// Namespace for the nlhe routines.
pub(crate) struct Nlhe;

impl Nlhe {
    /// The demo's settlement, read off the terminal witness exactly as the client's tree reads it:
    /// the final pot and what hero walked away with, under the seat that took it down.
    fn settle(witness: &Witness) -> Option<Tally> {
        let game = witness.head();
        if !game.must_stop() {
            return None;
        }
        let hero = witness.turn().position();
        let seats = game.seats();
        let winner = (!game.is_showdown())
            .then(|| seats.iter().position(|seat| seat.state().is_active()))
            .flatten();
        let net = winner.map(|w| if w == hero { game.pot() - seats[hero].spent() } else { -seats[hero].spent() });
        let made = (witness.seen().street() != Street::Pref)
            .then(|| Strength::from(Hand::add(*witness.seen().pocket(), *witness.seen().public())).ranking());
        Some(Tally::new(
            winner.map_or_else(|| "?".into(), |w| NlheGame::badge(NlheTurn::from(Turn::Choice(w)))),
            if game.is_showdown() { "SHOWDOWN" } else { "FOLD" },
            vec![
                Token::new(vec![Rune::new(game.pot().to_string(), Tint::Dark, Face::Data)]),
                Token::new(vec![Rune::new(
                    net.map_or_else(|| made.map_or("?", |m| m.glyph()).to_string(), |n| format!("{n:+}")),
                    Tint::Gold,
                    Face::Data,
                )]),
            ],
        ))
    }

    /// A card face as a token: bold rank, suit pip inked by color.
    fn token(card: Card) -> Token {
        let tint = match card.suit() {
            Suit::D | Suit::H => Tint::Red,
            Suit::C | Suit::S => Tint::Dark,
        };
        let suit = match card.suit() {
            Suit::C => "♣",
            Suit::D => "♦",
            Suit::H => "♥",
            Suit::S => "♠",
        };
        Token::new(vec![
            Rune::new(card.rank().to_string(), Tint::Dark, Face::Text),
            Rune::new(suit, tint, Face::Pip),
        ])
    }

    /// Menus in display order: fold, check, call, opens ascending, raises
    /// ascending, shove — the same order the client tree renders.
    fn arrange(mut edges: Vec<Edge>) -> Vec<Edge> {
        edges.retain(Edge::is_choice);
        edges.sort_by(|a, b| Nlhe::order(a).partial_cmp(&Nlhe::order(b)).expect("total order"));
        edges
    }

    fn order(edge: &Edge) -> (u8, f32) {
        match edge {
            Edge::Fold => (0, 0.0),
            Edge::Check => (1, 0.0),
            Edge::Call => (2, 0.0),
            Edge::Open(n) => (3, f32::from(*n)),
            Edge::Raise(odds) => (4, Probability::from(*odds)),
            Edge::Shove => (5, 0.0),
            Edge::Draw => (6, 0.0),
        }
    }

    fn bump(center: f64, spread: f64, height: f64) -> Vec<f64> {
        (0..crate::BUCKETS)
            .map(|k| height * (-((k as f64 / 31.0 - center).powi(2)) / (2.0 * spread * spread)).exp())
            .collect()
    }

    fn mix(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
        a.into_iter().zip(b).map(|(x, y)| x + y).collect()
    }

    /// Preflop hand strength on the 169 grid, in [0, 1].
    fn strength(row: usize, col: usize) -> f64 {
        let hi = 14 - row.min(col);
        let lo = 14 - row.max(col);
        if row == col {
            return 0.52 + (hi as f64 - 2.0) / 12.0 * 0.48;
        }
        let mut strong = (hi as f64 * 0.9 + lo as f64 * 0.55) / (14.0 * 0.9 + 13.0 * 0.55) * 0.74;
        if col > row {
            strong += 0.06;
        }
        match hi - lo {
            1 => strong += 0.045,
            2 => strong += 0.02,
            _ => {}
        }
        if hi >= 13 && lo >= 10 {
            strong += 0.05;
        }
        strong.clamp(0.0, 1.0)
    }

    /// How connected a holding is to the demo board — flush, straight, pair ties.
    fn affinity(row: usize, col: usize) -> f64 {
        let hi = 14 - row.min(col);
        let lo = 14 - row.max(col);
        let mut bond = 0.0f64;
        if BOARD.contains(&hi) {
            bond += 0.45;
        }
        if BOARD.contains(&lo) {
            bond += 0.32;
        }
        if hi >= 10 && lo >= 10 {
            bond += 0.3;
        }
        if col > row {
            bond += 0.22;
        }
        if row == col && hi >= 8 {
            bond += 0.3;
        }
        if row == col && BOARD.contains(&hi) {
            bond += 0.45;
        }
        bond.min(1.0)
    }

    fn smoothstep(x: f64, lo: f64, hi: f64) -> f64 {
        let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

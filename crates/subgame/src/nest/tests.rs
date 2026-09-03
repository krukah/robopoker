//! Wrapper-machinery tests on a toy `Augmentable` game — the payoff of going
//! generic: the nest bookkeeping is exercised with no blueprint, no DB, no NLHE.
#![cfg(test)]
use super::*;
use mccfr::*;
use monge::Support;
use pokerkit::*;

// ── a minimal Augmentable game ───────────────────────────────────────

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum ToyTurn {
    P0,
    P1,
    Chance,
    Terminal,
}
impl CfrTurn for ToyTurn {
    fn chance() -> Self {
        Self::Chance
    }

    fn terminal() -> Self {
        Self::Terminal
    }

    fn players() -> usize {
        2
    }
}
impl From<usize> for ToyTurn {
    fn from(i: usize) -> Self {
        if i.is_multiple_of(2) { Self::P0 } else { Self::P1 }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum ToyEdge {
    Check,
    Bet,
}
impl Support for ToyEdge {}
impl CfrEdge for ToyEdge {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ToyGame {
    pot: i32,
}
impl CfrGame for ToyGame {
    type E = ToyEdge;
    type T = ToyTurn;

    fn root() -> Self {
        Self { pot: 1 }
    }

    fn turn(&self) -> Self::T {
        ToyTurn::P0
    }

    fn apply(&self, edge: Self::E) -> Self {
        Self {
            pot: self.pot + i32::from(matches!(edge, ToyEdge::Bet)),
        }
    }

    fn payoff(&self, _: Self::T) -> Utility {
        0.0
    }
}
impl Augmentable for ToyGame {
    type Off = i32;

    fn augment(&self, off: Self::Off) -> Self {
        Self { pot: self.pot + off }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct ToyInfo;
impl CfrInfo for ToyInfo {
    type E = ToyEdge;
    type T = ToyTurn;
    type X = ToyInfo;
    type Y = ();

    fn public(&self) -> Self::X {
        *self
    }

    fn secret(&self) -> Self::Y {}
}
impl CfrPublic for ToyInfo {
    type E = ToyEdge;
    type T = ToyTurn;

    fn choices(&self) -> impl Iterator<Item = Self::E> + use<> {
        [ToyEdge::Check, ToyEdge::Bet].into_iter()
    }

    fn subgame(&self) -> Vec<Self::E> {
        vec![ToyEdge::Check]
    }
}

// ── tests ────────────────────────────────────────────────────────────

#[test]
fn edge_accessors_and_defaults() {
    let game: NestEdge<ToyEdge, i32> = NestEdge::Game(ToyEdge::Bet);
    let off: NestEdge<ToyEdge, i32> = NestEdge::Off(50);
    assert_eq!(game.game(), Some(ToyEdge::Bet));
    assert_eq!(game.offtree(), None);
    assert_eq!(off.offtree(), Some(50));
    assert_eq!(off.game(), None);
    assert_eq!(off.default_regret(), 0.0);
    assert_eq!(off.default_policy(), 0.0);
}

#[test]
fn entry_public_splices_off_only_at_entry() {
    let base = NestPublic::<ToyInfo, i32>::Game(ToyInfo).choices().count();
    let entry = NestPublic::<ToyInfo, i32>::Entry(ToyInfo, 50)
        .choices()
        .collect::<Vec<_>>();
    assert_eq!(entry.len(), base + 1);
    assert!(entry.contains(&NestEdge::Off(50)));
    assert!(
        NestPublic::<ToyInfo, i32>::Game(ToyInfo)
            .choices()
            .all(|e| e.offtree().is_none())
    );
}

#[test]
fn subgame_wraps_canonical_edges() {
    assert_eq!(NestPublic::<ToyInfo, i32>::Game(ToyInfo).subgame(), vec![NestEdge::Game(ToyEdge::Check)],);
}

#[test]
fn info_three_way_tagging() {
    assert_eq!(NestInfo::<ToyInfo, i32>::Augmented(ToyInfo).inner(), ToyInfo);
    assert_ne!(NestInfo::<ToyInfo, i32>::Augmented(ToyInfo), NestInfo::Game(ToyInfo));
    assert!(matches!(NestInfo::<ToyInfo, i32>::Entry(ToyInfo, 9).public(), NestPublic::Entry(_, 9)));
    assert!(matches!(NestInfo::<ToyInfo, i32>::Game(ToyInfo).public(), NestPublic::Game(_)));
    assert!(matches!(NestInfo::<ToyInfo, i32>::Augmented(ToyInfo).public(), NestPublic::Game(_)));
}

#[test]
fn game_entry_flag_set_then_cleared() {
    let entry = NestGame::entry(ToyGame::root(), 50);
    assert!(entry.is_entry());
    assert_eq!(entry.off(), 50);
    let next = entry.apply(NestEdge::Game(ToyEdge::Check));
    assert!(!next.is_entry());
    assert_eq!(next.off(), 50, "off-tree payload threads through apply");
}

#[test]
fn canonical_edge_delegates_to_inner_apply() {
    let after = NestGame::new(ToyGame::root(), 50).apply(NestEdge::Game(ToyEdge::Bet));
    assert_eq!(after.inner().pot, ToyGame::root().apply(ToyEdge::Bet).pot);
}

#[test]
fn offtree_edge_calls_augment() {
    let after = NestGame::entry(ToyGame::root(), 50).apply(NestEdge::Off(50));
    assert_eq!(after.inner().pot, ToyGame::root().augment(50).pot);
    assert!(!after.is_entry());
}

#[test]
fn tag_table() {
    assert!(matches!(tag(ToyInfo, &NestGame::entry(ToyGame::root(), 7), false), NestInfo::Entry(_, 7)));
    assert!(matches!(tag(ToyInfo, &NestGame::entry(ToyGame::root(), 7), true), NestInfo::Entry(_, 7)));
    assert!(matches!(tag(ToyInfo, &NestGame::new(ToyGame::root(), 7), true), NestInfo::Augmented(_)));
    assert!(matches!(tag(ToyInfo, &NestGame::new(ToyGame::root(), 7), false), NestInfo::Game(_)));
}

//! Wrapper-machinery tests on a toy `Augmentable` game — the payoff of going
//! generic: the nest bookkeeping is exercised with no blueprint, no DB, no NLHE.
#![cfg(test)]
use super::*;
use crate::nest::fixture::*;
use crate::*;
use mccfr::*;

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
    let base = NestPublic::<ToyInfo, i32>::Game(ToyInfo).menu().count();
    let entry = NestPublic::<ToyInfo, i32>::Entry(ToyInfo, 50)
        .menu()
        .collect::<Vec<_>>();
    assert_eq!(entry.len(), base + 1);
    assert!(entry.contains(&NestEdge::Off(50)));
    assert!(
        NestPublic::<ToyInfo, i32>::Game(ToyInfo)
            .menu()
            .all(|e| e.offtree().is_none())
    );
}

#[test]
fn info_three_way_tagging() {
    let augmented = ToyInfo.augmented::<i32>();
    assert_ne!(augmented, NestInfo::<ToyInfo, i32>::new(NestPublic::Game(ToyInfo), ()));
    assert!(matches!(augmented.public(), NestPublic::Augmented(ToyInfo)));
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
    assert_eq!(after.inner().pot(), ToyGame::root().apply(ToyEdge::Bet).pot());
}

#[test]
fn offtree_edge_calls_augment() {
    let after = NestGame::entry(ToyGame::root(), 50).apply(NestEdge::Off(50));
    assert_eq!(after.inner().pot(), ToyGame::root().augment(50).pot());
    assert!(!after.is_entry());
}

#[test]
fn tag_table() {
    assert!(matches!(
        NestPublic::tag(ToyInfo, &NestGame::entry(ToyGame::root(), 7), false),
        NestPublic::Entry(_, 7)
    ));
    assert!(matches!(
        NestPublic::tag(ToyInfo, &NestGame::entry(ToyGame::root(), 7), true),
        NestPublic::Entry(_, 7)
    ));
    assert!(matches!(
        NestPublic::tag(ToyInfo, &NestGame::new(ToyGame::root(), 7), true),
        NestPublic::Augmented(_)
    ));
    assert!(matches!(NestPublic::tag(ToyInfo, &NestGame::new(ToyGame::root(), 7), false), NestPublic::Game(_)));
}

#[test]
fn info_count_matches_choices() {
    assert_eq!(CfrPublic::count(&ToyInfo), CfrInfo::menu(&ToyInfo).count());
    [
        NestPublic::Entry(ToyInfo, 50),
        NestPublic::Augmented(ToyInfo),
        NestPublic::Game(ToyInfo),
    ]
    .map(|x| NestInfo::<ToyInfo, i32>::new(x, ()))
    .iter()
    .for_each(|i| assert_eq!(i.public().count(), i.menu().count(), "{i:?}"));
}

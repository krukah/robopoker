use super::*;
use rbp_cards::*;
use rbp_core::Arbitrary;
use rbp_gameplay::*;
use rbp_mccfr::*;

type NlheTree = Tree<NlheTurn, NlheEdge, NlheGame, NlheInfo>;
type NlheBranch = Branch<NlheEdge, NlheGame>;

/// NLHE information set: what a player knows at a decision point.
///
/// Combines public state (subgame history + choices) with private state
/// (abstraction bucket encoding hand strength).
///
/// # Information Boundary
///
/// | Context | Recall | Info | Secret |
/// |---------|--------|------|--------|
/// | Training (CFR) | Perfect (both hands) | NlheInfo | Abstraction bucket |
/// | Inference (play) | Partial (hero only) | NlheInfo | Abstraction bucket |
///
/// At **training time**, `Perfect` recall knows both players' cards but CFR
/// only indexes by `NlheInfo` (public edges + private bucket). The secret
/// bucket is derived from the acting player's observation at each node.
///
/// At **inference time**, `Partial` recall knows only hero's cards. Strategy
/// lookup uses `NlheInfo::from((&recall, abstraction))` for policy queries.
///
/// # Action Space
///
/// Available actions are stored in `NlhePublic` and become part of the info set's
/// identity. Two states with identical `(subgame, secret)` but different available
/// actions are distinct info sets. This handles cases where different game tree
/// paths lead to different betting options due to stack constraints.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NlheInfo {
    public: NlhePublic,
    secret: NlheSecret,
}

impl NlheInfo {
    /// The current street (from secret's embedded street).
    pub fn street(&self) -> Street {
        self.secret.street()
    }
    /// Depth (trailing aggressive actions) for bet sizing grid selection.
    pub fn aggression(&self) -> usize {
        self.public.aggression()
    }
    /// Current-street historical edges as a Path.
    pub fn subgame(&self) -> Path {
        self.public.subgame()
    }
    /// Available actions at this decision point.
    pub fn choices(&self) -> Path {
        self.public.choices().into_iter().map(Edge::from).collect()
    }
    /// The private abstraction bucket.
    pub fn bucket(&self) -> NlheSecret {
        self.secret
    }
}

impl std::fmt::Display for NlheInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}>>{}", self.street(), self.subgame(), self.secret)
    }
}

impl CfrInfo for NlheInfo {
    type X = NlhePublic;
    type Y = NlheSecret;
    type E = NlheEdge;
    type T = NlheTurn;
    fn public(&self) -> Self::X {
        self.public
    }
    fn secret(&self) -> Self::Y {
        self.secret
    }
}

// =============================================================================
// Construction methods
// =============================================================================

impl<R> From<(&R, Abstraction)> for NlheInfo
where
    R: Recall,
{
    /// Constructs info set for policy lookup from recall history.
    ///
    /// Critically, `choices()` is computed from the **edge-derived** game state,
    /// not the actual action-derived state. This ensures consistency with training:
    /// - Training applies edges via `CfrGame::apply(edge)` → canonical chip amounts
    /// - Inference must do the same, even if user took off-grid bet sizes
    ///
    /// Without this, a custom raise of 19 chips (snapped to "1:2 pot" edge) would
    /// produce different `choices()` than training, causing info set mismatch.
    fn from((recall, secret): (&R, Abstraction)) -> Self {
        let subgame = recall.subgame();
        let canonical = recall
            .history()
            .into_iter()
            .map(NlheEdge::from)
            .fold(NlheGame::root(), |game, edge| CfrGame::apply(&game, edge));
        let choices = Game::from(canonical).choices(subgame.aggression());
        Self::from((subgame, secret, choices))
    }
}

impl From<(Path, Abstraction, Path)> for NlheInfo {
    fn from((subgame, secret, choices): (Path, Abstraction, Path)) -> Self {
        let subgame = subgame
            .into_iter()
            .rev()
            .take_while(|e| e.is_choice())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Path>();
        let public = NlhePublic::new(subgame, choices);
        let secret = NlheSecret::from(secret);
        Self { public, secret }
    }
}

impl From<(&NlheEncoder, &NlheTree, NlheBranch)> for NlheInfo {
    /// Creates an info set during tree expansion.
    /// Used by [`Encoder::info`] to compute info for new tree nodes.
    /// Collects current-street edge history from tree traversal.
    fn from((encoder, tree, leaf): (&NlheEncoder, &NlheTree, NlheBranch)) -> Self {
        let (edge, ref game, head) = leaf;
        let subgame = std::iter::once(edge)
            .chain(tree.at(head).map(|(_, e)| e))
            .take_while(|e| e.is_choice())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(Edge::from)
            .collect::<Path>();
        let choices = game.as_ref().choices(subgame.aggression());
        let secret = NlheSecret::from(encoder.abstraction(&game.sweat()));
        let public = NlhePublic::new(subgame, choices);
        Self { public, secret }
    }
}

impl Arbitrary for NlheInfo {
    fn random() -> Self {
        use rand::prelude::IndexedRandom;
        use std::ops::Not;
        loop {
            let street = Street::random();
            let (mut game, mut depth) = (Game::root(), 0usize);
            let subgame = std::iter::from_fn(|| {
                (game.street() < street || game.turn().is_chance()).then(|| {
                    game.choices(depth)
                        .into_iter()
                        .filter(|e| !e.is_folded())
                        .collect::<Vec<_>>()
                        .choose(&mut rand::rng())
                        .map(|&e| {
                            game = game.apply(game.snap(game.actionize(e)));
                            depth = e
                                .is_chance()
                                .not()
                                .then(|| depth + e.is_aggro() as usize)
                                .unwrap_or(0);
                            e
                        })
                })?
            })
            .filter(|e| e.is_choice())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .take_while(|e| e.is_choice())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Path>();
            let choices = game.choices(subgame.aggression());
            if choices.length() > 0 {
                let secret = NlheSecret::from(Abstraction::from(street));
                return Self {
                    public: NlhePublic::new(subgame, choices),
                    secret,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistent_edge_collection_flop() {
        let game = Game::root();
        let game = game.apply(Action::Call(1));
        let game = game.apply(Action::Check);
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        assert_eq!(game.street(), Street::Flop);
        let depth = 0;
        let learned = game
            .choices(depth)
            .into_iter()
            .map(NlheEdge::from)
            .collect::<Vec<NlheEdge>>();
        for amount in game.to_raise()..game.to_shove() {
            let action = Action::Raise(amount);
            if game.is_allowed(&action) {
                let edge = NlheEdge::from(game.edgify(action, depth));
                assert!(
                    learned.contains(&edge),
                    "Raise({}) -> {:?} not in trained edges for flop depth={}",
                    amount,
                    edge,
                    depth
                );
            }
        }
    }

    #[test]
    fn consistent_edge_collection_turn() {
        let game = Game::root();
        let game = game.apply(Action::Call(1));
        let game = game.apply(Action::Check);
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        let game = game.apply(Action::Check);
        let game = game.apply(Action::Check);
        let turn = game.deck().deal(Street::Flop);
        let game = game.apply(Action::Draw(turn));
        assert_eq!(game.street(), Street::Turn);
        let depth = 0;
        let trained = game
            .choices(depth)
            .into_iter()
            .map(NlheEdge::from)
            .collect::<Vec<NlheEdge>>();
        for amount in game.to_raise()..game.to_shove() {
            let action = Action::Raise(amount);
            let edge = NlheEdge::from(game.edgify(action, depth));
            assert!(
                trained.contains(&edge),
                "Raise({}) -> {:?} not in trained edges for turn depth={}",
                amount,
                edge,
                depth
            );
        }
    }

    #[test]
    fn consistent_aggression_calculation() {
        let path1: Path = [
            Edge::Draw,
            Edge::Raise(Odds::new(1, 1)),
            Edge::Call,
            Edge::Draw,
            Edge::Check,
            Edge::Raise(Odds::new(1, 2)),
            Edge::Shove,
        ]
        .into_iter()
        .collect();
        assert_eq!(2, path1.aggression());
        let path2: Path = [
            Edge::Raise(Odds::new(1, 1)),
            Edge::Raise(Odds::new(1, 2)),
            Edge::Shove,
        ]
        .into_iter()
        .collect();
        assert_eq!(3, path2.aggression());
        let path3: Path = [
            Edge::Check, //
            Edge::Call,
            Edge::Check,
        ]
        .into_iter()
        .collect();
        assert_eq!(0, path3.aggression());
    }

    #[test]
    fn roundtrip_string_serialization() {
        let info = NlheInfo::random();
        let deserialized = NlheInfo::from((
            Path::from(i64::from(info.subgame())),
            Abstraction::from(i16::from(info.bucket())),
            Path::from(i64::from(info.choices())),
        ));
        assert_eq!(info.subgame(), deserialized.subgame());
        assert_eq!(info.street(), deserialized.street());
        assert_eq!(info.bucket(), deserialized.bucket());
        assert_eq!(info.choices(), deserialized.choices());
    }

    #[test]
    fn roundtrip_edgify_actionize() {
        let depth = 0;
        let game = Game::root();
        for a1 in game.legal() {
            let e1 = NlheEdge::from(game.edgify(a1, depth));
            let a2 = game.actionize(Edge::from(e1));
            let e2 = NlheEdge::from(game.edgify(a2, depth));
            assert_eq!(
                e1, e2,
                "Roundtrip failed: {:?} -> {:?} -> {:?} -> {:?}",
                a1, e1, a2, e2
            );
        }
    }

    #[test]
    fn raises_into_edges() {
        let game = Game::root();
        let depth = 0;
        let trained = game
            .choices(depth)
            .into_iter()
            .map(NlheEdge::from)
            .collect::<std::collections::HashSet<NlheEdge>>();
        for amount in game.to_raise()..game.to_shove() {
            let raise = Action::Raise(amount);
            let edge = NlheEdge::from(game.edgify(raise, depth));
            assert!(trained.contains(&edge), "amount={} edge={:?}", amount, edge);
        }
    }

    #[test]
    fn from_path_filters_to_current_street() {
        let recall = Partial::from((Turn::Choice(0), Arrangement::from(Street::Flop)))
            .push(Action::Call(1))
            .push(Action::Check)
            .push(Action::Raise(3))
            .push(Action::Raise(9))
            .push(Action::Call(6));
        let abs = Abstraction::random();
        let info = NlheInfo::from((recall.subgame(), abs, recall.choices()));
        let current = recall
            .subgame()
            .into_iter()
            .rev()
            .take_while(|e| e.is_choice())
            .collect::<Path>()
            .rev()
            .collect::<Path>();
        assert_eq!(info.subgame(), current);
    }
    #[test]
    fn canonical_choices_from_edge_reconstruction() {
        // Build a recall with arbitrary (potentially off-grid) actions
        let recall = Partial::from((Turn::Choice(0), Arrangement::from(Street::Flop)))
            .push(Action::Call(1))
            .push(Action::Check)
            .push(Action::Raise(7)); // arbitrary amount
        // The actual game state has pot reflecting 7-chip raise
        let _actual_game = recall.head();
        // The canonical game state reconstructs from edges
        let canonical_game = recall
            .history()
            .into_iter()
            .map(NlheEdge::from)
            .fold(NlheGame::root(), |g, e| CfrGame::apply(&g, e));
        let canonical_choices = Game::from(canonical_game).choices(recall.subgame().aggression());
        // NlheInfo should use canonical choices, not actual choices
        let abs = Abstraction::from(Street::Flop);
        let info = NlheInfo::from((&recall, abs));
        assert_eq!(
            info.choices(),
            canonical_choices,
            "info should use canonical choices"
        );
        // Only assert actual != canonical if they're actually different (depends on snapping)
        // The key assertion is that info uses canonical, regardless of whether they differ
    }
}

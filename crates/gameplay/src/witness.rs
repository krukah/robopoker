use super::*;
use rbp_cards::*;
use rbp_core::*;
use std::ops::Not;

/// Perfect-recall game history from a **single player's** perspective.
///
/// While [`Game`] is memoryless, `Witness` tracks the complete action sequence
/// from the start of a hand. This is the primary type for **inference time**:
/// hero knows their own cards but not the opponent's.
///
/// # Information Boundary
///
/// | Type | Perspective | Used For |
/// |------|-------------|----------|
/// | `Witness` | Hero only (own cards) | Inference, UI, opponent iteration |
/// | `Perfect` | God's view (both hands) | Training CFR traversal |
///
/// # Key Operations
///
/// - `NlheInfo::from((&witness, abstraction))` for strategy lookup
/// - `Perfect::from((&witness, hole))` for opponent modeling
/// - `witness.histories()` → iterate all possible opponent hands
///
/// # Structure
///
/// - `pov` — Which player's perspective we're tracking
/// - `actions` — Action sequence excluding blinds (bets, draws)
/// - `reveals` — The card arrangement for this hand (hero's observation)
///
/// # Invariants
///
/// Carries initial stacks and dealer position for correct state reconstruction.
/// Blinds are constant and handled by `root()` returning a POST-blind state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Witness {
    pov: Turn,
    #[serde(default = "default_stacks")]
    stacks: [Chips; N],
    #[serde(default)]
    dealer: Position,
    actions: Vec<Action>,
    reveals: Arrangement,
}
fn default_stacks() -> [Chips; N] {
    [STACK; N]
}

impl Arbitrary for Witness {
    fn random() -> Self {
        Self::initial(Turn::Choice(0))
    }
}
impl Witness {
    /// Stacks at the start of the hand.
    pub fn stacks(&self) -> [Chips; N] {
        self.stacks
    }
    /// Dealer position for the hand.
    pub fn dealer(&self) -> Position {
        self.dealer
    }
}

impl Witness {
    /// Creates a recall at the start of a hand (blinds posted, no decisions).
    pub fn initial(pov: Turn) -> Self {
        Self {
            pov,
            stacks: [STACK; N],
            dealer: 0,
            actions: Vec::new(),
            reveals: Arrangement::from(Street::Pref),
        }
    }
    /// Creates a recall with explicit stacks and dealer (for live gameplay).
    pub fn initial_with(pov: Turn, reveals: Arrangement, stacks: [Chips; N], dealer: Position) -> Self {
        Self {
            pov,
            stacks,
            dealer,
            actions: Vec::new(),
            reveals,
        }
    }
    /// Returns a new recall with the given perspective.
    pub fn with_pov(&self, pov: Turn) -> Self {
        Self {
            pov,
            stacks: self.stacks,
            dealer: self.dealer,
            actions: self.actions.clone(),
            reveals: self.reveals,
        }
    }
}

impl Recall for Witness {
    fn root(&self) -> Game {
        Game::blinds().into_iter().fold(self.base(), |mut g, a| g.consume(a))
    }

    fn actions(&self) -> &[Action] {
        &self.actions
    }
}

/// Strategy lookup methods.
impl Witness {
    /// Returns all betting edges (Open or Raise) available at the current state.
    pub fn betting_edges(&self) -> Vec<Edge> {
        let game = self.head();
        self.choices()
            .into_iter()
            .filter(|edge| matches!(edge, Edge::Open(_) | Edge::Raise(_)))
            .filter(|edge| matches!(game.actionize(*edge), Action::Raise(_)))
            .collect()
    }

    /// Enumerates all possible opponent hands with complete-info histories.
    ///
    /// Yields (villain observation, [`Perfect`] history) pairs representing
    /// the uniform prior over villain holdings. Each villain observation
    /// shares hero's board but has a different pocket drawn from cards not
    /// visible to hero. The [`Perfect`] carries both players' cards, enabling
    /// downstream reach computation via blueprint policy lookup.
    pub fn possibilities(&self) -> Vec<(Observation, Perfect)> {
        self.seen()
            .opponents()
            .map(|villain| {
                let hole = Hole::from(*villain.pocket());
                (villain, Perfect::from((self, hole)))
            })
            .collect()
    }
}

/// Constructs recall from a POV and arrangement (no decisions yet).
/// Uses default stacks and dealer (analysis mode).
impl From<(Turn, Arrangement)> for Witness {
    fn from((pov, reveals): (Turn, Arrangement)) -> Self {
        Self {
            pov,
            stacks: [STACK; N],
            dealer: 0,
            actions: Vec::new(),
            reveals,
        }
    }
}

/// random non-folding actions lead to this street
impl From<Street> for Witness {
    fn from(_: Street) -> Self {
        todo!()
    }
}

/// Lossy: per-street card assignment uses canonical [`Hand`] iteration
/// order. Prefer constructing via [`Witness::try_arrange`] when an
/// [`Arrangement`] with the actual deal order is available.
impl From<(Turn, Observation, Vec<Action>)> for Witness {
    fn from((pov, seen, actions): (Turn, Observation, Vec<Action>)) -> Self {
        Self::try_build(pov, seen, actions).expect("valid action sequence")
    }
}

impl Witness {
    /// Fallible constructor from (POV, observation, actions).
    ///
    /// Converts `seen` to an [`Arrangement`] via `Arrangement::from(seen)`,
    /// which assigns public cards to street slots using canonical [`Hand`]
    /// iteration order. This is a **lossy** conversion: the original deal
    /// order is not recoverable from an [`Observation`] alone.
    ///
    /// Prefer [`try_arrange`](Self::try_arrange) when the caller has an
    /// [`Arrangement`] that preserves the actual deal order.
    ///
    /// The `actions` parameter should NOT include blinds.
    pub fn try_build(pov: Turn, seen: Observation, actions: Vec<Action>) -> anyhow::Result<Self> {
        Self::try_arrange(pov, Arrangement::from(seen), actions)
    }
    /// Fallible constructor from (POV, arrangement, actions).
    ///
    /// Preferred over [`try_build`](Self::try_build) when an [`Arrangement`]
    /// with the correct per-street card assignment is available. The
    /// arrangement determines which cards [`revealed`](Self::revealed)
    /// attributes to each street and which [`Draw`](Action::Draw) actions
    /// `sprout` inserts at street boundaries.
    ///
    /// The `actions` parameter should NOT include blinds or draws —
    /// draws are auto-inserted by `sprout` based on the arrangement.
    pub fn try_arrange(pov: Turn, reveals: Arrangement, actions: Vec<Action>) -> anyhow::Result<Self> {
        Self::try_arrange_with(pov, reveals, [STACK; N], actions)
    }
    /// [`try_arrange`](Self::try_arrange) with explicit starting stacks.
    pub fn try_arrange_with(
        pov: Turn,
        reveals: Arrangement,
        stacks: [Chips; N],
        actions: Vec<Action>,
    ) -> anyhow::Result<Self> {
        actions.into_iter().try_fold(
            Self {
                pov,
                stacks,
                dealer: 0,
                actions: Vec::new(),
                reveals,
            },
            |r, a| r.try_push(a),
        )
    }
}

/// State reconstruction methods.
impl Witness {
    /// Returns the initial game state (before blinds, with hero's hole cards).
    pub fn base(&self) -> Game {
        Game::preblind(self.dealer, self.stacks).wipe(Hole::from(self.seen()))
    }
    /// The current betting street.
    pub fn street(&self) -> Street {
        self.head().street()
    }
    /// The street based on Draw actions in the action sequence.
    pub fn drawn(&self) -> Street {
        Street::from(self.actions.iter().filter(|a| a.is_chance()).count() as isize)
    }
    /// The player perspective for this recall.
    pub fn turn(&self) -> Turn {
        self.pov
    }
    /// The card arrangement for this recall.
    pub fn arr(&self) -> Arrangement {
        self.reveals
    }
    /// The observation (hole cards + board) for this recall.
    pub fn seen(&self) -> Observation {
        self.reveals.observation()
    }
    /// Resets to initial state (no decisions), preserving POV and cards.
    pub fn reset(&self) -> Self {
        Self {
            pov: self.turn(),
            stacks: self.stacks,
            dealer: self.dealer,
            reveals: self.reveals,
            actions: Vec::new(),
        }
    }
    /// Node index for graph traversal.
    pub fn cursor(&self) -> petgraph::graph::NodeIndex {
        petgraph::graph::NodeIndex::new(self.actions().len().saturating_sub(1))
    }
    /// Returns (position, action, street) for each action in the sequence.
    pub fn plays(&self) -> Vec<(Position, Action, Street)> {
        self.states()
            .windows(2)
            .zip(self.actions().iter().copied())
            .filter_map(|(pair, action)| {
                action
                    .is_choice()
                    .then(|| (pair[0].turn().position(), action, pair[0].street()))
            })
            .collect()
    }
    /// Finds the last aggressor on the final betting street.
    /// Returns None if no aggressive action was taken (all checks/calls).
    pub fn aggressor(&self) -> Option<Position> {
        self.plays()
            .into_iter()
            .filter_map(|(pos, action, _)| action.is_aggro().then_some(pos))
            .next_back()
    }
    /// Truncates actions to a specific street.
    pub fn truncate(&self, street: Street) -> Self {
        let pov = self.turn();
        let reveals = self.reveals;
        let actions = self
            .states()
            .into_iter()
            .skip(1)
            .zip(self.actions().iter().copied())
            .map(|(game, action)| (action, game))
            .collect::<Vec<(Action, Game)>>()
            .into_iter()
            .take_while(|(_, game)| game.street() <= street)
            .map(|(action, _)| action)
            .collect::<Vec<Action>>();
        let recall = Self {
            pov,
            stacks: self.stacks,
            dealer: self.dealer,
            reveals,
            actions,
        };
        recall.sprout()
    }

    /// Swaps the card arrangement, updating draw actions to match.
    pub fn replace(&self, reveals: Arrangement) -> Self {
        let mut actions = self.actions().to_vec();
        actions
            .iter_mut()
            .filter(|a| a.is_chance())
            .zip(reveals.draws())
            .for_each(|(old, new)| *old = new);
        Self {
            pov: self.turn(),
            stacks: self.stacks,
            dealer: self.dealer,
            actions,
            reveals,
        }
    }

    /// Player decisions (non-draw) for a specific street.
    pub fn decisions(&self, street: Street) -> Vec<Action> {
        let mut actions = Vec::new();
        let mut current = Street::Pref;
        for action in self.actions().iter().copied() {
            if action.is_chance() {
                current = current.next();
            } else if current == street {
                actions.push(action);
            }
        }
        actions
    }
    /// Returns all (Action, Edge, Street) tuples for choice nodes.
    pub fn yorrify(&self) -> Vec<(Action, Edge, Street)> {
        let mut current = Street::Pref;
        self.actions()
            .iter()
            .zip(self.history().iter())
            .filter_map(|(a, e)| {
                if a.is_chance() {
                    current = current.next();
                    None
                } else {
                    Some((*a, *e, current))
                }
            })
            .collect()
    }

    /// Community cards dealt so far (in deal order).
    pub fn board(&self) -> Vec<Card> {
        let street = self.head().street();
        Street::all()
            .iter()
            .skip(1)
            .filter(|s| **s <= street)
            .copied()
            .flat_map(|s| self.revealed(s))
            .collect()
    }

    /// Cards revealed on a specific street.
    pub fn revealed(&self, street: Street) -> Vec<Card> {
        self.reveals.revealed(street)
    }
    /// The canonical form of the observation.
    pub fn isomorphism(&self) -> Isomorphism {
        Isomorphism::from(self.seen())
    }

    /// True if no decisions have been made.
    pub fn empty(&self) -> bool {
        self.actions().is_empty()
    }

    /// True if observation's public cards match the dealt draw actions.
    pub fn aligned(&self) -> bool {
        *self.seen().public()
            == self
                .actions()
                .iter()
                .filter(|a| a.is_chance())
                .filter_map(Action::hand)
                .fold(Hand::empty(), Hand::add)
    }
}

/// Game tree traversal methods.
///
/// These methods provide granular access to the decision structure
/// for UI components that need to visualize the game tree.
impl Witness {
    /// Returns the aggression depth at a specific state index.
    ///
    /// Aggression is the count of trailing aggressive edges on the current street,
    /// computed from history[0..state_idx].
    fn aggression_at(&self, i: usize) -> usize {
        self.history()
            .iter()
            .take(i)
            .rev()
            .take_while(|e| e.is_choice())
            .filter(|e| e.is_aggro())
            .count()
    }
    /// Returns the available edges at a specific state index.
    ///
    /// For Choice turns: betting edges (Fold, Call, Raise variants)
    /// For Chance turns: [Edge::Draw]
    /// For Terminal: empty
    fn edges_at(&self, i: usize) -> Vec<Edge> {
        self.states().get(i).map_or(Vec::new(), |game| match game.turn() {
            Turn::Chance => vec![Edge::Draw],
            Turn::Terminal => Vec::new(),
            Turn::Choice(_) => game.choices(self.aggression_at(i)).into_iter().collect(),
        })
    }
    /// Returns the edge taken at a specific state index, if any.
    fn taken_at(&self, i: usize) -> Option<Edge> {
        self.history().get(i).copied()
    }
    /// Returns unified tree traversal data for visualization.
    ///
    /// Each tuple contains: (available_edges, taken_edge_index)
    /// - Choice turns: multiple betting edges
    /// - Chance turns: [Edge::Draw] (use revealed() to get cards)
    /// - Terminal: empty edges
    pub fn tree_nodes(&self) -> Vec<(Vec<Edge>, Option<usize>)> {
        self.states()
            .into_iter()
            .enumerate()
            .map(|(i, _)| {
                let edges = self.edges_at(i);
                let taken = self.taken_at(i).and_then(|te| edges.iter().position(|e| *e == te));
                (edges, taken)
            })
            .collect()
    }
}

/// Action modification methods.
impl Witness {
    /// Removes the most recent action and any trailing draws.
    pub fn undo(&self) -> Self {
        debug_assert!(self.can_undo());
        let mut copy = self.clone();
        copy.actions.pop();
        copy.recoil()
    }
    /// Adds an action, auto-inserting draw actions when needed.
    pub fn push(&self, action: Action) -> Self {
        self.try_push(action).expect("valid action")
    }
    /// Fallible version of [`push`](Self::push).
    ///
    /// Returns `Err` if the action is not legal in the current state,
    /// enabling graceful error handling instead of panicking.
    pub fn try_push(&self, action: Action) -> anyhow::Result<Self> {
        if !self.can_push(&action) {
            return Err(anyhow::anyhow!("illegal action {:?} at {:?}", action, self.head().turn()));
        }
        let mut copy = self.clone();
        copy.actions.push(action);
        Ok(copy.sprout())
    }
}

/// Validation.
impl Witness {
    /// Validates alignment and playability, returning error if invalid.
    pub fn validate(self) -> anyhow::Result<Self> {
        let recall = self.sprout();
        if !recall.aligned() {
            return Err(anyhow::anyhow!("recall is not aligned {self}"));
        }
        if !recall.can_play() {
            return Err(anyhow::anyhow!("recall is not playable {self}"));
        }
        Ok(recall)
    }

    /// Validates alignment for observation-only use (e.g. the opponent
    /// range), without requiring it to be the POV's turn. See `can_observe`.
    pub fn validate_observation(self) -> anyhow::Result<Self> {
        let recall = self.sprout();
        if !recall.aligned() {
            return Err(anyhow::anyhow!("recall is not aligned {self}"));
        }
        if !recall.can_observe() {
            return Err(anyhow::anyhow!("recall observation is not street-consistent {self}"));
        }
        Ok(recall)
    }
}

/// Auto-advancement to non-chance states.
impl Witness {
    /// Advances by inserting draw actions until at a decision point.
    fn sprout(&self) -> Self {
        let mut copy = self.clone();
        while copy.can_deal() {
            let street = copy.head().street().next();
            let reveal = copy.revealed(street).into();
            copy.actions.push(Action::Draw(reveal));
        }
        copy
    }

    /// Retreats by removing draw actions until at a decision point.
    fn recoil(&self) -> Self {
        let mut copy = self.clone();
        while copy.can_deal() {
            copy.actions.pop();
        }
        copy
    }
}

/// State predicates.
impl Witness {
    /// True if it's hero's turn and observation is current.
    pub fn can_play(&self) -> bool {
        self.head().turn() == self.turn() //               is it our turn right now?
            && self.head().street() == self.seen().street() //    have we exhausted info from Obs?
    }

    /// True if the observation has enough cards to back every choice node
    /// in the action history. Independent of whose turn it is.
    ///
    /// The opponent range is a backward-looking function of past decisions
    /// (see `opponent_observations`), so it is well-defined at any node
    /// in the line where the observation covers the streets those
    /// decisions happened on — not just hero-to-act nodes.
    ///
    /// Accepts two cases:
    /// - `head.street == seen.street` — standard decision or mid-street state.
    /// - `head.street == seen.street + 1` with `head.turn == Chance` — the
    ///   in-between state right after a closing call/check where betting
    ///   advanced the street but the next street's cards haven't been
    ///   dealt yet. All observed actions are still on `seen.street`, so
    ///   the abstraction lookups during replay are fully grounded.
    pub fn can_observe(&self) -> bool {
        let head = self.head();
        let seen = self.seen().street();
        head.street() == seen || (head.turn() == Turn::Chance && head.street() == seen.next())
    }

    /// True if the action is legal in the current state.
    pub fn can_push(&self, action: &Action) -> bool {
        self.head().is_allowed(action)
    }

    /// True if there are actions to undo.
    pub fn can_undo(&self) -> bool {
        !self.actions.is_empty()
    }

    /// True if a draw action should be auto-inserted.
    fn can_deal(&self) -> bool {
        self.can_know() && self.head().turn() == Turn::Chance
    }

    /// True if observation reveals more cards than current state.
    fn can_know(&self) -> bool {
        self.head().street() < self.seen().street()
    }
}

/// Display shows a compact visual representation of the game history
/// Format: table with cards from arrangement (preserving deal order)
/// and actions in a fixed-width grid layout
impl std::fmt::Display for Witness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const L: usize = 4;
        const R: usize = 44;
        const A: usize = 8;
        let hole = self
            .reveals
            .pocket()
            .iter()
            .map(|c| format!("{c}"))
            .collect::<Vec<_>>()
            .join(" ");
        let board = self
            .board()
            .iter()
            .map(|c| format!("{c}"))
            .collect::<Vec<_>>()
            .join(" ");
        let cards = if board.is_empty() { hole.clone() } else { format!("{hole} │ {board}") };
        writeln!(f, "┌{}┬{}┐", "─".repeat(L), "─".repeat(R))?;
        writeln!(f, "│ {:>2} │ {:<w$} │", self.turn().label(), cards, w = R - 2)?;
        writeln!(f, "├{}┼{}┤", "─".repeat(L), "─".repeat(R))?;
        Street::all()
            .iter()
            .filter_map(|street| {
                let actions = self.decisions(*street);
                actions.is_empty().not().then_some((street, actions))
            })
            .try_for_each(|(street, actions)| {
                let mut grid = String::new();
                for a in &actions {
                    use std::fmt::Write;
                    write!(grid, "{:<w$}", a.symbol(), w = A)?;
                }
                writeln!(f, "│ {:>2} │ {:<w$} │", street.symbol(), grid, w = R - 2)
            })?;
        write!(f, "└{}┴{}┘", "─".repeat(L), "─".repeat(R))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Not;

    /// initial recall: aligned, at preflop, empty (no decisions yet), reset is identity
    #[test]
    fn initial_invariants() {
        let r = Witness::initial(Turn::Choice(0));
        assert!(r.empty());
        assert!(r.aligned());
        assert_eq!(r.reset(), r);
        assert_eq!(r.seen().street(), Street::Pref);
        assert_eq!(r.head().street(), Street::Pref);
        assert_eq!(r.actions().len(), 0);
    }

    /// reset preserves pov and reveals, clears decisions back to just blinds
    /// reset is idempotent: reset(reset(x)) == reset(x)
    #[test]
    fn reset_idempotent() {
        let r = Witness::initial(Turn::Choice(0))
            .push(Action::Call(1))
            .push(Action::Raise(5))
            .push(Action::Raise(20))
            .push(Action::Call(15));
        assert_eq!(r.reset(), r.reset().reset());
    }

    /// push then undo returns to original path length
    #[test]
    fn push_undo_inverse() {
        let r = Witness::initial(Turn::Choice(0));
        let a = r.head().legal().first().copied().expect("legal");
        assert_eq!(r.push(a).undo().subgame().length(), r.subgame().length());
    }

    /// base() returns Game::default with hero's hole cards; no blinds posted yet
    /// root() returns game state after blinds are posted
    /// head() returns current state after applying all actions to root
    #[test]
    fn base_vs_root_vs_head() {
        let r = Witness::initial(Turn::Choice(0));
        let base = r.base();
        let root = r.root();
        let head = r.head();
        assert_eq!(base.street(), Street::Pref);
        assert_eq!(root.street(), Street::Pref);
        assert_eq!(head.street(), Street::Pref);
        assert_eq!(base.pot(), 0); // no blinds yet
        assert_eq!(root.pot(), Game::sblind() + Game::bblind()); // blinds posted
        assert_eq!(head.pot(), Game::sblind() + Game::bblind()); // same as root when empty
    }

    /// states reconstructs game states: [root, after_action_0, after_action_1, ..., head]
    /// states length = actions length + 1 (root state plus one state per action)
    #[test]
    fn states_reconstruction() {
        let r = Witness::initial(Turn::Choice(0)).push(Action::Call(1));
        let states = r.states();
        assert_eq!(states.len(), r.actions().len() + 1);
        assert_eq!(states.first(), Some(&r.root()));
        assert_eq!(states.last(), Some(&r.head()));
        states
            .windows(2)
            .zip(r.actions().iter())
            .for_each(|(pair, &act)| assert_eq!(pair[1], pair[0].apply(act)));
    }

    /// subgame returns current street edges only
    #[test]
    fn subgame_current_street() {
        let r = Witness::initial(Turn::Choice(0));
        assert_eq!(r.subgame().length(), 0);
        let r = r.push(Action::Call(1));
        assert_eq!(r.subgame().length(), 1);
    }

    /// aligned: observation street matches draws in actions
    /// From tuple uses push() which sprouts, so both approaches align
    #[test]
    fn alignment_check() {
        let obs = Observation::from(Street::Flop);
        let act = vec![
            Action::Call(1), //
            Action::Check,
        ];
        assert!(Witness::from((Turn::Choice(0), obs, act)).aligned());
        assert!(
            Witness::from((Turn::Choice(0), Arrangement::from(Street::Flop)))
                .push(Action::Call(1))
                .push(Action::Check)
                .aligned()
        );
    }

    /// behindness: seen().street() > head().street() means recall is behind
    /// this is valid when user sets observation before adding all actions
    #[test]
    fn behindness_observation_ahead() {
        let behind = Witness {
            pov: Turn::Choice(0),
            stacks: [STACK; N],
            dealer: 0,
            actions: Vec::new(),
            reveals: Arrangement::from(Street::Turn),
        };
        assert!(behind.seen().street() > behind.head().street()); // behind
        assert!(behind.aligned().not()); // not aligned until actions catch up
    }

    /// board length: pref=0, flop=3, turn=4, river=5
    #[test]
    fn board_by_street() {
        let r = Witness::from((Turn::Choice(0), Arrangement::from(Street::Rive)));
        assert_eq!(r.board().len(), 0);
        let r = r.push(Action::Call(1)).push(Action::Check);
        assert_eq!(r.board().len(), 3);
        let r = r.push(Action::Check).push(Action::Check);
        assert_eq!(r.board().len(), 4);
        let r = r.push(Action::Check).push(Action::Check);
        assert_eq!(r.board().len(), 5);
    }

    /// truncate cuts actions to specified street, then sprout advances if obs allows
    /// to test pure truncation, use observation matching target street
    #[test]
    fn truncate_to_street() {
        let r = Witness::from((Turn::Choice(0), Arrangement::from(Street::Flop)))
            .push(Action::Call(1)) // P0 pref
            .push(Action::Check) // P1 pref -> flop
            .push(Action::Check) // P1 flop
            .push(Action::Check); // P0 flop (no turn, obs is flop)
        let t = r.truncate(Street::Pref);
        // sprout advances to flop since obs has flop cards
        assert!(r.head().street() == Street::Flop);
        assert!(t.head().street() == Street::Flop);
        assert!(t.actions().len() < r.actions().len());
    }

    /// decisions(street) returns non-blind, non-draw actions for that street
    #[test]
    fn decisions_per_street() {
        let r = Witness::from((Turn::Choice(0), Arrangement::from(Street::Flop)))
            .push(Action::Call(1))
            .push(Action::Check)
            .push(Action::Check)
            .push(Action::Check);
        assert_eq!(r.decisions(Street::Pref).len(), 2);
        assert_eq!(r.decisions(Street::Flop).len(), 2);
        assert!(r.decisions(Street::Pref).iter().all(Action::is_choice));
        assert!(r.decisions(Street::Flop).iter().all(Action::is_choice));
    }

    /// walk through all streets: P0 first preflop, P1 first postflop
    #[test]
    fn playability_all_streets() {
        let r = Witness::from((Turn::Choice(0), Arrangement::from(Street::Rive)));
        assert_eq!(r.head().turn(), Turn::Choice(0));
        assert_eq!(r.head().street(), Street::Pref);
        let r = r.push(Action::Call(1)).push(Action::Check);
        assert_eq!(r.head().street(), Street::Flop);
        assert_eq!(r.head().turn(), Turn::Choice(1));
        let r = r.push(Action::Check).push(Action::Check);
        assert_eq!(r.head().street(), Street::Turn);
        assert_eq!(r.head().turn(), Turn::Choice(1));
        let r = r.push(Action::Check).push(Action::Check);
        assert_eq!(r.head().street(), Street::Rive);
        assert_eq!(r.head().turn(), Turn::Choice(1));
        assert!(r.aligned());
    }

    /// when not hero's turn, head().turn() != pov
    #[test]
    fn playability_not_our_turn() {
        let r = Witness::from((Turn::Choice(0), Arrangement::from(Street::Pref))).push(Action::Call(1));
        assert_eq!(r.head().turn(), Turn::Choice(1));
    }

    /// from Arrangement starts with empty actions (blinds in root)
    #[test]
    fn from_arrangement_empty_actions() {
        let r = Witness::from((Turn::Choice(0), Arrangement::from(Street::Pref)));
        assert_eq!(r.actions().len(), 0);
        // but root() has blinds posted
        assert_eq!(r.root().pot(), Game::sblind() + Game::bblind());
    }

    /// from tuple stores only provided actions (no blinds)
    #[test]
    fn from_tuple_stores_actions() {
        let obs = Observation::from(Street::Pref);
        let act = vec![
            Action::Call(1), //
        ];
        let r = Witness::from((Turn::Choice(0), obs, act.clone()));
        assert_eq!(r.actions().len(), act.len());
        // all_actions() includes blinds for display
        assert_eq!(r.complete().len(), Game::blinds().len() + act.len());
    }

    /// replace swaps arrangement, updates draw actions
    #[test]
    fn replace_swaps_arrangement() {
        let obs = Observation::from(Street::Flop);
        let act = vec![
            Action::Call(1), //
            Action::Check,
        ];
        let old = Witness::from((Turn::Choice(0), obs, act));
        let new = old.replace(Arrangement::from(Street::Flop));
        assert_ne!(new.seen(), old.seen());
        assert_eq!(new.turn(), old.turn());
    }

    /// revealed(street) returns cards for that street
    #[test]
    fn revealed_per_street() {
        let r = Witness::from((Turn::Choice(0), Arrangement::from(Street::Turn)));
        assert_eq!(r.revealed(Street::Flop).len(), 3);
        assert_eq!(r.revealed(Street::Turn).len(), 1);
        assert_eq!(r.revealed(Street::Rive).len(), 0);
    }

    /// empty: no decisions beyond blinds
    #[test]
    fn empty_means_no_decisions() {
        assert!(Witness::initial(Turn::Choice(0)).empty());
        assert!(Witness::initial(Turn::Choice(0)).push(Action::Call(1)).empty().not());
    }

    /// aggression counts trailing aggressive edges
    #[test]
    fn aggression_counts_trailing() {
        let obs = Observation::from(Street::Pref);
        let act = vec![
            Action::Raise(4), //
            Action::Raise(8),
        ];
        let r = Witness::from((Turn::Choice(0), obs, act));
        assert_eq!(
            r.aggression(),
            r.subgame()
                .into_iter()
                .rev()
                .take_while(Edge::is_choice)
                .filter(Edge::is_aggro)
                .count()
        );
    }

    /// choices returns nonempty abstracted edges
    #[test]
    fn choices_nonempty() {
        assert!(
            Witness::from((Turn::Choice(0), Arrangement::from(Street::Pref)))
                .choices()
                .length()
                > 0
        );
    }

    /// can_play: hero's turn and at observation street
    #[test]
    fn can_play_conditions() {
        let r = Witness::from((Turn::Choice(0), Arrangement::from(Street::Pref)));
        assert_eq!(r.can_play(), r.turn() == Turn::Choice(0)); // can_play iff pov matches head's turn
        let s = r.push(Action::Call(1));
        assert_eq!(s.can_play(), s.turn() == Turn::Choice(1)); // after P0 acts, it's P1's turn
    }

    /// can_undo: false at initial, true after push
    #[test]
    fn can_undo_conditions() {
        let r = Witness::initial(Turn::Choice(0));
        assert!(r.can_undo().not());
        assert!(r.push(Action::Call(1)).can_undo());
    }

    /// can_push: legal actions pass, illegal fail
    #[test]
    fn can_push_conditions() {
        let r = Witness::initial(Turn::Choice(0));
        assert!(r.can_push(&Action::Call(1)));
        assert!(r.can_push(&Action::Check).not());
    }
    /// try_arrange preserves the exact arrangement through street advances
    #[test]
    fn try_arrange_preserves_arrangement() {
        let arr = Arrangement::from(Street::Rive);
        let recall = Witness::try_arrange(
            Turn::Choice(0),
            arr,
            vec![Action::Call(1), Action::Check, Action::Check, Action::Check],
        )
        .unwrap();
        assert_eq!(recall.arr(), arr);
        assert_eq!(recall.revealed(Street::Flop), arr.revealed(Street::Flop));
        assert_eq!(recall.revealed(Street::Turn), arr.revealed(Street::Turn));
    }
    /// try_build and try_arrange agree when given the same observation
    #[test]
    fn try_build_matches_try_arrange_from_obs() {
        let obs = Observation::from(Street::Flop);
        let actions = vec![Action::Call(1), Action::Check];
        let from_build = Witness::try_build(Turn::Choice(0), obs, actions.clone()).unwrap();
        let from_arrange = Witness::try_arrange(Turn::Choice(0), Arrangement::from(obs), actions).unwrap();
        assert_eq!(from_build, from_arrange);
    }
    /// Draw actions inserted by sprout match the arrangement's revealed cards
    #[test]
    fn sprout_draws_match_arrangement() {
        let arr = Arrangement::from(Street::Rive);
        let recall = Witness::try_arrange(Turn::Choice(0), arr, vec![Action::Call(1), Action::Check]).unwrap();
        let draws: Vec<Hand> = recall.actions().iter().filter_map(Action::hand).collect();
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0], Hand::from(arr.revealed(Street::Flop)));
    }
}

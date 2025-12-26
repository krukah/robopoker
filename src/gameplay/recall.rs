use super::*;
use crate::cards::*;
use crate::mccfr::*;
use crate::*;
use std::ops::Not;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// a complete representation of perfect recall [Game] history
/// from the perspective of the hero [Turn].
///
/// note that this struct implicitly assumes:
/// - default stacks
/// - default dealer position
/// - blinds, draws, and player actions are all included in action path
pub struct Recall {
    pov: Turn,
    actions: Vec<Action>,
    reveals: Arrangement,
}

impl Arbitrary for Recall {
    fn random() -> Self {
        Self::initial()
    }
}

impl Recall {
    pub fn initial() -> Self {
        let pov = Turn::random();
        let reveals = Arrangement::from(Street::Pref);
        let actions = Game::blinds().to_vec();
        Self {
            pov,
            actions,
            reveals,
        }
    }
}

impl Recall {
    pub fn bind(&self, abs: Abstraction) -> Info {
        crate::mccfr::Info::from_path(self, abs)
    }
    pub fn futures(&self) -> Path {
        crate::mccfr::Info::futures(&self.head(), self.depth())
    }
    pub fn depth(&self) -> usize {
        crate::mccfr::Info::depth(&self.path())
    }
}

/// random player, blinds included
impl From<Arrangement> for Recall {
    fn from(cards: Arrangement) -> Self {
        let pov = Turn::random();
        let actions = Game::blinds().to_vec();
        let reveals = cards;
        Self {
            pov,
            actions,
            reveals,
        }
    }
}

/// random non-folding actions lead to this street
impl From<Street> for Recall {
    fn from(_: Street) -> Self {
        todo!()
    }
}

impl From<(Turn, Observation, Vec<Action>)> for Recall {
    fn from((pov, seen, actions): (Turn, Observation, Vec<Action>)) -> Self {
        let reveals = Arrangement::from(seen);
        let actions = Game::blinds().into_iter().chain(actions).collect();
        Self {
            pov,
            actions,
            reveals,
        }
    }
}

impl Recall {
    /// Returns the base game state with the hero's hole cards set from the observation.
    pub fn base(&self) -> Game {
        // @const-stacks
        // @const-dealer
        Game::default().wipe(Hole::from(self.seen()))
    }

    /// Returns the current game state after applying all actions in the recall history.
    /// Notably, independent of what cards we see in self.reveals.
    pub fn head(&self) -> Game {
        self.actions()
            .iter()
            .cloned()
            .fold(self.base(), |g, a| g.apply(a))
    }

    /// Returns the turn perspective of this recall.
    pub fn turn(&self) -> Turn {
        self.pov
    }

    /// Returns the observation (cards seen) for this recall.
    pub fn seen(&self) -> Observation {
        self.reveals.observation()
    }

    /// Resets the recall to the initial state (just blinds).
    pub fn reset(&self) -> Self {
        Self {
            pov: self.turn(),
            reveals: self.reveals.clone(),
            actions: Game::blinds().to_vec(),
        }
    }

    /// Returns the current cursor position as a node index based on the action history length.
    pub fn cursor(&self) -> petgraph::graph::NodeIndex {
        petgraph::graph::NodeIndex::new(self.actions().len() - 1)
    }

    /// Returns a clone of the action history.
    pub fn actions(&self) -> &Vec<Action> {
        &self.actions
    }

    /// Returns the complete game history as a vector of game states.
    pub fn history(&self) -> Vec<Game> {
        let base = self.base();
        let acts = self
            .actions()
            .iter()
            .scan(base, |g, a| Some(g.consume(*a)))
            .collect::<Vec<Game>>();
        std::iter::once(base).chain(acts).collect()
    }

    /// Truncates the recall to the specified street, preserving the order of actions.
    pub fn truncate(&self, street: Street) -> Self {
        let pov = self.turn();
        let reveals = self.reveals.clone();
        let actions = self
            .history() // inconsisnte
            .into_iter()
            .skip(1)
            .zip(self.actions().iter().cloned())
            .map(|(game, action)| (action, game))
            .collect::<Vec<(Action, Game)>>()
            .into_iter()
            .take_while(|(_, game)| game.street() <= street)
            .map(|(action, _)| action)
            .collect::<Vec<Action>>();
        let recall = Self {
            pov,
            reveals,
            actions,
        };
        recall.sprout()
    }

    /// Returns the same recall but with the Arrangement swapped
    pub fn replace(&self, reveals: Arrangement) -> Self {
        let mut actions = self.actions().clone();
        actions
            .iter_mut()
            .filter(|a| a.is_chance())
            .zip(reveals.draws())
            .for_each(|(old, new)| *old = new);
        Self {
            pov: self.turn(),
            actions,
            reveals,
        }
    }

    /// Returns all decision actions (non-blind, non-draw) for a specific street.
    pub fn decisions(&self, street: Street) -> Vec<Action> {
        let mut actions = Vec::new();
        let mut current = Street::Pref;
        for action in self.actions().iter().cloned() {
            if action.is_blind() {
                continue;
            } else if action.is_chance() {
                current = current.next();
            } else if current == street {
                actions.push(action);
            } else {
                continue;
            }
        }
        actions
    }

    /// Returns the cards of the board in the order they were dealt.
    /// Notably, this depends both on Arrangement cards and Action decisions.
    /// It uses Action decisions to determine what street to truncate on,
    /// and it uses Arrangement cards to determine what cards are on the board.
    pub fn board(&self) -> Vec<Card> {
        let street = self.head().street();
        Street::all()
            .iter()
            .skip(1)
            .filter(|s| **s <= street)
            .cloned()
            .flat_map(|s| self.revealed(s))
            .collect()
    }

    /// Returns the cards revealed for a specific street.
    pub fn revealed(&self, street: Street) -> Vec<Card> {
        self.reveals.revealed(street)
    }

    /// Returns the path representation WITHOUT blinds.
    pub fn path(&self) -> Path {
        // @perfect-recall
        let top = self.base();
        self.actions()
            .into_iter()
            .cloned()
            .scan(top, |g, a| Some(std::mem::replace(g, g.apply(a)).edgify(a)))
            .skip(2)
            .collect::<Path>()
    }

    /// Returns the isomorphism based on the observation.
    pub fn isomorphism(&self) -> Isomorphism {
        Isomorphism::from(self.seen())
    }

    /// Checks if there is anything here
    pub fn empty(&self) -> bool {
        self.actions().len() <= Game::blinds().len()
    }

    /// Checks if the recall is consistent by checking if the observed public cards match the dealt cards.
    /// More specifically: does the extent of publicly dealt cards in Arrangement match the
    /// publicly dealt cards in the set of Actions?
    pub fn aligned(&self) -> bool {
        self.seen().public().clone()
            == self
                .actions()
                .iter()
                .filter(|a| a.is_chance())
                .filter_map(|a| a.hand())
                .fold(Hand::empty(), Hand::add)
    }
}

impl Recall {
    /// Undoes the most recent action from the recall sequence.
    pub fn undo(&self) -> Self {
        assert!(self.can_undo());
        let mut copy = self.clone();
        copy.actions.pop();
        copy.recoil()
    }

    /// Pushes a new action to the recall sequence and automatically handles card reveals.
    /// Adds Draw actions to reveal cards when transitioning between streets.
    pub fn push(&self, action: Action) -> Self {
        assert!(self.can_push(&action));
        let mut copy = self.clone();
        copy.actions.push(action);
        copy.sprout()
    }
}

impl Recall {
    pub fn validate(self) -> anyhow::Result<Self> {
        let recall = self.sprout();
        if !recall.aligned() {
            return Err(anyhow::anyhow!("recall is not aligned {}", self));
        }
        if !recall.can_play() {
            return Err(anyhow::anyhow!("recall is not playable {}", self));
        }
        Ok(recall)
    }
}

impl Recall {
    /// Keep pushing cards until we're in a valid non-chance state.
    /// Reflex forward
    fn sprout(&self) -> Self {
        let mut copy = self.clone();
        while copy.can_deal() {
            let street = copy.head().street().next();
            let reveal = copy.revealed(street).into();
            copy.actions.push(Action::Draw(reveal));
        }
        copy
    }

    /// Keep pooping cards until we're in a valid non-chance state.
    /// Reflex backward
    fn recoil(&self) -> Self {
        let mut copy = self.clone();
        while copy.can_deal() {
            copy.actions.pop();
        }
        copy
    }
}

impl Recall {
    /// Checks whether the current game state allows for strategy lookup.
    /// Requires that it's the player's turn and game state is synchronized with observation.
    pub fn can_play(&self) -> bool {
        self.head().turn() == self.turn() //               is it our turn right now?
            && self.head().street() == self.seen().street() //    have we exhausted info from Obs?
    }

    /// Checks whether the given action can be legally pushed to the current recall sequence.
    pub fn can_push(&self, action: &Action) -> bool {
        self.head().is_allowed(action)
    }

    /// Checks whether the recall sequence can be rewound (undone).
    /// Returns true if there are any actions beyond the initial blinds.
    pub fn can_undo(&self) -> bool {
        self.actions.len() > Game::blinds().len()
    }

    /// Checks whether cards should be automatically revealed for the next street.
    /// Returns true if it's the dealer's turn and there are more streets to reveal.
    fn can_deal(&self) -> bool {
        self.can_know() && self.head().turn() == Turn::Chance
    }

    /// Checks whether the game can progress to the next street.
    /// Returns true if current street is behind the observation's street.
    fn can_know(&self) -> bool {
        self.head().street() < self.seen().street()
    }
}

/// Display shows a compact visual representation of the game history
/// Format: table with cards from arrangement (preserving deal order)
/// and actions in a fixed-width grid layout
impl std::fmt::Display for Recall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const L: usize = 4;
        const R: usize = 44;
        const A: usize = 8;
        let hole = self
            .reveals
            .pocket()
            .iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<_>>()
            .join(" ");
        let board = self
            .board()
            .iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<_>>()
            .join(" ");
        let cards = if board.is_empty() {
            format!("{}", hole)
        } else {
            format!("{} │ {}", hole, board)
        };
        writeln!(f, "┌{}┬{}┐", "─".repeat(L), "─".repeat(R))?;
        writeln!(
            f,
            "│ {:>2} │ {:<w$} │",
            self.turn().label(),
            cards,
            w = R - 2
        )?;
        writeln!(f, "├{}┼{}┤", "─".repeat(L), "─".repeat(R))?;
        Street::all()
            .iter()
            .filter_map(|street| {
                let actions = self.decisions(*street);
                actions.is_empty().not().then_some((street, actions))
            })
            .try_for_each(|(street, actions)| {
                let grid = actions
                    .iter()
                    .map(|a| format!("{:<w$}", a.symbol(), w = A))
                    .collect::<String>();
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
        let r = Recall::initial();
        assert!(r.empty());
        assert!(r.aligned());
        assert_eq!(r.reset(), r);
        assert_eq!(r.seen().street(), Street::Pref);
        assert_eq!(r.head().street(), Street::Pref);
        assert_eq!(r.actions().len(), Game::blinds().len());
    }

    /// reset preserves pov and reveals, clears decisions back to just blinds
    /// reset is idempotent: reset(reset(x)) == reset(x)
    #[test]
    fn reset_idempotent() {
        let r = Recall::initial()
            .push(Action::Call(1))
            .push(Action::Raise(5))
            .push(Action::Raise(20))
            .push(Action::Call(15));
        assert_eq!(r.reset(), r.reset().reset());
    }

    /// push then undo returns to original path length
    #[test]
    fn push_undo_inverse() {
        let r = Recall::initial();
        let a = r.head().legal().first().cloned().expect("legal");
        assert_eq!(r.push(a).undo().path().length(), r.path().length());
    }

    /// base() returns Game::default with hero's hole cards; no blinds posted yet
    /// head() returns game state after applying all actions (including blinds)
    /// the distinction: base is pre-blind, head is current state after all actions
    #[test]
    fn base_vs_head() {
        let r = Recall::initial();
        let base = r.base();
        let head = r.head();
        assert_eq!(base.street(), Street::Pref);
        assert_eq!(head.street(), Street::Pref);
        assert_eq!(base.pot(), 0); // no blinds yet
        assert_eq!(head.pot(), Game::sblind() + Game::bblind()); // blinds posted
    }

    /// history reconstructs game states: [base, after_action_0, after_action_1, ..., head]
    /// history length = actions length + 1 (base state plus one state per action)
    #[test]
    fn history_reconstruction() {
        let r = Recall::initial();
        let hist = r.history();
        assert_eq!(hist.len(), r.actions().len() + 1);
        assert_eq!(hist.first(), Some(&r.base()));
        assert_eq!(hist.last(), Some(&r.head()));
        hist.windows(2)
            .zip(r.actions().iter())
            .for_each(|(pair, &act)| assert_eq!(pair[1], pair[0].apply(act)));
    }

    /// path excludes blinds; contains only post-blind edges (draws + decisions)
    /// path length = actions.filter(not blind).count
    #[test]
    fn path_excludes_blinds() {
        let r = Recall::initial();
        let path = r.path();
        let want = r.actions().iter().filter(|a| !a.is_blind()).count();
        assert_eq!(path.length(), want);
    }

    /// aligned: observation street matches draws in actions
    /// From tuple doesn't sprout; build via push to align
    #[test]
    fn alignment_check() {
        let obs = Observation::from(Street::Flop);
        let act = vec![Action::Call(1), Action::Check];
        assert!(Recall::from((Turn::Choice(0), obs, act)).aligned().not());
        assert!(
            Recall::from(Arrangement::from(Street::Flop))
                .push(Action::Call(1))
                .push(Action::Check)
                .aligned()
        );
    }

    /// behindness: seen().street() > head().street() means recall is behind
    /// this is valid when user sets observation before adding all actions
    #[test]
    fn behindness_observation_ahead() {
        let behind = Recall {
            pov: Turn::Choice(0),
            actions: Game::blinds().to_vec(),
            reveals: Arrangement::from(Street::Turn),
        };
        assert!(behind.seen().street() > behind.head().street()); // behind
        assert!(behind.aligned().not()); // not aligned until actions catch up
    }

    /// board length: pref=0, flop=3, turn=4, river=5
    #[test]
    fn board_by_street() {
        let r = Recall::from(Arrangement::from(Street::Rive));
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
        let r = Recall::from(Arrangement::from(Street::Flop))
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
        let r = Recall::from(Arrangement::from(Street::Flop))
            .push(Action::Call(1))
            .push(Action::Check)
            .push(Action::Check)
            .push(Action::Check);
        assert_eq!(r.decisions(Street::Pref).len(), 2);
        assert_eq!(r.decisions(Street::Flop).len(), 2);
        assert!(r.decisions(Street::Pref).iter().all(|a| a.is_choice()));
        assert!(r.decisions(Street::Flop).iter().all(|a| a.is_choice()));
    }

    /// walk through all streets: P0 first preflop, P1 first postflop
    #[test]
    fn playability_all_streets() {
        let r = Recall::from(Arrangement::from(Street::Rive));
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
        let r = Recall::from(Arrangement::from(Street::Pref)).push(Action::Call(1));
        assert_eq!(r.head().turn(), Turn::Choice(1));
    }

    /// from Arrangement auto-appends blinds
    #[test]
    fn from_arrangement_appends_blinds() {
        let r = Recall::from(Arrangement::from(Street::Pref));
        assert_eq!(r.actions().len(), 2);
        assert_eq!(r.actions()[0], Action::Blind(Game::sblind()));
        assert_eq!(r.actions()[1], Action::Blind(Game::bblind()));
    }

    /// from tuple prepends blinds to provided actions
    #[test]
    fn from_tuple_prepends_blinds() {
        let obs = Observation::from(Street::Pref);
        let act = vec![Action::Call(1)];
        let r = Recall::from((Turn::Choice(0), obs, act.clone()));
        assert_eq!(r.actions().len(), Game::blinds().len() + act.len());
        assert!(r.actions().starts_with(&Game::blinds()));
    }

    /// replace swaps arrangement, updates draw actions
    #[test]
    fn replace_swaps_arrangement() {
        let obs = Observation::from(Street::Flop);
        let act = vec![Action::Call(1), Action::Check];
        let old = Recall::from((Turn::Choice(0), obs, act));
        let new = old.replace(Arrangement::from(Street::Flop));
        assert_ne!(new.seen(), old.seen());
        assert_eq!(new.turn(), old.turn());
    }

    /// revealed(street) returns cards for that street
    #[test]
    fn revealed_per_street() {
        let r = Recall::from(Arrangement::from(Street::Turn));
        assert_eq!(r.revealed(Street::Flop).len(), 3);
        assert_eq!(r.revealed(Street::Turn).len(), 1);
        assert_eq!(r.revealed(Street::Rive).len(), 0);
    }

    /// empty: no decisions beyond blinds
    #[test]
    fn empty_means_no_decisions() {
        assert!(Recall::initial().empty());
        assert!(Recall::initial().push(Action::Call(1)).empty().not());
    }

    /// depth counts trailing aggressive edges
    #[test]
    fn depth_counts_aggression() {
        let obs = Observation::from(Street::Pref);
        let act = vec![Action::Raise(4), Action::Raise(8)];
        let r = Recall::from((Turn::Choice(0), obs, act));
        assert_eq!(
            r.depth(),
            r.path()
                .into_iter()
                .rev()
                .take_while(|e| e.is_choice())
                .filter(|e| e.is_aggro())
                .count()
        );
    }

    /// futures returns nonempty abstracted edges
    #[test]
    fn futures_nonempty() {
        assert!(
            Recall::from(Arrangement::from(Street::Pref))
                .futures()
                .length()
                > 0
        );
    }

    /// can_play: hero's turn and at observation street
    #[test]
    fn can_play_conditions() {
        let r = Recall::from(Arrangement::from(Street::Pref));
        assert_eq!(r.can_play(), r.turn() == Turn::Choice(0)); // can_play iff pov matches head's turn
        let s = r.push(Action::Call(1));
        assert_eq!(s.can_play(), s.turn() == Turn::Choice(1)); // after P0 acts, it's P1's turn
    }

    /// can_undo: false at initial, true after push
    #[test]
    fn can_undo_conditions() {
        let r = Recall::initial();
        assert!(r.can_undo().not());
        assert!(r.push(Action::Call(1)).can_undo());
    }

    /// can_push: legal actions pass, illegal fail
    #[test]
    fn can_push_conditions() {
        let r = Recall::initial();
        assert!(r.can_push(&Action::Call(1)));
        assert!(r.can_push(&Action::Check).not());
    }
}

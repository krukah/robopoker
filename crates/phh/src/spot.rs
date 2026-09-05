//! Turning a reducible hand into decisions our blueprint can be asked about.
//!
//! One [`Spot`] is a single postflop decision: the hero's view of everything
//! that had happened when it was their turn, plus what they actually did. Ask a
//! strategy what it would do at [`Spot::witness`] and compare against
//! [`Spot::played`] — that comparison is Tier 2 ([`crate::Agreement`]). Play the
//! hand out from [`Spot::game`] behind each of the two actions and difference
//! the results — that is Tier 3 ([`crate::Ledger`]).
//!
//! Three things have to line up for the question to be fair:
//!
//! - **The preflop is substituted**, not translated — see [`crate::project`].
//! - **The hero sees only what they had seen.** The arrangement is truncated to
//!   the decision's street, so a flop question cannot leak the turn card.
//! - **Chip amounts divide** by the same scale [`crate::walk`] uses, and all-in
//!   is read off the log rather than off the divide.
//!
//! What a `Spot` carries beyond the hero's view — both players' hole cards, the
//! board the dealer actually ran out — is never shown to a strategy. It is there
//! so a rollout can settle a showdown, and so [`crate::worlds`] only has to
//! sample the streets the log never reached.

use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// One postflop decision, from the deciding player's point of view.
#[derive(Debug, Clone)]
pub struct Spot {
    witness: Witness,
    game: Game,
    actions: Vec<Action>,
    holes: [Hole; 2],
    board: Vec<Card>,
    stacks: [Chips; N],
    played: Action,
    seat: usize,
    street: Street,
}

impl Spot {
    /// Everything the hero had seen when it was their turn.
    pub fn witness(&self) -> &Witness {
        &self.witness
    }

    /// The engine state at the decision, with **both** players' cards seated.
    ///
    /// Unlike [`witness`](Self::witness) this is not a point of view — it holds
    /// what the hero could not see, so a rollout from here can reach a showdown.
    pub fn game(&self) -> Game {
        self.game
    }

    /// Every action taken to reach the decision, deals included.
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Both hole cards, indexed by engine position: `0` in position, `1` out.
    pub fn holes(&self) -> &[Hole; 2] {
        &self.holes
    }

    /// The board the dealer actually ran out, however far it got before the hand
    /// ended. Three to five cards on a hand that saw a flop.
    pub fn board(&self) -> &[Card] {
        &self.board
    }

    /// Starting stacks, in engine chips.
    pub fn stacks(&self) -> [Chips; N] {
        self.stacks
    }

    /// What the hero actually did, in engine chips.
    pub fn played(&self) -> Action {
        self.played
    }

    /// The log's seat index for the hero.
    pub fn seat(&self) -> usize {
        self.seat
    }

    pub fn street(&self) -> Street {
        self.street
    }

    /// Pot before the decision, in engine chips.
    pub fn pot(&self) -> Chips {
        self.game.pot()
    }

    /// The abstract edge the hero's action maps to.
    ///
    /// This is the quantity a blueprint distribution is comparable against: the
    /// blueprint speaks in edges and the log speaks in chips. Deterministic
    /// nearest-anchor snapping, so the same played action always scores against
    /// the same edge — randomized translation would add noise to the comparison
    /// without making it fairer.
    pub fn edge(&self) -> Edge {
        self.witness.head().edgify(self.played, self.witness.aggression())
    }
}

/// Every postflop decision in a hand that reduces to our tree.
///
/// Returns `None` when the hand does not reduce, when the substituted preflop
/// cannot be built, or when the log's stacks are not the depth our tree was
/// trained at. Decisions by both players are returned; filter on [`Spot::seat`]
/// for one player's.
pub fn spots(record: &Record, outcome: &Outcome) -> Option<Vec<Spot>> {
    let reduction = reduce(record, outcome)?;
    let bblind = record.blinds().iter().copied().max().unwrap_or(0);
    let projection = project(&reduction, bblind)?;
    let scale = scale(record).ok()?;
    let holes = record.holes();
    let board = record.board();
    let seats = reduction.seats();
    let stacks = stacks(record, &seats, scale)?;
    let seated = [
        holes.get(seats.ip()).copied().flatten()?,
        holes.get(seats.oop()).copied().flatten()?,
    ];
    let mut spots = Vec::new();
    let mut actions = projection.actions().to_vec();
    // `Game::root` deals random hole cards. Seat the real ones before any board
    // card is dealt, or a `Draw` will collide with a phantom and be rejected.
    let mut game = actions
        .iter()
        .fold(Game::root().deal(0, seated[0]).deal(1, seated[1]), |g, a| g.apply(*a));
    for node in outcome.nodes().iter().filter(|n| n.street() != Street::Pref) {
        // Deal whatever the log has dealt but the engine has not.
        while game.turn() == Turn::Chance {
            let street = game.street().next();
            let draw = Action::Draw(Hand::from(
                board
                    .get(street.n_board() - street.n_revealed()..street.n_board())?
                    .to_vec(),
            ));
            game = game.try_apply(draw).ok()?;
            actions.push(draw);
        }
        let hero = match node.seat() {
            seat if seat == seats.ip() => Turn::Choice(0),
            _ => Turn::Choice(1),
        };
        if game.turn() != hero {
            return None;
        }
        let played = game.snap(intent(&game, node, scale));
        spots.push(Spot {
            witness: Witness::try_arrange_with(
                hero,
                arrangement(&holes, &board, node.seat(), node.street()),
                stacks,
                actions.iter().copied().filter(Action::is_choice).collect(),
            )
            .ok()?,
            game,
            actions: actions.clone(),
            holes: seated,
            board: board.clone(),
            stacks,
            played,
            seat: node.seat(),
            street: node.street(),
        });
        game = game.try_apply(played).ok()?;
        actions.push(played);
    }
    Some(spots)
}

/// Every scoreable decision across a corpus.
///
/// `player` selects one player's decisions by name and `others` inverts that
/// selection into the control: if a strategy scores the rest of the table as
/// well as it scores Pluribus, the score is measuring generic poker rather than
/// Pluribus in particular. Hands that fail to reduce are skipped silently —
/// [`crate::shape`] is the place to count them.
pub fn harvest(records: &[Record], player: Option<&str>, others: bool) -> Vec<Spot> {
    records
        .iter()
        .filter_map(|record| Some((record, replay(record).ok()?)))
        .filter_map(|(record, outcome)| Some((record, spots(record, &outcome)?)))
        .flat_map(|(record, spots)| {
            let seat = player.and_then(|name| record.seat_of(name));
            spots
                .into_iter()
                .filter(move |spot| player.is_none() || (seat == Some(spot.seat())) != others)
        })
        .collect()
}

/// Starting stacks in engine chips, and `None` if they are not our tree's depth.
///
/// A blueprint is trained at one stack depth and its bet-sizing grid is anchored
/// to it, so a hand played deeper or shallower is not a spot our strategy has an
/// opinion about — the honest move is to drop it rather than rescale a
/// [`Witness`] and pretend. Every Pluribus hand is 100bb, which is exactly
/// [`pokerkit::STACK`], so nothing in that corpus is dropped here.
fn stacks(record: &Record, seats: &Seats, scale: Amount) -> Option<[Chips; N]> {
    let depth = |seat: usize| Chips::try_from(*record.starting_stacks().get(seat)? / scale).ok();
    [depth(seats.ip())?, depth(seats.oop())?]
        .into_iter()
        .all(|stack| stack == STACK)
        .then_some([STACK; N])
}

/// Hole cards plus the board **as of** `street` — never further.
fn arrangement(holes: &[Option<Hole>], board: &[Card], seat: usize, street: Street) -> Arrangement {
    Arrangement::from(
        holes
            .get(seat)
            .copied()
            .flatten()
            .map(|hole| Vec::<Card>::from(Hand::from(hole)))
            .unwrap_or_default()
            .into_iter()
            .chain(board.iter().copied().take(street.n_board()))
            .collect::<Vec<Card>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    fn hand(actions: &str) -> (Record, Outcome) {
        let text = format!(
            "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
             blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
             starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
             actions = [{DEAL}, {actions}]\n"
        );
        let record = crate::parse(&text, "test").unwrap();
        let outcome = crate::replay(&record).unwrap();
        (record, outcome)
    }

    /// Button opens, big blind calls, both check the flop, big blind bets turn.
    fn worked() -> Vec<Spot> {
        let (r, o) = hand(
            "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
             'd db 7d5h9d', 'p2 cc', 'p6 cc', 'd db 2s', 'p2 cbr 250', 'p6 f'",
        );
        spots(&r, &o).expect("reduces")
    }

    #[test]
    fn one_spot_per_postflop_decision() {
        let spots = worked();
        assert_eq!(spots.len(), 4);
        assert_eq!(spots.iter().map(Spot::seat).collect::<Vec<_>>(), [1, 5, 1, 5]);
        assert_eq!(
            spots.iter().map(Spot::street).collect::<Vec<_>>(),
            [Street::Flop, Street::Flop, Street::Turn, Street::Turn]
        );
    }

    /// The out-of-position seat acts first postflop and plays the big blind.
    #[test]
    fn seats_map_onto_heads_up_roles() {
        let spots = worked();
        assert_eq!(spots[0].witness().turn(), Turn::Choice(1));
        assert_eq!(spots[1].witness().turn(), Turn::Choice(0));
    }

    /// A flop decision must not be able to see the turn card.
    #[test]
    fn the_hero_sees_only_their_own_street() {
        let spots = worked();
        assert_eq!(spots[0].witness().seen().public().size(), 3);
        assert_eq!(spots[2].witness().seen().public().size(), 4);
    }

    #[test]
    fn each_hero_sees_their_own_hole_cards() {
        let spots = worked();
        // p2 is 8s4c, p6 is 6c7s.
        assert_eq!(spots[0].witness().seen().pocket(), &Hand::try_from("8s4c").unwrap());
        assert_eq!(spots[1].witness().seen().pocket(), &Hand::try_from("6c7s").unwrap());
    }

    /// The engine state carries both hands even though neither witness does —
    /// without it a rollout could never reach a showdown.
    #[test]
    fn the_game_holds_what_the_witness_hides() {
        let spots = worked();
        assert_eq!(spots[0].holes(), &[Hole::try_from("6c7s").unwrap(), Hole::try_from("8s4c").unwrap()]);
        assert_eq!(spots[0].game().turn(), spots[0].witness().turn());
        assert_eq!(spots[0].game().pot(), spots[0].pot());
    }

    /// The log stopped dealing when the button folded the turn, so four cards is
    /// everything there is — the river is [`crate::worlds`]'s problem.
    #[test]
    fn the_board_is_however_far_the_log_got() {
        assert_eq!(worked()[0].board().len(), 4);
    }

    #[test]
    fn actions_translate_into_engine_chips() {
        let spots = worked();
        assert_eq!(spots[0].played(), Action::Check);
        assert_eq!(spots[1].played(), Action::Check);
        // 250 into a 500 pot is a half-pot bet; at 1/50 scale, 5 into 10.
        assert_eq!(spots[2].pot(), 10);
        assert_eq!(spots[2].played(), Action::Raise(5));
        assert_eq!(spots[2].edge(), Edge::Raise(Odds::new(1, 2)));
        assert_eq!(spots[3].played(), Action::Fold);
    }

    #[test]
    fn a_blind_versus_blind_hand_yields_nothing() {
        let (r, o) = hand("'p3 f', 'p4 f', 'p5 f', 'p6 f', 'p1 cc', 'p2 cc', 'd db 7d5h9d', 'p1 cc', 'p2 cc'");
        assert!(spots(&r, &o).is_none());
    }

    /// A log played at a depth our grid was not anchored to is not a spot we can
    /// score, and saying so beats rescaling the witness and pretending.
    #[test]
    fn a_hand_at_the_wrong_stack_depth_is_dropped() {
        let text = "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
                    blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
                    starting_stacks = [20000, 20000, 20000, 20000, 20000, 20000]\n\
                    actions = ['d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                    'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s', \
                    'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
                    'd db 7d5h9d', 'p2 cc', 'p6 cc']\n";
        let record = crate::parse(text, "deep").unwrap();
        let outcome = crate::replay(&record).unwrap();
        assert!(spots(&record, &outcome).is_none());
    }
}

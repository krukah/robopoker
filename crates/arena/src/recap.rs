use deuce::*;
use kicker::*;
use parlor::records::Hand as HandRecord;
use pokerkit::*;

/// Per-hand result from replaying a stored hand.
#[derive(Debug, Clone)]
pub struct Recap {
    hand: ID<HandRecord>,
    /// The line, rebuilt from the stored plays.
    witness: Witness,
    seat: Position,
    hole: Hole,
    stack: Chips,
    won: Chips,
    pot: Chips,
    street: Street,
    folded: bool,
    showdown: bool,
    vpip: bool,
    pfr: bool,
    actions: Vec<(Street, Action)>,
}
impl Recap {
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub fn new(
        hand: ID<HandRecord>,
        witness: Witness,
        seat: Position,
        hole: Hole,
        stack: Chips,
        won: Chips,
        pot: Chips,
        street: Street,
        folded: bool,
        showdown: bool,
        vpip: bool,
        pfr: bool,
        actions: Vec<(Street, Action)>,
    ) -> Self {
        Self {
            hand,
            witness,
            seat,
            hole,
            stack,
            won,
            pot,
            street,
            folded,
            showdown,
            vpip,
            pfr,
            actions,
        }
    }

    pub fn hand(&self) -> ID<HandRecord> {
        self.hand
    }

    pub fn witness(&self) -> &Witness {
        &self.witness
    }

    pub fn seat(&self) -> Position {
        self.seat
    }

    pub fn hole(&self) -> Hole {
        self.hole
    }

    pub fn stack(&self) -> Chips {
        self.stack
    }

    pub fn won(&self) -> Chips {
        self.won
    }

    pub fn pot(&self) -> Chips {
        self.pot
    }

    pub fn street(&self) -> Street {
        self.street
    }

    pub fn folded(&self) -> bool {
        self.folded
    }

    pub fn showdown(&self) -> bool {
        self.showdown
    }

    pub fn vpip(&self) -> bool {
        self.vpip
    }

    pub fn pfr(&self) -> bool {
        self.pfr
    }

    pub fn actions(&self) -> &[(Street, Action)] {
        &self.actions
    }
}

impl Recap {}

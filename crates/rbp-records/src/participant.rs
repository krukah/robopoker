use super::*;
use rbp_auth::Member;
use rbp_cards::Hole;
use rbp_core::*;

/// Player participation in a hand.
/// Composite key: (hand_id, seat)
#[derive(Debug, Clone)]
pub struct Participant {
    hand: ID<Hand>,
    user: Option<ID<Member>>,
    seat: Position,
    hole: Hole,
    stack: Chips,
    showed: bool, // are these the same thing?
    mucked: bool, // are these the same thing?
}

impl Participant {
    pub fn new(
        hand: ID<Hand>,
        user: Option<ID<Member>>,
        seat: Position,
        hole: Hole,
        stack: Chips,
    ) -> Self {
        Self {
            hand,
            user,
            seat,
            hole,
            stack,
            showed: false,
            mucked: false,
        }
    }
    pub fn hand(&self) -> ID<Hand> {
        self.hand
    }
    pub fn user(&self) -> Option<ID<Member>> {
        self.user
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
    pub fn showed(&self) -> bool {
        self.showed
    }
    pub fn mucked(&self) -> bool {
        self.mucked
    }
    pub fn show(&mut self) {
        self.showed = true;
    }
    pub fn muck(&mut self) {
        self.mucked = true;
    }
}

#[cfg(feature = "database")]
mod schema {
    use super::*;
    use rbp_pg::*;

    impl Schema for Participant {
        fn name() -> &'static str {
            PLAYERS
        }
        fn columns() -> &'static [tokio_postgres::types::Type] {
            &[
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::INT2,
                tokio_postgres::types::Type::INT8,
                tokio_postgres::types::Type::INT2,
                tokio_postgres::types::Type::BOOL,
                tokio_postgres::types::Type::BOOL,
            ]
        }
        fn creates() -> &'static str {
            const_format::concatcp!(
                "CREATE TABLE IF NOT EXISTS ",
                PLAYERS,
                " (
                    hand_id     UUID NOT NULL REFERENCES ",
                HANDS,
                "(id) ON DELETE CASCADE,
                    user_id     UUID REFERENCES ",
                USERS,
                "(id),
                    seat        SMALLINT NOT NULL,
                    hole        BIGINT NOT NULL,
                    stack       SMALLINT NOT NULL,
                    showed      BOOLEAN DEFAULT FALSE, -- are these the same thing?
                    mucked      BOOLEAN DEFAULT FALSE, -- are these the same thing?
                    PRIMARY KEY (hand_id, seat)
                );"
            )
        }
        fn indices() -> &'static str {
            const_format::concatcp!(
                "CREATE INDEX IF NOT EXISTS idx_players_user ON ",
                PLAYERS,
                " (user_id);"
            )
        }
        fn copy() -> &'static str {
            unimplemented!()
        }
        fn truncates() -> &'static str {
            unimplemented!()
        }
        fn freeze() -> &'static str {
            unimplemented!()
        }
    }
}

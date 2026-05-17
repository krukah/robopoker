use super::*;
use rbp_auth::Member;
use rbp_cards::Hole;
use rbp_core::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Hidden = 0,
    Showed = 1,
    Mucked = 2,
}
impl From<i16> for Visibility {
    fn from(v: i16) -> Self {
        match v {
            1 => Visibility::Showed,
            2 => Visibility::Mucked,
            _ => Visibility::Hidden,
        }
    }
}
impl From<Visibility> for i16 {
    fn from(v: Visibility) -> Self {
        v as i16
    }
}

/// Player participation in a hand.
/// Composite key: (hand_id, seat)
#[derive(Debug, Clone)]
pub struct Participant {
    hand: ID<Hand>,
    user: Option<ID<Member>>,
    seat: Position,
    hole: Hole,
    stack: Chips,
    visibility: Visibility,
    pnl: Chips,
}

impl Participant {
    pub fn new(
        hand: ID<Hand>,
        user: Option<ID<Member>>,
        seat: Position,
        hole: Hole,
        stack: Chips,
        pnl: Chips,
    ) -> Self {
        Self {
            hand,
            user,
            seat,
            hole,
            stack,
            visibility: Visibility::default(),
            pnl,
        }
    }

    pub fn with_visibility(
        hand: ID<Hand>,
        user: Option<ID<Member>>,
        seat: Position,
        hole: Hole,
        stack: Chips,
        visibility: Visibility,
        pnl: Chips,
    ) -> Self {
        Self {
            hand,
            user,
            seat,
            hole,
            stack,
            visibility,
            pnl,
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

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn pnl(&self) -> Chips {
        self.pnl
    }

    pub fn show(&mut self) {
        self.visibility = Visibility::Showed;
    }

    pub fn muck(&mut self) {
        self.visibility = Visibility::Mucked;
    }
}

#[cfg(feature = "database")]
mod schema {
    use super::*;
    use rbp_database::*;
    use std::sync::OnceLock;

    impl Schema for Participant {
        fn name() -> &'static str {
            players()
        }

        fn columns() -> &'static [tokio_postgres::types::Type] {
            &[
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::INT2,
                tokio_postgres::types::Type::INT8,
                tokio_postgres::types::Type::INT2,
                tokio_postgres::types::Type::INT2,
                tokio_postgres::types::Type::INT2,
            ]
        }

        fn creates() -> &'static str {
            static SQL: OnceLock<&str> = OnceLock::<&str>::new();
            SQL.get_or_init(|| {
                leaked(format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    hand_id     UUID NOT NULL REFERENCES {}(id) ON DELETE CASCADE,
                    user_id     UUID REFERENCES {}(id),
                    seat        SMALLINT NOT NULL,
                    hole        BIGINT NOT NULL,
                    stack       SMALLINT NOT NULL,
                    visibility  SMALLINT NOT NULL DEFAULT 0,
                    pnl         SMALLINT NOT NULL DEFAULT 0,
                    PRIMARY KEY (hand_id, seat)
                );",
                    players(),
                    hands(),
                    users()
                ))
            })
        }

        fn indices() -> &'static str {
            static SQL: OnceLock<&str> = OnceLock::<&str>::new();
            SQL.get_or_init(|| {
                leaked(format!(
                    "CREATE INDEX IF NOT EXISTS idx_players_user_hand ON {} (user_id, hand_id);",
                    players()
                ))
            })
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

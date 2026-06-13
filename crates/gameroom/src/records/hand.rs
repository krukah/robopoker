use super::*;
use rbp_cards::*;
use rbp_core::*;

/// Persistent hand record for a completed poker hand.
#[derive(Debug, Clone)]
pub struct Hand {
    id: ID<Self>,
    room: ID<Room>,
    pot: Chips,
    board: Board,
    dealer: Position,
}

impl Hand {
    pub fn new(id: ID<Self>, room: ID<Room>, board: Board, pot: Chips, dealer: Position) -> Self {
        Self {
            id,
            room,
            pot,
            board,
            dealer,
        }
    }

    pub fn room(&self) -> ID<Room> {
        self.room
    }

    pub fn board(&self) -> Board {
        self.board
    }

    pub fn pot(&self) -> Chips {
        self.pot
    }

    pub fn dealer(&self) -> Position {
        self.dealer
    }
}

impl Unique for Hand {
    fn id(&self) -> ID<Self> {
        self.id
    }
}

#[cfg(feature = "server")]
mod schema {
    use super::*;
    use rbp_database::*;
    use std::sync::OnceLock;

    impl Schema for Hand {
        fn name() -> &'static str {
            hands()
        }

        fn columns() -> &'static [tokio_postgres::types::Type] {
            &[
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::INT8,
                tokio_postgres::types::Type::INT2,
                tokio_postgres::types::Type::INT2,
            ]
        }

        fn creates() -> &'static str {
            static SQL: OnceLock<&str> = OnceLock::<&str>::new();
            SQL.get_or_init(|| {
                leaked(format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    id          UUID PRIMARY KEY,
                    room_id     UUID NOT NULL REFERENCES {}(id),
                    board       BIGINT NOT NULL,
                    pot         SMALLINT NOT NULL,
                    dealer      SMALLINT NOT NULL
                );",
                    hands(),
                    rooms()
                ))
            })
        }

        fn indices() -> &'static str {
            static SQL: OnceLock<&str> = OnceLock::<&str>::new();
            SQL.get_or_init(|| leaked(format!("CREATE INDEX IF NOT EXISTS idx_hands_room ON {} (room_id);", hands())))
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

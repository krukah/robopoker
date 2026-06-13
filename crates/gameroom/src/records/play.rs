use super::*;
use rbp_auth::Member;
use rbp_core::*;
use rbp_gameplay::*;

/// Individual action in a hand.
/// Composite key: (hand_id, seq)
#[derive(Debug, Clone)]
pub struct Play {
    seq: Epoch,
    hand: ID<Hand>,
    player: Option<ID<Member>>,
    action: Action,
    elapsed: Option<i32>,
}

impl Play {
    pub fn new(hand: ID<Hand>, seq: Epoch, player: Option<ID<Member>>, action: Action, elapsed: Option<i32>) -> Self {
        Self {
            seq,
            hand,
            player,
            action,
            elapsed,
        }
    }

    pub fn seq(&self) -> Epoch {
        self.seq
    }

    pub fn hand(&self) -> ID<Hand> {
        self.hand
    }

    pub fn player(&self) -> Option<ID<Member>> {
        self.player
    }

    pub fn action(&self) -> Action {
        self.action
    }

    pub fn elapsed(&self) -> Option<i32> {
        self.elapsed
    }
}

#[cfg(feature = "server")]
mod schema {
    use super::*;
    use rbp_database::*;
    use std::sync::OnceLock;

    impl Schema for Play {
        fn name() -> &'static str {
            actions()
        }

        fn columns() -> &'static [tokio_postgres::types::Type] {
            &[
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::INT2,
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::INT4,
                tokio_postgres::types::Type::INT4,
            ]
        }

        fn creates() -> &'static str {
            static SQL: OnceLock<&str> = OnceLock::<&str>::new();
            SQL.get_or_init(|| {
                leaked(format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    hand_id     UUID NOT NULL REFERENCES {}(id) ON DELETE CASCADE,
                    seq         SMALLINT NOT NULL,
                    player_id   UUID REFERENCES {}(id),
                    encoded     INTEGER NOT NULL,
                    elapsed_ms  INTEGER,
                    PRIMARY KEY (hand_id, seq)
                );",
                    actions(),
                    hands(),
                    users()
                ))
            })
        }

        fn indices() -> &'static str {
            static SQL: OnceLock<&str> = OnceLock::<&str>::new();
            SQL.get_or_init(|| {
                leaked(format!("CREATE INDEX IF NOT EXISTS idx_actions_player ON {} (player_id);", actions()))
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

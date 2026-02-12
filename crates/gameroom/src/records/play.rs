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
}

impl Play {
    pub fn new(hand: ID<Hand>, seq: Epoch, player: Option<ID<Member>>, action: Action) -> Self {
        Self {
            hand,
            seq,
            player,
            action,
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
}

#[cfg(feature = "database")]
mod schema {
    use super::*;
    use rbp_database::*;

    impl Schema for Play {
        fn name() -> &'static str {
            ACTIONS
        }
        fn columns() -> &'static [tokio_postgres::types::Type] {
            &[
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::INT2,
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::INT4,
            ]
        }
        fn creates() -> &'static str {
            const_format::concatcp!(
                "CREATE TABLE IF NOT EXISTS ",
                ACTIONS,
                " (
                    hand_id     UUID NOT NULL REFERENCES ",
                HANDS,
                "(id) ON DELETE CASCADE,
                    seq         SMALLINT NOT NULL,
                    player_id   UUID REFERENCES ",
                USERS,
                "(id),
                    encoded     INTEGER NOT NULL,
                    PRIMARY KEY (hand_id, seq)
                );"
            )
        }
        fn indices() -> &'static str {
            const_format::concatcp!(
                "CREATE INDEX IF NOT EXISTS idx_actions_player ON ",
                ACTIONS,
                " (player_id);"
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

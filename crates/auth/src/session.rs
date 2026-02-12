use super::*;
use rbp_core::ID;
use rbp_core::Unique;

/// Persisted session for token management.
#[derive(Debug, Clone)]
pub struct Session {
    id: ID<Self>,
    user: ID<Member>,
    hash: Vec<u8>,
    expires: std::time::SystemTime,
    // can do something with this field later
    #[allow(unused)]
    revoked: bool,
}

impl Unique for Session {
    fn id(&self) -> ID<Self> {
        self.id
    }
}

impl Session {
    pub fn new(id: ID<Self>, user: ID<Member>, hash: Vec<u8>) -> Self {
        Self {
            id,
            user,
            hash,
            expires: std::time::SystemTime::now() + Crypto::duration(),
            revoked: false,
        }
    }
    pub fn user(&self) -> ID<Member> {
        self.user
    }
    pub fn hash(&self) -> &[u8] {
        &self.hash
    }
    pub fn expires_at(&self) -> std::time::SystemTime {
        self.expires
    }
}

#[cfg(feature = "database")]
mod schema {
    use super::*;
    use rbp_database::*;

    impl Schema for Session {
        fn name() -> &'static str {
            SESSIONS
        }
        fn columns() -> &'static [tokio_postgres::types::Type] {
            &[
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::BYTEA,
                tokio_postgres::types::Type::TIMESTAMPTZ,
                tokio_postgres::types::Type::BOOL,
            ]
        }
        fn creates() -> &'static str {
            const_format::concatcp!(
                "CREATE TABLE IF NOT EXISTS ",
                SESSIONS,
                " (
                    id          UUID PRIMARY KEY,
                    user_id     UUID NOT NULL REFERENCES ",
                USERS,
                "(id) ON DELETE CASCADE,
                    token_hash  BYTEA NOT NULL,
                    expires_at  TIMESTAMPTZ NOT NULL,
                    revoked     BOOLEAN DEFAULT FALSE
                );"
            )
        }
        fn indices() -> &'static str {
            const_format::concatcp!(
                "CREATE INDEX IF NOT EXISTS idx_sessions_user ON ",
                SESSIONS,
                " (user_id);
                 CREATE INDEX IF NOT EXISTS idx_sessions_token ON ",
                SESSIONS,
                " (token_hash);
                 CREATE INDEX IF NOT EXISTS idx_sessions_expires ON ",
                SESSIONS,
                " (expires_at) WHERE NOT revoked;"
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

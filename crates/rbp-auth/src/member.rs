use rbp_core::ID;
use rbp_core::Unique;

/// Authenticated user with verified identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Member {
    id: ID<Self>,
    username: String,
    email: String,
}

impl Member {
    pub fn new(id: ID<Self>, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
        }
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn email(&self) -> &str {
        &self.email
    }
}

impl Unique for Member {
    fn id(&self) -> ID<Self> {
        self.id
    }
}

#[cfg(feature = "database")]
mod schema {
    use super::*;
    use rbp_pg::*;

    /// Schema implementation for Member (users table).
    /// Note: hashword is a database-only field, not part of Member domain type.
    impl Schema for Member {
        fn name() -> &'static str {
            USERS
        }
        fn columns() -> &'static [tokio_postgres::types::Type] {
            &[
                tokio_postgres::types::Type::UUID,
                tokio_postgres::types::Type::VARCHAR,
                tokio_postgres::types::Type::VARCHAR,
                tokio_postgres::types::Type::TEXT,
            ]
        }
        fn creates() -> &'static str {
            const_format::concatcp!(
                "CREATE TABLE IF NOT EXISTS ",
                USERS,
                " (
                    id          UUID PRIMARY KEY,
                    username    VARCHAR(32) UNIQUE NOT NULL,
                    email       VARCHAR(255) UNIQUE NOT NULL,
                    hashword    TEXT NOT NULL
                );"
            )
        }
        fn indices() -> &'static str {
            const_format::concatcp!(
                "CREATE INDEX IF NOT EXISTS idx_users_username ON ",
                USERS,
                " (username);
                 CREATE INDEX IF NOT EXISTS idx_users_email ON ",
                USERS,
                " (email);"
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

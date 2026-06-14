use pokerkit::ID;
use pokerkit::Unique;

/// Authenticated user with verified identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Member {
    id: ID<Self>,
    username: String,
    email: String,
}

impl Member {
    pub fn new(id: ID<Self>, username: String, email: String) -> Self {
        Self { id, username, email }
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

#[cfg(feature = "server")]
mod schema {
    use super::*;
    use ledger::*;
    use std::sync::OnceLock;

    /// Schema implementation for Member (users table).
    /// Note: hashword is a database-only field, not part of Member domain type.
    impl Schema for Member {
        fn name() -> &'static str {
            users()
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
            static SQL: OnceLock<&str> = OnceLock::<&str>::new();
            SQL.get_or_init(|| {
                leaked(format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    id          UUID PRIMARY KEY,
                    username    VARCHAR(32) UNIQUE NOT NULL,
                    email       VARCHAR(255) UNIQUE NOT NULL,
                    hashword    TEXT NOT NULL
                );",
                    users()
                ))
            })
        }

        fn indices() -> &'static str {
            static SQL: OnceLock<&str> = OnceLock::<&str>::new();
            SQL.get_or_init(|| {
                leaked(format!(
                    "CREATE INDEX IF NOT EXISTS idx_users_username ON {} (username);
                 CREATE INDEX IF NOT EXISTS idx_users_email ON {} (email);",
                    users(),
                    users()
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

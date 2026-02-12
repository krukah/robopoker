use super::*;
use rbp_core::ID;
use rbp_core::Unique;
use rbp_pg::*;
use std::sync::Arc;
use tokio_postgres::Client;

/// Repository trait for authentication database operations.
/// Abstracts SQL from domain modules.
#[allow(async_fn_in_trait)]
pub trait AuthRepository {
    async fn signin(&self, session: &Session) -> Result<(), PgErr>;
    async fn revoke(&self, session: ID<Session>) -> Result<(), PgErr>;
    async fn exists(&self, username: &str, email: &str) -> Result<bool, PgErr>;
    async fn create(&self, member: &Member, hashword: &str) -> Result<(), PgErr>;
    async fn lookup(&self, username: &str) -> Result<Option<(Member, String)>, PgErr>;
}

impl AuthRepository for Arc<Client> {
    async fn exists(&self, username: &str, email: &str) -> Result<bool, PgErr> {
        self.query_opt(
            const_format::concatcp!(
                "SELECT 1 FROM ",
                USERS,
                " WHERE username = $1 OR email = $2"
            ),
            &[&username, &email],
        )
        .await
        .map(|opt| opt.is_some())
    }

    async fn create(&self, member: &Member, hashword: &str) -> Result<(), PgErr> {
        self.execute(
            const_format::concatcp!(
                "INSERT INTO ",
                USERS,
                " (id, username, email, hashword) VALUES ($1, $2, $3, $4)"
            ),
            &[
                &member.id().inner(),
                &member.username(),
                &member.email(),
                &hashword,
            ],
        )
        .await
        .map(|_| ())
    }

    async fn lookup(&self, username: &str) -> Result<Option<(Member, String)>, PgErr> {
        self.query_opt(
            const_format::concatcp!(
                "SELECT id, username, email, hashword FROM ",
                USERS,
                " WHERE username = $1"
            ),
            &[&username],
        )
        .await
        .map(|opt| {
            opt.map(|row| {
                (
                    Member::new(
                        ID::from(row.get::<_, uuid::Uuid>(0)),
                        row.get::<_, String>(1),
                        row.get::<_, String>(2),
                    ),
                    row.get::<_, String>(3),
                )
            })
        })
    }

    async fn signin(&self, session: &Session) -> Result<(), PgErr> {
        self.execute(
            const_format::concatcp!(
                "INSERT INTO ",
                SESSIONS,
                " (id, user_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)"
            ),
            &[
                &session.id().inner(),
                &session.user().inner(),
                &session.hash(),
                &session.expires_at(),
            ],
        )
        .await
        .map(|_| ())
    }

    async fn revoke(&self, session: ID<Session>) -> Result<(), PgErr> {
        self.execute(
            const_format::concatcp!("UPDATE ", SESSIONS, " SET revoked = TRUE WHERE id = $1"),
            &[&session.inner()],
        )
        .await
        .map(|_| ())
    }
}

use super::*;
use rbp_core::ID;
use rbp_core::Unique;
use rbp_database::*;
use std::sync::Arc;
use std::sync::OnceLock;
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
    async fn seed(&self, member: &Member) -> Result<(), PgErr>;
}

impl AuthRepository for Arc<Client> {
    async fn exists(&self, username: &str, email: &str) -> Result<bool, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT 1 FROM {} WHERE username = $1 OR email = $2", users()));
        self.query_opt(sql.as_str(), &[&username, &email])
            .await
            .map(|opt| opt.is_some())
    }

    async fn create(&self, member: &Member, hashword: &str) -> Result<(), PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL
            .get_or_init(|| format!("INSERT INTO {} (id, username, email, hashword) VALUES ($1, $2, $3, $4)", users()));
        self.execute(sql.as_str(), &[&member.id().inner(), &member.username(), &member.email(), &hashword])
            .await
            .map(|_| ())
    }

    async fn lookup(&self, username: &str) -> Result<Option<(Member, String)>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql =
            SQL.get_or_init(|| format!("SELECT id, username, email, hashword FROM {} WHERE username = $1", users()));
        self.query_opt(sql.as_str(), &[&username]).await.map(|opt| {
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
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!("INSERT INTO {} (id, user_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)", sessions())
        });
        self.execute(
            sql.as_str(),
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
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("UPDATE {} SET revoked = TRUE WHERE id = $1", sessions()));
        self.execute(sql.as_str(), &[&session.inner()]).await.map(|_| ())
    }

    async fn seed(&self, member: &Member) -> Result<(), PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "INSERT INTO {} (id, username, email, hashword) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
                users()
            )
        });
        self.execute(sql.as_str(), &[&member.id().inner(), &member.username(), &member.email(), &"!bot"])
            .await
            .map(|_| ())
    }
}

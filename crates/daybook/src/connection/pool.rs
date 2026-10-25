use std::sync::Arc;
use tokio_postgres::Client;

/// Connection pooling and string interning.
pub struct Pool;

impl Pool {
    /// Connects to PostgreSQL via the `DB_URL` env var, sharable across tasks.
    ///
    /// Panics if `DB_URL` is unset or the connection fails.
    pub async fn db() -> Arc<Client> {
        tracing::info!("connecting to database");
        let tls = tokio_postgres::tls::NoTls;
        let ref url = std::env::var("DB_URL").expect("DB_URL must be set");
        let (client, connection) = tokio_postgres::connect(url, tls)
            .await
            .expect("database connection failed");
        tokio::spawn(async move {
            connection
                .await
                .inspect_err(|e| tracing::error!(error = %e, "database connection lost"))
                .ok();
        });
        client
            .execute("SET client_min_messages TO WARNING", &[])
            .await
            .expect("set client_min_messages");
        Arc::new(client)
    }

    /// Leaks a formatted string to obtain a `&'static str`.
    /// Used once per table name at process startup.
    pub fn leaked(s: String) -> &'static str {
        Box::leak(s.into_boxed_str())
    }
}

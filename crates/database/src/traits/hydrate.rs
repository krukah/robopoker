//! Loading domain objects from PostgreSQL.
use tokio_postgres::Client;

/// Loading domain objects from PostgreSQL.
///
/// Complements [`Schema`](crate::Schema) and [`Streamable`](crate::Streamable)
/// to enable round-trip persistence. While those traits handle writing,
/// `Hydrate` handles reading data back into memory.
#[async_trait::async_trait]
pub trait Hydrate: Sized {
    /// Loads this type from the database.
    ///
    /// Takes an `Arc<Client>` to allow the implementation to spawn
    /// concurrent queries if needed.
    async fn hydrate(client: std::sync::Arc<Client>) -> Self;
}

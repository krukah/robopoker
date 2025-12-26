/// Types that can be loaded from Postgres.
/// Complement to Schema/Streamable traits for round-trip persistence.
#[async_trait::async_trait]
pub trait Hydrate: Sized {
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self;
}

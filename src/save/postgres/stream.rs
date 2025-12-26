use super::*;
use tokio_postgres::Client;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// Trait for types that can stream to Postgres via binary COPY.
/// Requires Schema for table metadata.
#[async_trait::async_trait]
pub trait Streamable: Schema + Sized + Send {
    type Row: Row;

    /// Convert self into an iterator of rows.
    fn rows(self) -> impl Iterator<Item = Self::Row> + Send;

    /// Stream rows to postgres via binary COPY.
    /// Default implementation uses rows() and Row::write().
    async fn stream(self, client: &Client) {
        let sink = client.copy_in(Self::copy()).await.expect("copy_in");
        let writer = BinaryCopyInWriter::new(sink, Self::columns());
        futures::pin_mut!(writer);
        for row in self.rows() {
            row.write(writer.as_mut()).await;
        }
        writer.finish().await.expect("finish");
    }
    /// Create indices and freeze table (call once after all data uploaded).
    async fn finalize(client: &Client) {
        log::info!("indexing table ({})", Self::name());
        client
            .batch_execute(Self::indices())
            .await
            .expect("indices");
        log::info!("freezing table ({})", Self::name());
        client.batch_execute(Self::freeze()).await.expect("freeze");
    }
}

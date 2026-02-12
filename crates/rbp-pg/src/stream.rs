use super::*;
use tokio_postgres::Client;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// Bulk data upload via PostgreSQL's binary COPY protocol.
///
/// Enables high-throughput streaming of domain objects to the database
/// using PostgreSQL's most efficient data ingestion path. The binary
/// format avoids text parsing overhead and matches Rust's native types.
///
/// # Requirements
///
/// Implementors must also implement [`Schema`] for table metadata and
/// define a [`Row`] type that handles binary serialization.
///
/// # Performance
///
/// Binary COPY is orders of magnitude faster than INSERT statements
/// for bulk loading. A typical clustering run uploads millions of rows
/// in seconds rather than hours.
#[async_trait::async_trait]
pub trait Streamable: Schema + Sized + Send {
    /// The row type for binary serialization.
    type Row: Row;
    /// Converts this collection into an iterator of rows for streaming.
    fn rows(self) -> impl Iterator<Item = Self::Row> + Send;
    /// Streams all rows to PostgreSQL via binary COPY.
    ///
    /// Opens a COPY stream, writes each row in binary format, and
    /// finalizes the upload. Consumes `self` to enable move semantics.
    async fn stream(self, client: &Client) {
        let sink = client.copy_in(Self::copy()).await.expect("copy_in");
        let writer = BinaryCopyInWriter::new(sink, Self::columns());
        futures::pin_mut!(writer);
        for row in self.rows() {
            row.write(writer.as_mut()).await;
        }
        writer.finish().await.expect("finish");
    }
    /// Creates indices and optimizes table for read-heavy access.
    ///
    /// Call once after all data has been uploaded. Creates indices
    /// defined by [`Schema::indices`] and applies freeze settings.
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

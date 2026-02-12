//! PostgreSQL serialization traits.
//!
//! Traits for table metadata, bulk loading, and round-trip persistence.
use std::pin::Pin;
use tokio_postgres::Client;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// Schema metadata for PostgreSQL tables.
///
/// Provides compile-time SQL generation for table creation, indexing,
/// and bulk data operations. All methods return `&'static str` to avoid
/// runtime allocations and enable compile-time string construction via
/// [`const_format::concatcp!`].
///
/// # Design
///
/// This trait contains no I/O operations—it purely describes table structure.
/// Actual database operations are handled by [`Streamable`] and [`Hydrate`].
pub trait Schema {
    /// Returns the table name in the database.
    fn name() -> &'static str;
    /// Returns the `COPY ... FROM STDIN BINARY` command for bulk loading.
    fn copy() -> &'static str;
    /// Returns `CREATE TABLE IF NOT EXISTS` DDL statement.
    fn creates() -> &'static str;
    /// Returns `CREATE INDEX IF NOT EXISTS` statements for all indices.
    fn indices() -> &'static str;
    /// Returns `TRUNCATE TABLE` statement for clearing data.
    fn truncates() -> &'static str;
    /// Returns SQL to optimize table for read-heavy workloads.
    ///
    /// Typically sets `fillfactor = 100` and disables autovacuum for
    /// tables that are bulk-loaded once and never modified.
    fn freeze() -> &'static str;
    /// Returns PostgreSQL column types for binary COPY protocol.
    fn columns() -> &'static [tokio_postgres::types::Type];
}

/// Derived table generation from enumerable domain values.
///
/// For tables whose contents can be exhaustively enumerated at runtime
/// (e.g., street configurations, abstraction definitions), this trait
/// generates INSERT statements programmatically.
///
/// # Usage
///
/// Implement [`exhaust`](Derive::exhaust) to enumerate all valid values,
/// and [`inserts`](Derive::inserts) to format each as an INSERT statement.
/// The [`derives`](Derive::derives) method combines these into a single
/// SQL batch.
///
/// # Contrast with Streamable
///
/// Use `Derive` for small, enumerable tables where INSERT is sufficient.
/// Use [`Streamable`] for large datasets requiring binary COPY performance.
pub trait Derive: Sized + Schema {
    /// Enumerates all values that should be inserted into the table.
    fn exhaust() -> Vec<Self>;
    /// Formats this value as an INSERT statement.
    fn inserts(&self) -> String;
    /// Generates a batch of INSERT statements for all enumerated values.
    fn derives() -> String {
        Self::exhaust()
            .iter()
            .map(Self::inserts)
            .collect::<Vec<_>>()
            .join("\n;")
    }
}

/// Loading domain objects from PostgreSQL.
///
/// Complements [`Schema`] and [`Streamable`] to enable round-trip
/// persistence. While those traits handle writing, `Hydrate` handles
/// reading data back into memory.
#[async_trait::async_trait]
pub trait Hydrate: Sized {
    /// Loads this type from the database.
    ///
    /// Takes an `Arc<Client>` to allow the implementation to spawn
    /// concurrent queries if needed.
    async fn hydrate(client: std::sync::Arc<Client>) -> Self;
}

/// Binary row serialization for PostgreSQL COPY protocol.
///
/// Each implementation handles a specific tuple arity, writing fields
/// in binary format to match the table schema. The trait enables
/// [`Streamable`] to work with any row shape.
///
/// # Safety
///
/// Field order and types must exactly match the table schema defined
/// by the corresponding [`Schema`] implementation.
#[async_trait::async_trait]
pub trait Row: Send {
    /// Writes this row to the binary COPY stream.
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>);
}

/// Row format for isomorphism → abstraction mappings.
#[async_trait::async_trait]
impl Row for (i64, i16) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer.write(&[&self.0, &self.1]).await.expect("write");
    }
}

/// Row format for triangular index → distance mappings.
#[async_trait::async_trait]
impl Row for (i32, f32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer.write(&[&self.0, &self.1]).await.expect("write");
    }
}

/// Row format for transition probabilities.
#[async_trait::async_trait]
impl Row for (i16, i16, f32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer.write(&[&self.0, &self.1, &self.2]).await.expect("write");
    }
}

/// Row format for blueprint strategies.
#[rustfmt::skip]
#[async_trait::async_trait]
impl Row for (i64, i16, i64, i64, f32, f32, f32, i32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer
            .write(&[&self.0, &self.1, &self.2, &self.3, &self.4, &self.5, &self.6, &self.7])
            .await
            .expect("write");
    }
}

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

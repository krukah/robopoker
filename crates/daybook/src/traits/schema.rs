//! Table metadata and DDL generation.

/// Schema metadata for PostgreSQL tables — pure description, no I/O.
///
/// All methods return `&'static str` to avoid runtime allocation and allow
/// compile-time construction via [`const_format::concatcp!`]. The actual
/// database operations live in [`Streamable`](crate::Streamable) and
/// [`Hydrate`](crate::Hydrate).
pub trait Schema {
    /// Returns the table name in the database.
    fn name() -> &'static str;
    /// Returns the `COPY ... FROM STDIN BINARY` command for bulk loading.
    fn copy() -> &'static str;
    /// Returns `CREATE TABLE IF NOT EXISTS` DDL statement.
    fn creates() -> &'static str;
    /// Returns `CREATE INDEX IF NOT EXISTS` statements for all indices.
    /// Implementations may also fold in idempotent derived-column
    /// population SQL (e.g. UPDATE for a per-group row index) — those
    /// statements run before the CREATE INDEX so indices land on
    /// populated rows. See `Lookup::indices` in clustering for an
    /// example.
    fn indices() -> &'static str;
    /// Returns `TRUNCATE TABLE` statement for clearing data.
    fn truncates() -> &'static str;
    /// SQL to optimize the table for read-heavy workloads — typically
    /// `fillfactor = 100` plus autovacuum off for write-once tables.
    fn freeze() -> &'static str;
    /// Returns PostgreSQL column types for binary COPY protocol.
    fn columns() -> &'static [tokio_postgres::types::Type];
}

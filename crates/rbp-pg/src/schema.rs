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

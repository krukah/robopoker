/// Pure schema definitions for Postgres tables.
/// No I/O operations - just metadata about table structure.
/// All methods return &'static str to avoid runtime allocations.
/// Use const_format::concatcp! to build SQL strings at compile time.
pub trait Schema {
    /// Returns the name of the table in the database.
    fn name() -> &'static str;
    /// Returns the COPY command used to load data into the database.
    fn copy() -> &'static str;
    /// Returns the SQL to prepare the table schema.
    fn creates() -> &'static str;
    /// Returns the SQL to create indices on the table.
    fn indices() -> &'static str;
    /// Returns the SQL to truncate the table.
    fn truncates() -> &'static str;
    /// Returns the SQL to freeze the table (disable autovacuum, set fillfactor).
    fn freeze() -> &'static str;
    /// Returns the column types for the table.
    fn columns() -> &'static [tokio_postgres::types::Type];
}

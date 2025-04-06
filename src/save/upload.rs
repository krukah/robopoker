use tokio_postgres::types::Type;

// blueprint    ~ 154M, (grows with number of CFR iterations)
// isomorphism  ~ 139M,
// metric       ~ 40K,
// transition   ~ 29K,
// abstraction  ~ 500,
// street       ~ 4

/// things that can be written to and read from disk, and uploaded into Postgres.
/// may or may not be dependent on other entities being written/in memory.
/// dependencies for methods returning Self are up to the implementor.
pub trait Table {
    /// Returns the name of the table in the database
    fn name() -> String;
    /// Returns the COPY command used to load data into the database
    fn copy() -> String;
    /// Returns the SQL to prepare the table schema
    fn creates() -> String;
    /// Returns the SQL to create indices on the table
    fn indices() -> String;
    /// Returns the column types for the table
    fn columns() -> &'static [Type];
    /// Returns the source file paths to load data from
    fn sources() -> Vec<String>;
    /// query to nuke table in Postgres
    fn truncates() -> String {
        format!(
            "TRUNCATE TABLE {}; ALTER TABLE {} SET UNLOGGED;",
            Self::name(),
            Self::name()
        )
    }
}

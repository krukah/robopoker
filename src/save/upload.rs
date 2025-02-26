use crate::cards::street::Street;
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
pub trait Upload {
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
    /// build from scratch
    fn grow(street: Street) -> Self;
    /// read from disk
    fn load(street: Street) -> Self;
    /// write to disk
    fn save(&self);

    /// query to nuke table in Postgres
    fn truncates() -> String {
        format!(
            "TRUNCATE TABLE {}; ALTER TABLE {} SET UNLOGGED;",
            Self::name(),
            Self::name()
        )
    }
    /// path to file on disk
    fn path(street: Street) -> String {
        format!(
            "{}/pgcopy/{}.{}",
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            Self::name(),
            street
        )
    }
    /// check if file exists on disk
    fn done(street: Street) -> bool {
        std::fs::metadata(Self::path(street)).is_ok()
    }
    /// Postgres signature header + 8 null bytes for flags and extension
    /// header for binary copy: https://www.postgresql.org/docs/current/static/sql-copy.html
    fn header() -> &'static [u8] {
        b"PGCOPY\n\xFF\r\n\0\0\0\0\0\0\0\0\0"
    }
    /// Postgres signature footer to signal end of binary file
    fn footer() -> u16 {
        0xFFFF
    }
}

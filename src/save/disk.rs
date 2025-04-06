use crate::cards::street::Street;

// #[cfg(feature = "native")]
/// for types that can be written to
/// and loaded from disk, which is
/// not necessarily the same as those
/// which can be serialized to and from
/// database.
pub trait Disk {
    /// Returns the name of the entity. should be consistentn with Table impl
    fn name() -> String;
    /// build from scratch
    fn grow(street: Street) -> Self;
    /// read from disk
    fn load(street: Street) -> Self;
    /// write to disk
    fn save(&self);
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

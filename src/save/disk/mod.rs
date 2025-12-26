#[cfg(feature = "database")]
mod writer;
#[cfg(feature = "database")]
pub use writer::*;

use crate::cards::*;

/// Legacy file I/O trait for types that can be written to
/// and loaded from disk in Postgres binary COPY format.
pub trait Disk {
    /// Returns the name of the entity. Should match Schema::name() if both are implemented.
    fn name() -> &'static str;
    /// Build from scratch.
    fn grow(street: Street) -> Self;
    /// Read from disk.
    fn load(street: Street) -> Self;
    /// Write to disk.
    fn save(&self);
    /// Returns source file paths to load data from.
    fn sources() -> Vec<std::path::PathBuf>;
    /// Path to file on disk.
    fn path(street: Street) -> std::path::PathBuf {
        let ref path = format!(
            "{}/pgcopy/{}.{}",
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            Self::name(),
            street
        );
        std::path::Path::new(path).parent().map(std::fs::create_dir);
        std::path::PathBuf::from(path)
    }
    /// Check if file exists on disk.
    fn done(street: Street) -> bool {
        std::fs::metadata(Self::path(street)).is_ok()
    }
    /// Postgres signature header + 8 null bytes for flags and extension.
    fn header() -> &'static [u8] {
        b"PGCOPY\n\xFF\r\n\0\0\0\0\0\0\0\0\0"
    }
    /// Postgres signature footer to signal end of binary file.
    fn footer() -> u16 {
        0xFFFF
    }
}

use super::*;

/// Trait for deriving database tables from other tables.
pub trait Derive: Sized + Schema {
    fn exhaust() -> Vec<Self>;
    fn inserts(&self) -> String;
    fn derives() -> String {
        Self::exhaust()
            .iter()
            .map(Self::inserts)
            .collect::<Vec<_>>()
            .join("\n;")
    }
}

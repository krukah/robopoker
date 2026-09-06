//! INSERT statement generation for enumerable types.
use crate::Schema;

/// Derived table generation from enumerable domain values.
///
/// For small tables whose contents can be exhaustively enumerated at runtime
/// (street configurations, abstraction definitions). Use
/// [`Streamable`](crate::Streamable) instead for large datasets that need
/// binary COPY throughput.
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

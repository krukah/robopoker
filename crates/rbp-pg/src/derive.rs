use super::*;

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

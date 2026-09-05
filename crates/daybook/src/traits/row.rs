//! Binary row serialization for PostgreSQL COPY protocol.
use std::pin::Pin;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// Binary row serialization for PostgreSQL COPY protocol.
///
/// Each implementation handles a specific tuple arity, writing fields
/// in binary format to match the table schema. The trait enables
/// [`Streamable`](crate::Streamable) to work with any row shape.
///
/// # Safety
///
/// Field order and types must exactly match the table schema defined
/// by the corresponding [`Schema`](crate::Schema) implementation.
#[async_trait::async_trait]
pub trait Row: Send {
    /// Writes this row to the binary COPY stream.
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>);
}

// Concrete row types live beside the domain types they serialize —
// `nlhe::Wire` (blueprint), `lloyd::{Mapping, Shift, Gap}` (isomorphism,
// transitions, metric) — so every column is written as its domain type via
// the ToSql codecs on those types, never as an anonymous primitive tuple.

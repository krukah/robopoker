//! Binary row serialization for PostgreSQL COPY protocol.
use std::pin::Pin;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// Binary row serialization for PostgreSQL COPY protocol.
///
/// Each implementation handles a specific tuple arity, writing fields in binary format to match
/// the table schema. The trait enables [`Streamable`](crate::Streamable) to work with any row
/// shape.
///
/// # Safety
#[async_trait::async_trait]
pub trait Row: Send {
    /// Writes this row to the binary COPY stream.
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>);
}

// Concrete row types live beside the domain types they serialize, so every column is written
// as its domain type through that type's ToSql codec, never as an anonymous primitive tuple.

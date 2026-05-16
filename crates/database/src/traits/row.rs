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

/// Row format for isomorphism → abstraction mappings.
#[async_trait::async_trait]
impl Row for (i64, i16) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer.write(&[&self.0, &self.1]).await.expect("write");
    }
}

/// Row format for triangular index → distance mappings.
#[async_trait::async_trait]
impl Row for (i32, f32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer.write(&[&self.0, &self.1]).await.expect("write");
    }
}

/// Row format for transition probabilities.
#[async_trait::async_trait]
impl Row for (i16, i16, f32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer
            .write(&[&self.0, &self.1, &self.2])
            .await
            .expect("write");
    }
}

/// Row format for blueprint strategies.
/// `(past, present, choices, geometry, edge, weight, regret, payoff, visits)`.
/// Geometry is the SPR bucket on the infoset key (see `Geometry` in
/// `crates/holdem/src/geometry.rs`).
#[rustfmt::skip]
#[async_trait::async_trait]
impl Row for (i64, i16, i64, i16, i64, f32, f32, f32, i32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer
            .write(&[&self.0, &self.1, &self.2, &self.3, &self.4, &self.5, &self.6, &self.7, &self.8])
            .await
            .expect("write");
    }
}

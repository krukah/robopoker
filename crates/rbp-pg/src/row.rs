use std::pin::Pin;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// Binary row serialization for PostgreSQL COPY protocol.
///
/// Each implementation handles a specific tuple arity, writing fields
/// in binary format to match the table schema. The trait enables
/// [`Streamable`] to work with any row shape.
///
/// # Safety
///
/// Field order and types must exactly match the table schema defined
/// by the corresponding [`Schema`] implementation.
#[async_trait::async_trait]
pub trait Row: Send {
    /// Writes this row to the binary COPY stream.
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>);
}

/// Row format for isomorphism → abstraction mappings.
///
/// - `i64`: Observation encoding (card combination)
/// - `i16`: Abstraction bucket index
#[async_trait::async_trait]
impl Row for (i64, i16) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer //   (obs,     abs)
            .write(&[&self.0, &self.1])
            .await
            .expect("write");
    }
}

/// Row format for triangular index → distance mappings.
///
/// - `i32`: Triangular matrix index for (i, j) pair
/// - `f32`: EMD distance between abstractions
#[async_trait::async_trait]
impl Row for (i32, f32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer //   (tri,     dx)
            .write(&[&self.0, &self.1])
            .await
            .expect("write");
    }
}

/// Row format for transition probabilities.
///
/// - `i16`: Source abstraction bucket
/// - `i16`: Target abstraction bucket
/// - `f32`: Transition weight
#[async_trait::async_trait]
impl Row for (i16, i16, f32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer //   (prev,    next,    dx)
            .write(&[&self.0, &self.1, &self.2])
            .await
            .expect("write");
    }
}

/// Row format for blueprint strategies.
///
/// - `i64`: Past action encoding (subgame history)
/// - `i16`: Present abstraction bucket
/// - `i64`: Choices encoding (available edges)
/// - `i64`: Edge encoding (action taken)
/// - `f32`: Strategy weight
/// - `f32`: Cumulative regret
/// - `f32`: Expected value
/// - `i32`: Encounter counts
#[async_trait::async_trait]
impl Row for (i64, i16, i64, i64, f32, f32, f32, i32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer //   (past,    present, choices, edge,    weight,  regret,  evalue,  counts)
            .write(&[
                &self.0, &self.1, &self.2, &self.3, &self.4, &self.5, &self.6, &self.7,
            ])
            .await
            .expect("write");
    }
}

//! Transitions-table row on the binary COPY wire.
//!
//! Requires the `server` feature.
use kicker::*;
use pokerkit::*;
use std::pin::Pin;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// One transitions-table row in domain types, column order
/// `(prev, next, dx)` — a street-to-street bucket transition and its
/// probability mass. Codecs live on the domain types.
pub struct Shift {
    prev: Abstraction,
    next: Abstraction,
    dx: Energy,
}

impl From<(Abstraction, Abstraction, Energy)> for Shift {
    fn from((prev, next, dx): (Abstraction, Abstraction, Energy)) -> Self {
        Self { prev, next, dx }
    }
}

#[async_trait::async_trait]
impl daybook::Row for Shift {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer.write(&[&self.prev, &self.next, &self.dx]).await.expect("write");
    }
}

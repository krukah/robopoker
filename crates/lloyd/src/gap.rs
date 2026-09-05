//! Metric-table row on the binary COPY wire.
//!
//! Requires the `server` feature.
use pokerkit::*;
use std::pin::Pin;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// One metric-table row, column order `(tri, dx)` — the triangular pair
/// index of two buckets and the EMD between them.
pub struct Gap {
    tri: i32,
    dx: Energy,
}

impl From<(i32, Energy)> for Gap {
    fn from((tri, dx): (i32, Energy)) -> Self {
        Self { tri, dx }
    }
}

#[async_trait::async_trait]
impl daybook::Row for Gap {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer.write(&[&self.tri, &self.dx]).await.expect("write");
    }
}

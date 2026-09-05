//! Isomorphism-table row on the binary COPY wire.
//!
//! Requires the `server` feature.
use deuce::*;
use kicker::*;
use std::pin::Pin;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// One isomorphism-table row in domain types, column order
/// `(obs, abs, position)` — the suit-canonical observation, its learned
/// bucket, and the dense per-bucket index the topology sampler reads. The
/// SQL codecs live on the domain types (deuce/kicker `sql` features), so no
/// primitive conversions leak out of this file.
pub struct Mapping {
    obs: Isomorphism,
    abs: Abstraction,
    position: i32,
}

impl From<(Isomorphism, Abstraction, i32)> for Mapping {
    fn from((obs, abs, position): (Isomorphism, Abstraction, i32)) -> Self {
        Self { obs, abs, position }
    }
}

#[async_trait::async_trait]
impl daybook::Row for Mapping {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer
            .write(&[&self.obs, &self.abs, &self.position])
            .await
            .expect("write");
    }
}

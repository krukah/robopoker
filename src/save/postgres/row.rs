use std::pin::Pin;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// A row that can write itself to a pinned BinaryCopyInWriter.
/// Each implementation handles its own arity.
#[async_trait::async_trait]
pub trait Row: Send {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>);
}

/// Lookup: (obs, abs) -> isomorphism table
#[async_trait::async_trait]
impl Row for (i64, i16) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer //   (obs,     abs)
            .write(&[&self.0, &self.1])
            .await
            .expect("write");
    }
}

/// Metric: (tri, dx) -> metric table
#[async_trait::async_trait]
impl Row for (i32, f32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer //   (tri,     dx)
            .write(&[&self.0, &self.1])
            .await
            .expect("write");
    }
}

/// Future: (prev, next, dx) -> transitions table
#[async_trait::async_trait]
impl Row for (i16, i16, f32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer //   (prev,    next,    dx)
            .write(&[&self.0, &self.1, &self.2])
            .await
            .expect("write");
    }
}

/// NlheProfile: (past, present, future, edge, policy, regret) -> blueprint table
#[async_trait::async_trait]
impl Row for (i64, i16, i64, i64, f32, f32) {
    async fn write(self, writer: Pin<&mut BinaryCopyInWriter>) {
        writer //   (past,    present, future,  edge,    policy,  regret)
            .write(&[&self.0, &self.1, &self.2, &self.3, &self.4, &self.5])
            .await
            .expect("write");
    }
}

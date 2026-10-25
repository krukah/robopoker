use candle_core::Device;

/// Where a [`CfrNet`](crate::CfrNet) runs: every parameter and tensor lives on `device`.
pub trait Backend: Send + Sync {
    fn device(&self) -> &Device;
}

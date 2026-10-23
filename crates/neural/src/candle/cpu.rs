use crate::*;
use candle_core::Device;

#[derive(Debug, Clone)]
pub struct CPU(Device);

impl Backend for CPU {
    fn device(&self) -> &Device {
        &self.0
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self(Device::Cpu)
    }
}

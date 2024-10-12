use krnl::{
    anyhow::Result,
    buffer::{Buffer, Slice, SliceMut},
    device::Device,
    macros::module,
};

#[module]
mod kernels {
    use cards::{evaluator::Evaluator, hand::Hand, rank::Rank, strength::Strength};
    #[cfg(not(target_arch = "spirv"))]
    use krnl::krnl_core;
    use krnl_core::macros::kernel;


    pub fn saxpy_impl(alpha: f32, x: f32, y: &mut f32) {
        // Currently rust-gpu fails to compile Enums<T> 
        Strength::from(Evaluator::from(Hand::from(1000)));

        *y += alpha * x;
    }

    // Item kernels for iterator patterns.
    #[kernel]
    pub fn saxpy(alpha: f32, #[item] x: f32, #[item] y: &mut f32) {
        saxpy_impl(alpha, x, y);
    }
}

fn saxpy(alpha: f32, x: Slice<f32>, mut y: SliceMut<f32>) -> Result<()> {
    if let Some((x, y)) = x.as_host_slice().zip(y.as_host_slice_mut()) {
        x.iter()
            .copied()
            .zip(y.iter_mut())
            .for_each(|(x, y)| kernels::saxpy_impl(alpha, x, y));
        return Ok(());
    }
    
    kernels::saxpy::builder()?
        .build(y.device())?
        .dispatch(alpha, x, y)
}

fn main() -> Result<()> {
    let x = vec![1f32, 1f32, 1f32];
    let alpha = 2f32;
    let y = vec![0f32, 0f32, 0f32];
    let device = Device::builder().build().ok().unwrap_or(Device::host());
    let x = Buffer::from(x).into_device(device.clone())?;
    let mut y = Buffer::from(y).into_device(device.clone())?;
    saxpy(alpha, x.as_slice(), y.as_slice_mut())?;
    let y = y.into_vec()?;
    println!("{y:?}");
    Ok(())
}

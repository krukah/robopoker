//! Deep CFR (Brown, Lerer, Gross & Sandholm, NeurIPS 2019, arXiv:1811.00164) over [`mccfr`]:
//! function approximators in place of the regret and strategy tables. See `README.md`.

mod batch;
mod candle;
mod cfr;
mod embedding;
mod hyperparams;
mod memo;
mod memory;
mod network;
mod profile;
mod solver;

pub use batch::*;
pub use candle::*;
pub use cfr::*;
pub use embedding::*;
pub use hyperparams::*;
pub use memo::*;
pub use memory::*;
pub use network::*;
pub use profile::*;
pub use solver::*;

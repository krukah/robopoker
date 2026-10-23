use crate::*;

/// Deep CFR on NLHE: [`FullInfo`] through [`NlheEmbedder`] into candle networks, sampled as the
/// paper does.
pub type FullSolver = neural::DeepCfr<FullEncoder, neural::CfrNet<NlheEmbedder>, mccfr::ExternalSampling>;

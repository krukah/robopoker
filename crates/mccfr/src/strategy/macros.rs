/// Generates a profile + solver pair for a CFR game.
///
/// From prefix `P` this generates:
/// - `PProfile` — struct with `CfrData` (epochs, encounters, metrics)
/// - `P<R, W, S>` — solver struct wrapping `PProfile` + encoder
#[macro_export]
macro_rules! mccfr {
    ($Prefix:ident, $CfrEncoder:ty, $T:ty, $E:ty, $G:ty, $I:ty, $batch:expr) => {
        paste::paste! {
            // ── profile struct ────────────────────────────────────────────────
            pub struct [<$Prefix Profile>] {
                pub(crate) epochs:     usize,
                pub(crate) encounters: std::collections::HashMap<
                    $I,
                    std::collections::HashMap<$E, $crate::Encounter>,
                >,
                pub(crate) metrics:    $crate::Metrics,
            }
            impl Default for [<$Prefix Profile>] {
                fn default() -> Self {
                    Self {
                        epochs:     0,
                        encounters: std::collections::HashMap::new(),
                        metrics:    $crate::Metrics::default(),
                    }
                }
            }
            impl $crate::CfrData for [<$Prefix Profile>] {
                type T = $T;
                type E = $E;
                type G = $G;
                type I = $I;
                fn encounters_ref(&self) -> &std::collections::HashMap<$I, std::collections::HashMap<$E, $crate::Encounter>> {
                    &self.encounters
                }
                fn encounters_mut(&mut self) -> &mut std::collections::HashMap<$I, std::collections::HashMap<$E, $crate::Encounter>> {
                    &mut self.encounters
                }
                fn epochs_ref(&self) -> usize {
                    self.epochs
                }
                fn epochs_mut(&mut self) -> &mut usize {
                    &mut self.epochs
                }
                fn store_metrics(&self) -> Option<&$crate::Metrics> {
                    Some(&self.metrics)
                }
            }
            // ── solver struct ─────────────────────────────────────────────────
            pub struct $Prefix<R, W, S>
            where
                R: $crate::RegretSchedule,
                W: $crate::WeightSchedule,
                S: $crate::SamplingScheme,
            {
                pub(crate) profile: [<$Prefix Profile>],
                pub(crate) encoder: $CfrEncoder,
                pub(crate) phantom: std::marker::PhantomData<fn() -> (R, W, S)>,
            }
            impl<R, W, S> Default for $Prefix<R, W, S>
            where
                R: $crate::RegretSchedule,
                W: $crate::WeightSchedule,
                S: $crate::SamplingScheme,
            {
                fn default() -> Self {
                    Self {
                        profile: Default::default(),
                        encoder: Default::default(),
                        phantom: std::marker::PhantomData,
                    }
                }
            }
            impl<R, W, S> $Prefix<R, W, S>
            where
                R: $crate::RegretSchedule,
                W: $crate::WeightSchedule,
                S: $crate::SamplingScheme,
            {
                pub fn new(profile: [<$Prefix Profile>], encoder: $CfrEncoder) -> Self {
                    Self {
                        profile,
                        encoder,
                        phantom: std::marker::PhantomData,
                    }
                }
            }
            impl<R, W, S> $crate::CfrEncoder for $Prefix<R, W, S>
            where
                R: $crate::RegretSchedule,
                W: $crate::WeightSchedule,
                S: $crate::SamplingScheme,
            {
                type T = $T;
                type E = $E;
                type G = $G;
                type I = $I;
                const CHECK_RECALL: bool = <$CfrEncoder as $crate::CfrEncoder>::CHECK_RECALL;
                fn seed(&self, game: &Self::G) -> Self::I {
                    self.encoder.seed(game)
                }
                fn info(
                    &self,
                    tree: &$crate::Tree<Self::T, Self::E, Self::G, Self::I>,
                    leaf: $crate::Leaf<Self::E, Self::G>,
                ) -> Self::I {
                    self.encoder.info(tree, leaf)
                }
                fn resume<P>(&self, past: P, game: &Self::G) -> Self::I
                where
                    P: IntoIterator<Item = Self::E>,
                {
                    self.encoder.resume(past, game)
                }
            }
            impl<R, W, S> $crate::Solver for $Prefix<R, W, S>
            where
                R: $crate::RegretSchedule,
                W: $crate::WeightSchedule,
                S: $crate::SamplingScheme,
            {
                type T = $T;
                type E = $E;
                type G = $G;
                type I = $I;
                type X = <$I as $crate::CfrInfo>::X;
                type Y = <$I as $crate::CfrInfo>::Y;
                type P = [<$Prefix Profile>];
                type N = $CfrEncoder;
                type R = R;
                type W = W;
                type S = S;
                fn batch_size() -> usize {
                    $batch
                }
                fn encoder(&self) -> &Self::N {
                    &self.encoder
                }
                fn profile(&self) -> &Self::P {
                    &self.profile
                }
                fn storage(&mut self) -> &mut Self::P {
                    &mut self.profile
                }
                fn advance(&mut self) {
                    $crate::CfrSampling::increment(&mut self.profile)
                }
            }
        }
    };
}

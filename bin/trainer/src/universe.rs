use clap::Args;
use pokerkit::*;

/// Universe selection for a public build; every hyperparameter keeps its default.
#[derive(Args, Clone, Copy, Debug)]
pub(crate) struct Universe {
    #[arg(long, value_enum)]
    regime: Option<Regime>,
    #[arg(long, value_enum)]
    version: Option<Version>,
}

impl Universe {
    /// Must run inside a tokio runtime: the signal handlers spawn onto it.
    pub(crate) fn boot(&self) {
        tracing_subscriber::fmt::init();
        if let Some(regime) = self.regime {
            Regime::init_regime(regime);
        }
        if let Some(version) = self.version {
            Version::init_version(version);
        }
        Signals::kys();
        Signals::brb();
    }
}

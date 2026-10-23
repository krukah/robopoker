use super::*;

#[derive(Parser, Debug)]
#[command(name = "trainer")]
#[command(group = ArgGroup::new("mode").required(true).args(["status", "cluster", "recluster", "fast", "slow", "reset", "forget", "deep"]))]
pub(crate) struct Cli {
    #[arg(long)]
    status: bool,
    #[arg(long)]
    cluster: bool,
    #[arg(long)]
    recluster: bool,
    #[arg(long)]
    fast: bool,
    #[arg(long)]
    slow: bool,
    #[arg(long)]
    reset: bool,
    #[arg(long)]
    forget: bool,
    /// Deep CFR on NLHE; saves the strategy net into DIR.
    #[arg(long, value_name = "DIR", num_args = 0..=1, default_missing_value = "nets")]
    deep: Option<std::path::PathBuf>,
    #[command(flatten)]
    universe: Universe,
}

impl Cli {
    pub(crate) fn universe(&self) -> &Universe {
        &self.universe
    }

    pub(crate) fn mode(&self) -> forge::Mode {
        [
            (self.fast(), forge::Mode::Fast),
            (self.slow(), forge::Mode::Slow),
            (self.cluster(), forge::Mode::Cluster),
            (self.recluster(), forge::Mode::Recluster),
            (self.reset(), forge::Mode::Reset),
            (self.forget(), forge::Mode::Forget),
            (self.status(), forge::Mode::Status),
            (self.deep().is_some(), forge::Mode::Deep(self.deep().cloned().unwrap_or_default())),
        ]
        .into_iter()
        .find_map(|(flag, mode)| flag.then_some(mode))
        .expect("clap group requires exactly one mode flag")
    }

    fn status(&self) -> bool {
        self.status
    }

    fn cluster(&self) -> bool {
        self.cluster
    }

    fn recluster(&self) -> bool {
        self.recluster
    }

    fn fast(&self) -> bool {
        self.fast
    }

    fn slow(&self) -> bool {
        self.slow
    }

    fn reset(&self) -> bool {
        self.reset
    }

    fn forget(&self) -> bool {
        self.forget
    }

    fn deep(&self) -> Option<&std::path::PathBuf> {
        self.deep.as_ref()
    }
}

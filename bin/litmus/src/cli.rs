use super::*;

#[derive(Parser, Debug)]
#[command(name = "litmus", about = "Static blueprint validation runner.")]
pub(crate) struct Cli {
    /// Path to scenarios.json.
    #[arg(long, default_value = "bin/litmus/scenarios.json")]
    pub(crate) scenarios: PathBuf,

    /// Where to write the markdown report. If omitted, prints to stdout.
    #[arg(long)]
    pub(crate) out: Option<PathBuf>,

    /// Pass `--version v1 --regime pluribus` to query the v1 blueprint table; defaults match the trainer.
    #[command(flatten)]
    pub(crate) universe: Universe,
}

impl Cli {
    pub(crate) fn scenarios(&self) -> &PathBuf {
        &self.scenarios
    }

    pub(crate) fn out(&self) -> Option<&PathBuf> {
        self.out.as_ref()
    }

    pub(crate) fn universe(&self) -> &Universe {
        &self.universe
    }
}

use super::*;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "litmus-audit", about = "Static action-grid audit of scenarios.json.")]
pub(crate) struct Cli {
    #[arg(long, default_value = "bin/litmus/scenarios.json")]
    pub(crate) scenarios: PathBuf,
    #[command(flatten)]
    pub(crate) universe: Universe,
}

impl Cli {
    pub(crate) fn scenarios(&self) -> &PathBuf {
        &self.scenarios
    }

    pub(crate) fn universe(&self) -> &Universe {
        &self.universe
    }
}

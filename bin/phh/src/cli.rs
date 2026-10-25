use super::*;

#[derive(Parser, Debug)]
#[command(name = "phh-cli", about = "Evaluate the blueprint against published hand histories.")]
pub(crate) struct Cli {
    /// Directory of `.phh` files, searched recursively.
    #[arg(long, default_value = DATA, env = "PHH_DATA", global = true)]
    pub(crate) data: std::path::PathBuf,

    #[command(subcommand)]
    pub(crate) mode: Mode,
}

impl Cli {
    pub(crate) fn data(&self) -> &std::path::Path {
        &self.data
    }

    pub(crate) fn mode(&self) -> &Mode {
        &self.mode
    }
}

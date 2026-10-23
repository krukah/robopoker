use super::*;

#[derive(Parser, Debug)]
#[command(name = "backend")]
pub(crate) struct Cli {
    #[command(flatten)]
    universe: Universe,
}

impl Cli {
    pub(crate) fn universe(&self) -> &Universe {
        &self.universe
    }
}

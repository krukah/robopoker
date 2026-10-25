use super::*;

#[derive(Parser, Debug)]
#[command(name = "slumbot")]
pub(crate) struct Cli {
    #[arg(long, default_value = "")]
    pub(crate) variants: String,
    #[arg(long, default_value_t = 1000)]
    pub(crate) hands: usize,
    #[arg(long)]
    pub(crate) continuous: bool,
    #[arg(long, default_value_t = 3)]
    pub(crate) throttle: usize,
    #[arg(long, default_value_t = 1)]
    pub(crate) sessions: usize,
    #[command(flatten)]
    pub(crate) universe: Universe,
}

impl Cli {
    pub(crate) fn variants(&self) -> &str {
        &self.variants
    }

    pub(crate) fn hands(&self) -> usize {
        self.hands
    }

    pub(crate) fn continuous(&self) -> bool {
        self.continuous
    }

    pub(crate) fn throttle(&self) -> usize {
        self.throttle
    }

    pub(crate) fn sessions(&self) -> usize {
        self.sessions
    }

    pub(crate) fn universe(&self) -> &Universe {
        &self.universe
    }
}

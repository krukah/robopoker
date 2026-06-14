//! Autotrain Binary
//!
//! Unified training pipeline with postgres as source of truth.
//!
//! Mode (exactly one of):
//!   --status, --fast, --slow, --cluster, --reset, --forget

use clap::ArgGroup;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "trainer")]
#[command(group = ArgGroup::new("mode").required(true).args(["status", "cluster", "fast", "slow", "reset", "forget"]))]
struct Cli {
    #[arg(long)]
    status: bool,
    #[arg(long)]
    cluster: bool,
    #[arg(long)]
    fast: bool,
    #[arg(long)]
    slow: bool,
    #[arg(long)]
    reset: bool,
    #[arg(long)]
    forget: bool,
}

impl Cli {
    fn mode(&self) -> forge::Mode {
        if self.fast {
            forge::Mode::Fast
        } else if self.slow {
            forge::Mode::Slow
        } else if self.cluster {
            forge::Mode::Cluster
        } else if self.reset {
            forge::Mode::Reset
        } else if self.forget {
            forge::Mode::Forget
        } else if self.status {
            forge::Mode::Status
        } else {
            unreachable!("clap group requires exactly one mode flag")
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    cli.mode().run().await;
}

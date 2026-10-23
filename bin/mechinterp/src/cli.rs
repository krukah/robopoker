use super::*;

#[derive(Parser, Debug)]
#[command(
    name = "mechinterp",
    about = "Name and describe every abstraction bucket from a sample of its members."
)]
pub(crate) struct Cli {
    /// Members sampled per bucket. A hundred pins every frequency to a few points.
    #[arg(long, default_value_t = 128)]
    pub(crate) samples: usize,
    /// Streets to interpret: any of pref, flop, turn, rive — or all.
    #[arg(long, value_enum, value_delimiter = ',', default_value = "flop,turn")]
    pub(crate) streets: Vec<Scope>,
    /// Print the glossary without writing it.
    #[arg(long)]
    pub(crate) dry: bool,
    /// Print each description, not just each name.
    #[arg(long)]
    pub(crate) long: bool,
}

impl Cli {
    pub(crate) fn streets(&self) -> Vec<Street> {
        self.scopes().iter().copied().flat_map(Scope::streets).collect()
    }

    pub(crate) fn samples(&self) -> usize {
        self.samples
    }

    pub(crate) fn dry(&self) -> bool {
        self.dry
    }

    pub(crate) fn long(&self) -> bool {
        self.long
    }

    fn scopes(&self) -> &[Scope] {
        &self.streets
    }
}

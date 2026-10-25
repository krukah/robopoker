use super::*;

/// What a `--streets` value names. A plain `Vec<String>` matched against
/// `Street::try_from` swallowed typos: `--streets xyz` parsed to nothing and
/// the run silently interpreted no buckets at all.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum Scope {
    #[value(alias = "preflop")]
    Pref,
    Flop,
    Turn,
    #[value(alias = "river")]
    Rive,
    All,
}

impl Scope {
    pub(crate) fn streets(self) -> Vec<Street> {
        match self {
            Self::All => Street::all().to_vec(),
            Self::Pref => vec![Street::Pref],
            Self::Flop => vec![Street::Flop],
            Self::Turn => vec![Street::Turn],
            Self::Rive => vec![Street::Rive],
        }
    }
}

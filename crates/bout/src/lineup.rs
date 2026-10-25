//! The `--variants` argument, parsed.

use pokerkit::Variant;

/// Which bots play, and how many sessions each one gets.
///
/// The session count is the knob that keeps a benchmark container API-bound rather than CPU-
/// bound: a blueprint lookup returns instantly and wants one session, while a subgame-solving
/// variant spends most of a decision in CFR and wants as many sessions as there are cores.
pub struct Lineup(Vec<(Variant, usize)>);

impl Lineup {
    /// Parse a comma-separated variant list, deduping by label. Prints usage
    /// and exits on an unknown token, an empty list, or a malformed `*N` —
    /// this runs at binary startup, where failing fast beats a container that
    /// burns an hour playing the wrong bot.
    pub fn parse(raw: &str, default: usize) -> Self {
        let mut lineup = Vec::<(Variant, usize)>::new();
        for token in raw.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            let (token, sessions) = Self::split(token);
            match Variant::parse(token) {
                None => Self::usage(&format!("unknown variant: {token}")),
                Some(v) if lineup.iter().any(|(seen, _)| seen.label() == v.label()) => {}
                Some(v) => lineup.push((v, sessions.unwrap_or(default).max(1))),
            }
        }
        if lineup.is_empty() {
            Self::usage("no variants given");
        }
        Self(lineup)
    }

    /// One entry per session to spawn, paired with its index within the
    /// variant so log spans can tell concurrent sessions apart.
    pub fn sessions(&self) -> impl Iterator<Item = (Variant, usize)> + '_ {
        self.0.iter().flat_map(|&(v, n)| (0..n).map(move |i| (v, i)))
    }

    /// Whether any variant needs the blueprint hydrated into memory — `fish`
    /// alone doesn't, and hydration is minutes of startup.
    pub fn hydrates(&self) -> bool {
        self.0.iter().any(|(v, _)| v.requires_blueprint())
    }

    /// Split a trailing `*N` session override off a variant token.
    fn split(token: &str) -> (&str, Option<usize>) {
        match token.rsplit_once('*') {
            None => (token, None),
            Some((variant, count)) => match count.parse::<usize>() {
                Ok(n) if n >= 1 => (variant, Some(n)),
                _ => Self::usage(&format!("invalid session count (expected `*N` with N>=1): {token}")),
            },
        }
    }

    /// Print the variant grammar and exit.
    fn usage(complaint: &str) -> ! {
        eprintln!("{complaint}");
        eprintln!("usage: --variants=a,b,c [--hands N] [--continuous] [--throttle N] [--sessions N]");
        eprintln!("       grammar: 8 hypercube cells + `fish`");
        eprintln!("         fish | base | depth | world | dirac |");
        eprintln!("         depth+world | depth+dirac | world+dirac | depth+world+dirac");
        eprintln!("       (`+`-joined flags from {{depth, world, dirac}} in canonical order;");
        eprintln!("        `base` is the sentinel name for the empty flag-set cell)");
        eprintln!("       per-variant session override: trailing `*N` on a token");
        eprintln!("         example: base*1,dirac*1,depth+dirac*4,depth+world*4");
        eprintln!("       --sessions: default session count when no `*N` suffix is given (default 1).");
        eprintln!("                   set to task vCPU count to saturate CPU during CFR think.");
        std::process::exit(1);
    }
}

impl std::fmt::Display for Lineup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|(v, n)| format!("{}*{}", v.label(), n))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

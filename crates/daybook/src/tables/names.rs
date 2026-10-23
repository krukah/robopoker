//! Table-name helpers. A name depends on the active regime and version,
//! so each is a function behind a `OnceLock` rather than a constant.

use crate::*;
use std::sync::OnceLock;

/// Shared table: name is always the literal default.
macro_rules! table {
    ($name:ident, $default:expr, $doc:expr) => {
        #[doc = $doc]
        pub fn $name() -> &'static str {
            static T: OnceLock<&str> = OnceLock::<&str>::new();
            *T.get_or_init(|| $default)
        }
    };
}
/// Clustering-derived table: name gets the suffix of the version whose K-means output this
/// version *uses*. A version that only changes the bet-sizing grid reads the existing
/// clustering tables of the version it derives from rather than requiring a fresh clustering
/// run. With a single live `V1`, this resolves to the `_v1` clustering tables. See
/// [`pokerkit::Version::clustering_suffix`].
macro_rules! versioned {
    ($name:ident, $default:expr, $doc:expr) => {
        #[doc = $doc]
        pub fn $name() -> &'static str {
            static T: OnceLock<&str> = OnceLock::<&str>::new();
            *T.get_or_init(|| Pool::leaked(format!("{}{}", $default, pokerkit::Version::current().clustering_suffix())))
        }
    };
}
/// Training-derived table: name gets the regime suffix (bet sizing) plus the version suffix
/// (abstraction family). Every table that depends on a trained strategy is keyed by InfoIds
/// that embed abstraction IDs from the active version, so regime-only suffixing would let two
/// versions corrupt each other's strategy data.
macro_rules! regime {
    ($name:ident, $default:expr, $doc:expr) => {
        #[doc = $doc]
        pub fn $name() -> &'static str {
            static T: OnceLock<&str> = OnceLock::<&str>::new();
            *T.get_or_init(|| {
                Pool::leaked(format!(
                    "{}{}{}{}",
                    $default,
                    pokerkit::Signals::players_suffix(),
                    pokerkit::Regime::current().suffix(),
                    pokerkit::Version::current().suffix(),
                ))
            })
        }
    };
}

// Shared tables (game-level / auth — invariant under V × R).
table!(actions, "actions", "Table for game actions (bets, raises, folds, etc.).");
table!(hands, "hands", "Table for completed poker hands.");
table!(players, "players", "Table for player participation in hands.");
table!(rooms, "rooms", "Table for active game rooms.");
table!(sessions, "sessions", "Table for user authentication sessions.");
table!(users, "users", "Table for registered user accounts and identity.");

// Versioned tables (abstraction-derived — depend on K-means params).
versioned!(abstraction, "abstraction", "Table for abstraction bucket definitions.");
versioned!(isomorphism, "isomorphism", "Table for isomorphism → abstraction mappings.");
versioned!(street, "street", "Table for street-specific metadata.");
versioned!(transitions, "transitions", "Table for abstraction transition probabilities.");
versioned!(
    mechinterp,
    "mechinterp",
    "Table for human-readable bucket glosses (name + description). Versioned \
     because a bucket id only means something inside the clustering that \
     produced it — see the `rosetta` crate."
);
versioned!(
    metric,
    "metric",
    "Table for pairwise abstraction distances. Versioned because EMD \
     between buckets is meaningless across abstractions with different K."
);

// Regime x Version tables (training-derived — K-means x bet sizing).
regime!(
    blueprint,
    "blueprint",
    "Table for MCCFR blueprint strategies (policy + regret). Keyed by \
     (Edge, InfoId) where InfoId embeds abstraction IDs from the active \
     version, hence the version suffix in addition to regime."
);

regime!(epoch, "epoch", "Table for training epoch metadata and progress.");
regime!(snapshot, "snapshot", "Table for training snapshot statistics (append-only).");
regime!(
    staging,
    "staging",
    "Buffer for COPY-IN during periodic blueprint flushes. Suffixed so \
     concurrent training runs at different (regime, version) tuples \
     don't corrupt each other's bulk-write buffers."
);
regime!(
    fingerprint,
    "fingerprint",
    "Single-row table storing a textual fingerprint of regime-affecting \
     constants at the time the blueprint was first written. Trainer \
     panics on startup if the live fingerprint differs from the stored \
     one — guards against silent drift in bet-sizing / stack constants \
     that share Edge serialization but change action semantics. Run \
     `--mode reset` to clear and re-fingerprint."
);

// Scoreboard tables (cross-fingerprint measurements). Deliberately `table!`, NOT `regime!`.
table!(benchmark, "benchmark", "Slumbot bb/100 scores per (fingerprint × variant). Append-only.");
table!(litmus, "litmus", "Litmus structural pass/fail counts per fingerprint. Append-only.");

//! Database pipeline for training artifacts.
//!
//! Bulk data movement between Rust structures and PostgreSQL, optimized for
//! the large-scale writes required during abstraction and blueprint training.
//!
//! ## Connectivity
//!
//! - [`db()`] — Establishes a database connection from `DB_URL`
//!
//! ## Serialization Traits
//!
//! - [`Schema`] — Table metadata and DDL generation
//! - [`Derive`] — INSERT statement generation for enumerable types
//! - [`Hydrate`] — Binary format decoding from rows
//! - [`Row`] — Binary row serialization for COPY protocol
//! - [`Streamable`] — Bulk data upload via COPY
//!
//! ## Core Types
//!
//! - [`Stage`] — Temporary staging table management
//! - [`Check`] — Schema validation and migration status
//!
//! ## Table Names
//!
//! Constants for all persistent entities: abstractions, blueprints,
//! metrics, hands, sessions, and more.
mod check;
mod measure;
mod schema;
mod stage;
mod traits;

pub use check::*;
pub use measure::measure;
// schema module provides trait impls, no items to re-export
pub use stage::*;
pub use traits::*;

use std::sync::Arc;
use tokio_postgres::Client;

/// Establishes a database connection.
///
/// Connects to PostgreSQL using the `DB_URL` environment variable.
/// Returns an `Arc<Client>` suitable for sharing across async tasks.
///
/// # Environment
///
/// Requires `DB_URL` to be set (e.g., `postgres://user:pass@host:port/db`).
///
/// # Panics
///
/// Panics if `DB_URL` is not set or if connection fails.
pub async fn db() -> Arc<Client> {
    tracing::info!("connecting to database");
    let tls = tokio_postgres::tls::NoTls;
    let ref url = std::env::var("DB_URL").expect("DB_URL must be set");
    let (client, connection) = tokio_postgres::connect(url, tls)
        .await
        .expect("database connection failed");
    tokio::spawn(async move {
        connection
            .await
            .inspect_err(|e| tracing::error!(error = %e, "database connection lost"))
            .ok();
    });
    client
        .execute("SET client_min_messages TO WARNING", &[])
        .await
        .expect("set client_min_messages");
    Arc::new(client)
}

/// PostgreSQL error type alias.
pub type PgErr = tokio_postgres::Error;

use std::sync::OnceLock;

/// Leaks a formatted string to obtain a `&'static str`.
/// Used once per table name at process startup.
pub fn leaked(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

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
/// Clustering-derived table: name gets the suffix of the version whose
/// K-means output this version *uses*. When a new `Version` only changes
/// the bet-sizing grid (e.g. V3 reuses V1's clustering), it should read
/// the existing `_v1` clustering tables — not require fresh clustering
/// under a `_v3` suffix. See [`pokerkit::Version::clustering_suffix`].
macro_rules! versioned {
    ($name:ident, $default:expr, $doc:expr) => {
        #[doc = $doc]
        pub fn $name() -> &'static str {
            static T: OnceLock<&str> = OnceLock::<&str>::new();
            *T.get_or_init(|| leaked(format!("{}{}", $default, pokerkit::version().clustering_suffix())))
        }
    };
}
/// Training-derived table: name gets the regime suffix (bet sizing)
/// plus the version suffix (abstraction family). Every table that
/// depends on a trained strategy is keyed by InfoIds that embed
/// abstraction IDs from the active version, so regime-only suffixing
/// would let two versions corrupt each other's strategy data.
///
/// Regime suffix comes first; V0's empty version suffix preserves
/// existing `<base>_<regime>` table names.
macro_rules! regime {
    ($name:ident, $default:expr, $doc:expr) => {
        #[doc = $doc]
        pub fn $name() -> &'static str {
            static T: OnceLock<&str> = OnceLock::<&str>::new();
            *T.get_or_init(|| {
                leaked(format!("{}{}{}", $default, pokerkit::regime().suffix(), pokerkit::version().suffix(),))
            })
        }
    };
}

// ── Shared tables (game-level / auth — invariant under V × R) ───────────────
table!(actions, "actions", "Table for game actions (bets, raises, folds, etc.).");
table!(hands, "hands", "Table for completed poker hands.");
table!(players, "players", "Table for player participation in hands.");
table!(rooms, "rooms", "Table for active game rooms.");
table!(sessions, "sessions", "Table for user authentication sessions.");
table!(users, "users", "Table for registered user accounts and identity.");

// ── Versioned tables (abstraction-derived — depend on K-means params) ───────
versioned!(abstraction, "abstraction", "Table for abstraction bucket definitions.");
versioned!(isomorphism, "isomorphism", "Table for isomorphism → abstraction mappings.");
versioned!(street, "street", "Table for street-specific metadata.");
versioned!(transitions, "transitions", "Table for abstraction transition probabilities.");
versioned!(
    metric,
    "metric",
    "Table for pairwise abstraction distances. Versioned because EMD \
     between buckets is meaningless across abstractions with different K."
);

// ── Regime × Version tables (training-derived — depend on K-means × bet sizing)
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

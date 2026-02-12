//! PostgreSQL integration and binary format serialization.
//!
//! Low-level database connectivity and efficient binary encoding for
//! high-throughput data transfer. Uses PostgreSQL's native binary format
//! to minimize serialization overhead.
//!
//! ## Connectivity
//!
//! - [`db()`] — Establishes a database connection from `DB_URL`
//!
//! ## Serialization
//!
//! - [`Schema`] — Table metadata and DDL generation
//! - [`Derive`] — INSERT statement generation for enumerable types
//! - [`Hydrate`] — Binary format decoding from rows
//! - [`Row`] — Binary row serialization for COPY protocol
//! - [`Streamable`] — Bulk data upload via COPY
//!
//! ## Table Names
//!
//! Constants for all persistent entities: abstractions, blueprints,
//! metrics, hands, sessions, and more.
mod derive;
mod hydrate;
mod row;
mod schema;
mod stream;

pub use derive::*;
pub use hydrate::*;
pub use row::*;
pub use schema::*;
pub use stream::*;

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
    log::info!("connecting to database");
    let tls = tokio_postgres::tls::NoTls;
    let ref url = std::env::var("DB_URL").expect("DB_URL must be set");
    let (client, connection) = tokio_postgres::connect(url, tls)
        .await
        .expect("database connection failed");
    tokio::spawn(connection);
    client
        .execute("SET client_min_messages TO WARNING", &[])
        .await
        .expect("set client_min_messages");
    Arc::new(client)
}

/// PostgreSQL error type alias.
pub type PgErr = tokio_postgres::Error;

/// Table for abstraction bucket definitions.
#[rustfmt::skip]
pub const ABSTRACTION: &str = "abstraction";
/// Table for game actions (bets, raises, folds, etc.).
#[rustfmt::skip]
pub const ACTIONS:     &str = "actions";
/// Table for MCCFR blueprint strategies (policy + regret).
#[rustfmt::skip]
pub const BLUEPRINT:   &str = "blueprint";
/// Table for training epoch metadata and progress.
#[rustfmt::skip]
pub const EPOCH:       &str = "epoch";
/// Table for completed poker hands.
#[rustfmt::skip]
pub const HANDS:       &str = "hands";
/// Table for isomorphism → abstraction mappings.
#[rustfmt::skip]
pub const ISOMORPHISM: &str = "isomorphism";
/// Table for pairwise abstraction distances.
#[rustfmt::skip]
pub const METRIC:      &str = "metric";
/// Table for player participation in hands.
#[rustfmt::skip]
pub const PLAYERS:     &str = "players";
/// Table for active game rooms.
#[rustfmt::skip]
pub const ROOMS:       &str = "rooms";
/// Table for user authentication sessions.
#[rustfmt::skip]
pub const SESSIONS:    &str = "sessions";
/// Table for staging data during bulk operations.
#[rustfmt::skip]
pub const STAGING:     &str = "staging";
/// Table for street-specific metadata.
#[rustfmt::skip]
pub const STREET:      &str = "street";
/// Table for abstraction transition probabilities.
#[rustfmt::skip]
pub const TRANSITIONS: &str = "transitions";
/// Table for registered user accounts.
#[rustfmt::skip]
pub const USERS:       &str = "users";

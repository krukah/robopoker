//! Player implementations for different game contexts.
//!
//! The bot zoo is built compositionally: every axis is a [`Brain`]
//! wrapper. [`Blueprint`] is the leaf (in-memory blueprint lookup);
//! [`Depth<B>`] / [`World<B>`] add subgame solver layers; [`Dirac<B>`]
//! sharpens the inner brain's distribution to a Dirac delta. Stack
//! them in canonical order (`Dirac<World<Depth<Blueprint>>>`) and
//! wrap with [`Agent<B>`] to get a `Player`. [`zoo`] is the single
//! runtime → comptime binding both slumbot and the hosting server use.
//!
//! Every helper that operates on a typed value lives as a method on that
//! type — picker bodies inside their own structs, telemetry on
//! [`Solved`]. The only free function in this module is the boundary
//! one: [`hydrate_blueprint`], which stitches the DB to the in-memory
//! [`Flagship`](holdem::Flagship) and is the single externally-visible
//! entry point.

/// Hydrate a single [`Flagship`](holdem::Flagship) blueprint from the
/// database and leak it as a `'static` reference. Wrap with the
/// composition you want via [`Mount::mount`] (or call [`zoo`]).
///
/// Hydration is the only DB-bound path; every Brain impl reads from the
/// in-memory blueprint, so per-decision DB roundtrips are gone. Call
/// once per process:
///
/// - **Backend server**: load once in `main()`, hand the `&'static` to
///   `Casino`; every bot shares it.
/// - **Slumbot one-container runner**: load once in
///   `spar::Runtime::run`; share across all spawned variant tasks.
#[cfg(feature = "server")]
pub async fn hydrate_blueprint(client: std::sync::Arc<tokio_postgres::Client>) -> &'static holdem::Flagship {
    use ledger::Hydrate;
    Box::leak(Box::new(holdem::Flagship::hydrate(client).await))
}

#[cfg(feature = "server")]
mod agent;
#[cfg(feature = "server")]
mod blueprint;
#[cfg(feature = "server")]
mod brain;
#[cfg(feature = "server")]
mod depth;
#[cfg(feature = "server")]
mod dirac;
mod fish;
#[cfg(feature = "cli")]
mod human;
#[cfg(feature = "server")]
mod mount;
#[cfg(feature = "server")]
mod solved;
#[cfg(feature = "server")]
mod variant;
#[cfg(feature = "server")]
mod world;
#[cfg(feature = "server")]
mod zoo;

#[cfg(feature = "server")]
pub use agent::*;
#[cfg(feature = "server")]
pub use blueprint::*;
#[cfg(feature = "server")]
pub use brain::*;
#[cfg(feature = "server")]
pub use depth::*;
#[cfg(feature = "server")]
pub use dirac::*;
pub use fish::*;
#[cfg(feature = "cli")]
pub use human::*;
#[cfg(feature = "server")]
pub use mount::*;
#[cfg(feature = "server")]
pub use solved::*;
#[cfg(feature = "server")]
pub use variant::*;
#[cfg(feature = "server")]
pub use world::*;
#[cfg(feature = "server")]
pub use zoo::*;

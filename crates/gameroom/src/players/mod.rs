//! Player implementations for different game contexts.
//!
//! The bot zoo is built compositionally: every axis is a [`Brain`]
//! wrapper. [`Blueprint`] is the leaf (in-memory blueprint lookup);
//! [`Depth<B>`] / [`World<B>`] add subgame solver layers; [`Dirac<B>`]
//! sharpens the inner brain's distribution to a Dirac delta. Stack
//! them in canonical order (`Dirac<World<Depth<Blueprint>>>`) and
//! wrap with [`Agent<B>`] to get a [`Player`]. [`zoo`] is the single
//! runtime → comptime binding both slumbot and the hosting server use.
//!
//! Every helper that operates on a typed value lives as a method on that
//! type — picker bodies inside their own structs, telemetry on
//! [`Solved`]. The only free function in this module is the boundary
//! one: [`hydrate_blueprint`], which stitches the DB to the in-memory
//! [`Flagship`](rbp_nlhe::Flagship) and is the single externally-visible
//! entry point.

/// Hydrate a single [`Flagship`](rbp_nlhe::Flagship) blueprint from the
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
///   `rbp_slumbot::Runtime::run`; share across all spawned variant tasks.
#[cfg(feature = "database")]
pub async fn hydrate_blueprint(
    client: std::sync::Arc<tokio_postgres::Client>,
) -> &'static rbp_nlhe::Flagship {
    use rbp_database::Hydrate;
    Box::leak(Box::new(rbp_nlhe::Flagship::hydrate(client).await))
}

#[cfg(feature = "database")]
mod agent;
#[cfg(feature = "database")]
mod blueprint;
#[cfg(feature = "database")]
mod brain;
#[cfg(feature = "database")]
mod depth;
#[cfg(feature = "database")]
mod dirac;
mod fish;
#[cfg(feature = "cli")]
mod human;
#[cfg(feature = "database")]
mod mount;
#[cfg(feature = "database")]
mod solved;
#[cfg(feature = "database")]
mod variant;
#[cfg(feature = "database")]
mod world;
#[cfg(feature = "database")]
mod zoo;

#[cfg(feature = "database")]
pub use agent::*;
#[cfg(feature = "database")]
pub use blueprint::*;
#[cfg(feature = "database")]
pub use brain::*;
#[cfg(feature = "database")]
pub use depth::*;
#[cfg(feature = "database")]
pub use dirac::*;
pub use fish::*;
#[cfg(feature = "cli")]
pub use human::*;
#[cfg(feature = "database")]
pub use mount::*;
#[cfg(feature = "database")]
pub use solved::*;
#[cfg(feature = "database")]
pub use variant::*;
#[cfg(feature = "database")]
pub use world::*;
#[cfg(feature = "database")]
pub use zoo::*;

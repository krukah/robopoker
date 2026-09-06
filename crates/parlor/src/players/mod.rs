//! Player implementations for different game contexts.
//!
//! The bot zoo is compositional: every axis is a [`Brain`] wrapper.
//! [`Blueprint`] is the leaf (in-memory lookup); [`Depth<B>`] / [`World<B>`]
//! add subgame solver layers; [`Dirac<B>`] sharpens the inner distribution to
//! a Dirac delta. Stack in canonical order (`Dirac<World<Depth<Blueprint>>>`)
//! and wrap with [`Agent<B>`] to get a `Player`. [`zoo`] is the single
//! runtime → comptime binding both slumbot and the hosting server use.
//!
//! Four dials, but **not** a free `2^4`: three are an orthogonal cube, the
//! fourth (nesting) is a *conditional* layer.
//!
//! ```mermaid
//! flowchart LR
//!   subgraph solve["one CFR re-solve"]
//!     W["world<br/>range (safe)"]
//!     D["depth<br/>leaf values"]
//!     N["nest<br/>+1 action at entry"]
//!   end
//!   W --> P["policy"]
//!   D --> P
//!   N --> P
//!   P --> K["dirac<br/>argmax (output layer)"] --> Out(["action"])
//! ```
//!
//! - **`Dirac`** is a pure *output* transform (argmax the final policy). Orthogonal
//!   to everything, including "no solve": `Dirac<Blueprint>` is valid. Factor it out
//!   — every row below also has a `Dirac<…>` twin.
//! - **`Depth` ⟂ `World`** — leaf evaluation vs opponent range; independently toggleable.
//! - **`Nest` is conditional.** Nesting is a re-solve *trigger + a menu edit*, not a
//!   solving method, so it needs a solve to attach to. It's driven by
//!   [`Translation::Exact`](pokerkit::Translation) at runtime (not a `Config` bool),
//!   and zoo wires it as a world-subgame re-solve: **nest ⟺ (`Exact` ∧ `world`)**,
//!   independent of the `depth` dial (which shapes only the on-tree fallback brain).
//!
//! Validity (× `Dirac` for each row):
//!
//! | depth | world | nest | meaning | status |
//! |:-:|:-:|:-:|---|---|
//! | – | – | – | blueprint (no re-solve) | ✓ wired |
//! | ✓ | – | – | depth-limited solve | ✓ `adapt_leaf` |
//! | – | ✓ | – | safe full-tree solve | ✓ `adapt_safe` |
//! | ✓ | ✓ | – | safe depth-limited solve | ✓ `adapt_full` |
//! | – | – | ✓ | nest, no solve to attach | ✗ incoherent (nothing to augment → = translation) |
//! | ✓ | – | ✓ | nest, range layer off | ✗ not built — zoo gates nesting on `world` |
//! | – | ✓ | ✓ | nest over safe fallback | ✓ `Nest<World<…>>` (nested solve = `adapt_nested`) |
//! | ✓ | ✓ | ✓ | nest over safe+depth fallback | ✓ `Nest<World<Depth<…>>>` (nested solve = `adapt_nested`) |
//!
//! **What the dials do under `nest`:** both wired nest cells run the *same* nested
//! re-solve — `adapt_nested` (safe + depth) — so the `depth`/`world` dials select
//! only the *on-tree fallback* brain `Nest<B>` delegates to when the line stays
//! on-grid, not the nested-solve method. A depth-less or unsafe *nested* variant
//! (`adapt_nested_leaf` / `adapt_nested_safe`) isn't wired; adding one lets
//! `Nest<B>` pick per dial. Deferred until the safe-vs-unsafe search question is
//! settled (`docs/active/pluribus-parity.md`).
//!
//! The *solve*-axis concepts (world/depth/nest orthogonality) are in
//! [`crates/subgame/README.md`](../../../subgame/README.md); this section is the
//! player-side view that also folds in `Dirac` and the zoo wiring.

/// Hydrate a [`Flagship`](nlhe::Flagship) blueprint from the database and leak
/// it as `'static`. Wrap with a composition via [`Mount::mount`] (or [`zoo`]).
///
/// The only DB-bound path — every Brain impl reads the in-memory blueprint, so
/// there are no per-decision roundtrips. Call once per process and share the
/// reference across all bots / spawned variant tasks.
#[cfg(feature = "server")]
pub async fn hydrate_blueprint(client: std::sync::Arc<tokio_postgres::Client>) -> &'static nlhe::Flagship {
    use daybook::Hydrate;
    Box::leak(Box::new(nlhe::Flagship::hydrate(client).await))
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
mod nest;
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
pub use nest::*;
#[cfg(feature = "server")]
pub use solved::*;
#[cfg(feature = "server")]
pub use variant::*;
#[cfg(feature = "server")]
pub use world::*;
#[cfg(feature = "server")]
pub use zoo::*;

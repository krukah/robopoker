#![doc = include_str!("doc/players.md")]

/// Helpers shared across the players module.
pub struct Players;

impl Players {
    /// Hydrate a [`Flagship`](nlhe::Flagship) blueprint from the database and leak
    /// it as `'static`. Wrap with a composition via [`crate::Mount::mount`] (or [`crate::Tag::zoo`]).
    ///
    /// The only DB-bound path — every Brain impl reads the in-memory blueprint, so
    /// there are no per-decision roundtrips. Call once per process and share the
    /// reference across all bots / spawned variant tasks.
    #[cfg(feature = "server")]
    pub async fn hydrate_blueprint(client: std::sync::Arc<tokio_postgres::Client>) -> &'static nlhe::Flagship {
        Box::leak(Box::new(nlhe::NlheBlueprint::solver(client).await))
    }
}

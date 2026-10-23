//! One container, every variant, one shared blueprint.
//!
//! A single process hydrates the blueprint and the database once, then spawns a tokio task per
//! (variant × session). All sessions share the postgres client, the `&'static Flagship` that
//! any subgame-solving cell reads, the [`Throttle`] capping aggregate in-flight requests, and
//! the SIGTERM / `TRAIN_DURATION` interrupt from [`pokerkit::Signals::brb`].

use crate::benchmark::*;
use crate::lineup::*;
use crate::mode::*;
use crate::recorder::*;
use crate::rival::*;
use crate::throttle::*;
use daybook::Check;
use daybook::Ensure;
use parlor::VariantExt;
use pokerkit::Variant;
use std::sync::Arc;
use tracing::Instrument;

/// A whole benchmark run against one external opponent.
pub struct Runtime<R>
where
    R: Rival,
{
    auth: R::Auth,
    lineup: Lineup,
    mode: Mode,
    inflight: usize,
}

impl<R> Runtime<R>
where
    R: Rival + 'static,
{
    /// Assemble from already-parsed CLI flags. `sessions` is the fallback
    /// session count for variant tokens with no `*N` suffix.
    pub fn new(
        auth: R::Auth,
        variants: &str,
        hands: usize,
        continuous: bool,
        inflight: usize,
        sessions: usize,
    ) -> Self {
        Self {
            auth,
            lineup: Lineup::parse(variants, sessions.max(1)),
            mode: Mode::new(hands, continuous),
            inflight,
        }
    }

    fn auth(&self) -> &R::Auth {
        &self.auth
    }

    fn lineup(&self) -> &Lineup {
        &self.lineup
    }

    fn mode(&self) -> Mode {
        self.mode
    }

    fn inflight(&self) -> usize {
        self.inflight
    }

    /// Hydrate once, then run every session to completion.
    pub async fn run(self) {
        tracing::info!(
            rival = R::ADVERSARY.service(),
            variants = %self.lineup(),
            mode = ?self.mode(),
            inflight = self.inflight(),
            "bout runtime starting",
        );
        let db = daybook::Pool::db().await;
        db.ensure::<daybook::Benchmark>().await;
        let epoch = db.epochs().await as i64;
        let flagship = if self.lineup().hydrates() {
            Some(parlor::Players::hydrate_blueprint(db.clone()).await)
        } else {
            None
        };
        let throttle = Throttle::new(self.inflight());
        let handles = self
            .lineup()
            .sessions()
            .map(|(variant, session)| {
                let auth = self.auth().clone();
                let db = db.clone();
                let throttle = throttle.clone();
                let mode = self.mode();
                tokio::spawn(
                    async move { Self::session(auth, variant, db, flagship, throttle, mode, epoch).await }
                        .instrument(tracing::info_span!("variant", name = variant.label(), session)),
                )
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle
                .await
                .inspect_err(|e| tracing::error!(error = %e, "variant task panicked"))
                .ok();
        }
    }

    /// One session: its own player, its own room, its own transport.
    async fn session(
        auth: R::Auth,
        variant: Variant,
        db: Arc<tokio_postgres::Client>,
        flagship: Option<&'static nlhe::Flagship>,
        throttle: Throttle,
        mode: Mode,
        epoch: i64,
    ) {
        let mut player = variant.into_player(flagship);
        let mut recorder = Recorder::new(db.clone(), variant.id(), R::ADVERSARY, R::STACK).await;
        let bench = Benchmark::run::<R>(&auth, variant, player.as_mut(), &mut recorder, throttle, mode).await;
        bench.report();
        bench.persist(db.as_ref(), epoch).await;
    }
}

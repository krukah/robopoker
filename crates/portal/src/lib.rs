//! Unified Backend Server
//!
//! ## Submodules
//!
//! - [`topology`]  — Abstraction exploration and clustering queries
//! - [`strategy`]  — Strategy lookups
//! - [`gameplay`]  — Hand history evaluation and AIVAT analysis
//! - [`hosting`]   — WebSocket game hosting infrastructure
//! - [`training`]  — MCCFR training observability

pub mod gameplay;
pub mod hosting;
pub mod litmus;
mod metrics;
pub mod strategy;
pub mod topology;
pub mod training;

pub use gameplay::GameplayAPI;
pub use hosting::Casino;
pub use hosting::RoomHandle;
pub use strategy::StrategyAPI;
pub use topology::CLI;
pub use topology::Query;
pub use topology::TopologyAPI;
pub use training::TrainingAPI;

use actix_cors::Cors;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::middleware::Logger;
use actix_web::web;
use std::sync::Arc;
use tokio_postgres::Client;

/// Ensures all tables exist. Single point of truth for schema creation.
async fn ensure_all(client: &Client) {
    use ledger::Ensure;
    client.ensure::<bouncer::Member>().await;
    client.ensure::<bouncer::Session>().await;
    client.ensure::<parlor::Room>().await;
    client.ensure::<parlor::records::Hand>().await;
    client.ensure::<parlor::Participant>().await;
    client.ensure::<parlor::Play>().await;
    client.ensure::<holdem::NlheProfile>().await;
    client.ensure::<forge::EpochMeta>().await;
    client.ensure::<forge::Snapshot>().await;
}

async fn health(client: web::Data<Arc<Client>>) -> impl Responder {
    match client
        .execute("SELECT 1", &[])
        .await
        .inspect_err(|e| tracing::error!("health check failed: {e}"))
    {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(_) => HttpResponse::ServiceUnavailable().body("database unavailable"),
    }
}

#[rustfmt::skip]
pub async fn run() -> Result<(), std::io::Error> {
    let client = ledger::db().await;
    ensure_all(&client).await;
    use parlor::VariantExt;
    let mut seedables: Vec<bouncer::Member> = pokerkit::Variant::all()
        .iter()
        .map(|v| v.member())
        .collect();
    seedables.push(parlor::slumbot_opponent());
    for member in &seedables {
        bouncer::AuthRepository::seed(&client, member)
            .await
            .unwrap_or_else(|e| tracing::warn!("failed to seed {}: {}", member.username(), e));
        tracing::info!("seeded bot user {}", member.username());
    }
    let blueprint: Option<&'static holdem::Flagship> = if std::env::var("SKIP_BLUEPRINT").ok().as_deref() == Some("1") {
        tracing::warn!("SKIP_BLUEPRINT=1: skipping blueprint hydration (only Fish opponents will work)");
        None
    } else {
        tracing::info!("loading blueprint into memory");
        Some(Box::leak(Box::new(ledger::Hydrate::hydrate(client.clone()).await)))
    };
    let topology = web::Data::new(topology::TopologyAPI::new(client.clone()));
    let strategy = web::Data::new(strategy::StrategyAPI::new(client.clone()).with_blueprint(blueprint));
    let gameplay = web::Data::new(gameplay::GameplayAPI::new(client.clone()));
    let training = web::Data::new(training::TrainingAPI::new(client.clone()));
    let crypto = web::Data::new(bouncer::Crypto::from_env());
    let casino = web::Data::new(hosting::Casino::new(client.clone()).with_blueprint(blueprint));
    let litmus_backend = litmus::Backend::new(
        strategy::StrategyAPI::new(client.clone()),
        training::TrainingAPI::new(client.clone()),
    );
    let litmus = web::Data::new(Arc::new(::litmus::Litmus::new(litmus_backend)));
    let client = web::Data::new(client);
    tracing::info!("starting unified server");
    HttpServer::new(move || {
        App::new()
            .wrap(metrics::Metrics)
            .wrap(Logger::new("%r %s %Ts").exclude("/health"))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(topology.clone())
            .app_data(strategy.clone())
            .app_data(gameplay.clone())
            .app_data(training.clone())
            .app_data(casino.clone())
            .app_data(crypto.clone())
            .app_data(litmus.clone())
            .app_data(client.clone())
            .route("/health", web::get().to(health))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(bouncer::register))
                    .route("/logout", web::post().to(bouncer::logout))
                    .route("/login", web::post().to(bouncer::login))
                    .route("/me", web::get().to(bouncer::me)),
            )
            .service(
                web::scope("/room")
                    .route("/start", web::post().to(hosting::handlers::start))
                    .route("/enter/{room_id}", web::get().to(hosting::handlers::enter))
                    .route("/leave/{room_id}", web::post().to(hosting::handlers::leave)),
            )
            .service(
                web::scope("/topology")
                    .route("/replace-obs", web::post().to(topology::handlers::replace_obs))
                    .route("/nbr-any-abs", web::post().to(topology::handlers::nbr_any_wrt_abs))
                    .route("/nbr-obs-abs", web::post().to(topology::handlers::nbr_obs_wrt_abs))
                    .route("/nbr-abs-abs", web::post().to(topology::handlers::nbr_abs_wrt_abs))
                    .route("/nbr-kfn-abs", web::post().to(topology::handlers::kfn_wrt_abs))
                    .route("/nbr-knn-abs", web::post().to(topology::handlers::knn_wrt_abs))
                    .route("/nbr-kgn-abs", web::post().to(topology::handlers::kgn_wrt_abs))
                    .route("/exp-wrt-str", web::post().to(topology::handlers::exp_wrt_str))
                    .route("/exp-wrt-abs", web::post().to(topology::handlers::exp_wrt_abs))
                    .route("/exp-wrt-obs", web::post().to(topology::handlers::exp_wrt_obs))
                    .route("/hst-wrt-abs", web::post().to(topology::handlers::hst_wrt_abs))
                    .route("/hst-wrt-obs", web::post().to(topology::handlers::hst_wrt_obs))
                    .route("/distance", web::post().to(topology::handlers::distance)),
            )
            .service(
                web::scope("/strategy")
                    .route("/policy", web::post().to(strategy::handlers::policy))
                    .route("/depth", web::post().to(strategy::handlers::solve_depth))
                    .route("/world", web::post().to(strategy::handlers::solve_world))
                    .route("/full", web::post().to(strategy::handlers::solve_full))
                    .route("/range", web::post().to(strategy::handlers::range))
                    .route("/signalled", web::post().to(strategy::handlers::signalled))
                    .route("/grid-usage", web::get().to(strategy::handlers::grid_usage)),
            )
            .service(
                web::scope("/gameplay")
                    .route("/summary", web::post().to(gameplay::handlers::summary))
                    .route("/aivat", web::post().to(gameplay::handlers::aivat))
                    .route("/hand/{id}", web::get().to(gameplay::handlers::hand)),
            )
            .service(
                web::scope("/training")
                    .route("/status", web::get().to(training::handlers::status))
                    .route("/snapshots", web::post().to(training::handlers::snapshots))
                    .route("/stats", web::get().to(training::handlers::stats))
                    .route("/street-stats", web::get().to(training::handlers::street_stats))
                    .route("/cold", web::post().to(training::handlers::cold))
                    .route("/hot", web::post().to(training::handlers::hot))
                    .route("/convergence", web::post().to(training::handlers::convergence))
                    .route("/saturation", web::get().to(training::handlers::saturation)),
            )
            .service(
                web::scope("/litmus")
                    .route("/run", web::post().to(litmus::handlers::run))
                    .route("/run/markdown", web::post().to(litmus::handlers::report)),
            )
    })
    .workers(6)
    .bind(std::env::var("BIND_ADDR").expect("BIND_ADDR must be set"))?
    .run()
    .await
}

//! Unified Backend Server
//!
//! Combines analysis API routes and live game hosting routes
//! into a single actix-web server.
//!
//! ## Submodules
//!
//! - [`analysis`] — Training result analysis and query interface
//! - [`hosting`] — WebSocket game hosting infrastructure

pub mod analysis;
pub mod hosting;

// Re-export main types (not handlers or Client, which conflict between modules)
pub use analysis::API;
pub use analysis::CLI;
pub use analysis::Query;
pub use hosting::Casino;
pub use hosting::RoomHandle;

use actix_cors::Cors;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::middleware::Logger;
use actix_web::web;
use std::sync::Arc;
use tokio_postgres::Client;

async fn health(client: web::Data<Arc<Client>>) -> impl Responder {
    match client
        .execute("SELECT 1", &[])
        .await
        .inspect_err(|e| log::error!("health check failed: {}", e))
    {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(_) => HttpResponse::ServiceUnavailable().body("database unavailable"),
    }
}

#[rustfmt::skip]
pub async fn run() -> Result<(), std::io::Error> {
    let client = rbp_database::db().await;
    let api = web::Data::new(analysis::API::new(client.clone()));
    let crypto = web::Data::new(rbp_auth::Crypto::from_env());
    let casino = web::Data::new(hosting::Casino::new(client.clone()));
    let client = web::Data::new(client);
    log::info!("starting unified server");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%r %s %Ts"))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(api.clone())
            .app_data(casino.clone())
            .app_data(crypto.clone())
            .app_data(client.clone())
            .route("/health", web::get().to(health))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(rbp_auth::register))
                    .route("/logout", web::post().to(rbp_auth::logout))
                    .route("/login", web::post().to(rbp_auth::login))
                    .route("/me", web::get().to(rbp_auth::me)),
            )
            .service(
                web::scope("/room")
                    .route("/start", web::post().to(hosting::handlers::start))
                    .route("/enter/{room_id}", web::get().to(hosting::handlers::enter))
                    .route("/leave/{room_id}", web::post().to(hosting::handlers::leave)),
            )
            .service(
                web::scope("/api")
                    .route("/replace-obs", web::post().to(analysis::handlers::replace_obs))
                    .route("/nbr-any-abs", web::post().to(analysis::handlers::nbr_any_wrt_abs))
                    .route("/nbr-obs-abs", web::post().to(analysis::handlers::nbr_obs_wrt_abs))
                    .route("/nbr-abs-abs", web::post().to(analysis::handlers::nbr_abs_wrt_abs))
                    .route("/nbr-kfn-abs", web::post().to(analysis::handlers::kfn_wrt_abs))
                    .route("/nbr-knn-abs", web::post().to(analysis::handlers::knn_wrt_abs))
                    .route("/nbr-kgn-abs", web::post().to(analysis::handlers::kgn_wrt_abs))
                    .route("/exp-wrt-str", web::post().to(analysis::handlers::exp_wrt_str))
                    .route("/exp-wrt-abs", web::post().to(analysis::handlers::exp_wrt_abs))
                    .route("/exp-wrt-obs", web::post().to(analysis::handlers::exp_wrt_obs))
                    .route("/hst-wrt-abs", web::post().to(analysis::handlers::hst_wrt_abs))
                    .route("/hst-wrt-obs", web::post().to(analysis::handlers::hst_wrt_obs))
                    .route("/blueprint", web::post().to(analysis::handlers::blueprint)),
            )
    })
    .workers(6)
    .bind(std::env::var("BIND_ADDR").expect("BIND_ADDR must be set"))?
    .run()
    .await
}

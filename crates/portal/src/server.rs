use super::*;

/// Server bootstrap: schema, health, and the actix entry point.
pub struct Server;

impl Server {
    /// Ensures all tables exist. Single point of truth for schema creation.
    async fn ensure_all(client: &Client) {
        use daybook::Ensure;
        client.ensure::<bouncer::Member>().await;
        client.ensure::<bouncer::Session>().await;
        client.ensure::<parlor::Room>().await;
        client.ensure::<parlor::records::Hand>().await;
        client.ensure::<parlor::Participant>().await;
        client.ensure::<parlor::Play>().await;
        client.ensure::<nlhe::NlheBlueprint>().await;
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
        let client = daybook::Pool::db().await;
        Server::ensure_all(&client).await;
        daybook::Fingerprint::report(&client).await;
        use parlor::AdversaryExt;
        use parlor::VariantExt;
        let mut seedables = pokerkit::Variant::all()
            .iter()
            .copied()
            .map(pokerkit::Variant::member)
            .collect::<Vec<bouncer::Member>>();
        seedables.extend(pokerkit::Adversary::all().iter().copied().map(pokerkit::Adversary::member));
        for member in &seedables {
            bouncer::AuthRepository::seed(&client, member)
                .await
                .unwrap_or_else(|e| tracing::warn!("failed to seed {}: {}", member.username(), e));
            tracing::info!("seeded bot user {}", member.username());
        }
        let blueprint: Option<&'static nlhe::Flagship> = if std::env::var("SKIP_BLUEPRINT").ok().as_deref() == Some("1") {
            tracing::warn!("SKIP_BLUEPRINT=1: skipping blueprint hydration (only Fish opponents will work)");
            None
        } else {
            tracing::info!("loading blueprint into memory");
            Some(Box::leak(Box::new(nlhe::NlheBlueprint::solver(client.clone()).await)))
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
                .route("/health", web::get().to(Server::health))
                .service(
                    web::scope("/auth")
                        .route("/register", web::post().to(bouncer::Handlers::register))
                        .route("/logout", web::post().to(bouncer::Handlers::logout))
                        .route("/login", web::post().to(bouncer::Handlers::login))
                        .route("/me", web::get().to(bouncer::Handlers::me)),
                )
                .service(
                    web::scope("/room")
                        .route("/start", web::post().to(hosting::handlers::Handlers::start))
                        .route("/enter/{room_id}", web::get().to(hosting::handlers::Handlers::enter))
                        .route("/leave/{room_id}", web::post().to(hosting::handlers::Handlers::leave)),
                )
                .service(
                    web::scope("/topology")
                        .route("/replace-obs", web::post().to(topology::handlers::Handlers::replace_obs))
                        .route("/nbr-any-abs", web::post().to(topology::handlers::Handlers::nbr_any_wrt_abs))
                        .route("/nbr-obs-abs", web::post().to(topology::handlers::Handlers::nbr_obs_wrt_abs))
                        .route("/nbr-abs-abs", web::post().to(topology::handlers::Handlers::nbr_abs_wrt_abs))
                        .route("/nbr-kfn-abs", web::post().to(topology::handlers::Handlers::kfn_wrt_abs))
                        .route("/nbr-knn-abs", web::post().to(topology::handlers::Handlers::knn_wrt_abs))
                        .route("/nbr-kgn-abs", web::post().to(topology::handlers::Handlers::kgn_wrt_abs))
                        .route("/exp-wrt-str", web::post().to(topology::handlers::Handlers::exp_wrt_str))
                        .route("/exp-wrt-abs", web::post().to(topology::handlers::Handlers::exp_wrt_abs))
                        .route("/exp-wrt-obs", web::post().to(topology::handlers::Handlers::exp_wrt_obs))
                        .route("/hst-wrt-abs", web::post().to(topology::handlers::Handlers::hst_wrt_abs))
                        .route("/hst-wrt-obs", web::post().to(topology::handlers::Handlers::hst_wrt_obs))
                        .route("/gloss", web::post().to(topology::handlers::Handlers::gloss))
                        .route("/distance", web::post().to(topology::handlers::Handlers::distance)),
                )
                .service(
                    web::scope("/strategy")
                        .route("/policy", web::post().to(strategy::handlers::Handlers::policy))
                        .route("/blueprint", web::post().to(strategy::handlers::Handlers::strategies))
                        .route("/depth", web::post().to(strategy::handlers::Handlers::solve_depth))
                        .route("/world", web::post().to(strategy::handlers::Handlers::solve_world))
                        .route("/full", web::post().to(strategy::handlers::Handlers::solve_full))
                        .route("/range", web::post().to(strategy::handlers::Handlers::range))
                        .route("/signalled", web::post().to(strategy::handlers::Handlers::signalled))
                        .route("/grid-usage", web::get().to(strategy::handlers::Handlers::grid_usage)),
                )
                .service(
                    web::scope("/gameplay")
                        .route("/summary", web::post().to(gameplay::handlers::Handlers::summary))
                        .route("/aivat", web::post().to(gameplay::handlers::Handlers::aivat))
                        .route("/hands", web::post().to(gameplay::handlers::Handlers::hands))
                        .route("/hand/{id}", web::get().to(gameplay::handlers::Handlers::hand)),
                )
                .service(
                    web::scope("/training")
                        .route("/status", web::get().to(training::handlers::Handlers::status))
                        .route("/snapshots", web::post().to(training::handlers::Handlers::snapshots))
                        .route("/stats", web::get().to(training::handlers::Handlers::stats))
                        .route("/street-stats", web::get().to(training::handlers::Handlers::street_stats))
                        .route("/cold", web::post().to(training::handlers::Handlers::cold))
                        .route("/hot", web::post().to(training::handlers::Handlers::hot))
                        .route("/convergence", web::post().to(training::handlers::Handlers::convergence))
                        .route("/saturation", web::get().to(training::handlers::Handlers::saturation)),
                )
                .service(
                    web::scope("/litmus")
                        .route("/run", web::post().to(litmus::handlers::Handlers::run))
                        .route("/run/markdown", web::post().to(litmus::handlers::Handlers::report)),
                )
        })
        .workers(6)
        .bind(std::env::var("BIND_ADDR").expect("BIND_ADDR must be set"))?
        .run()
        .await
    }
}

use super::api::API;
use super::request::ReplaceAbs;
use super::request::ReplaceObs;
use super::request::ReplaceRow;
use super::request::SetStreets;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;

pub struct Server;

impl Server {
    pub async fn run() -> Result<(), std::io::Error> {
        let api = web::Data::new(API::new().await);
        log::info!("starting HTTP server");
        HttpServer::new(move || {
            App::new()
                .wrap(Logger::new("%r %s %D ms"))
                .wrap(
                    Cors::default()
                        .allow_any_origin()
                        .allow_any_method()
                        .allow_any_header(),
                )
                .app_data(api.clone())
                .route("/set-streets", web::post().to(set_streets))
                .route("/replace-obs", web::post().to(replace_obs))
                .route("/replace-abs", web::post().to(replace_abs))
                .route("/kfn-wrt-abs", web::post().to(get_kfn_wrt_abs))
                .route("/knn-wrt-abs", web::post().to(get_knn_wrt_abs))
                .route("/any-wrt-abs", web::post().to(get_any_wrt_abs))
                .route("/distributio", web::post().to(get_distributio))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
    }
}

// Route handlers
async fn set_streets(api: web::Data<API>, req: web::Json<SetStreets>) -> impl Responder {
    match Street::try_from(req.street.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid street format"),
        Ok(street) => match api.any_row_wrt_street(street).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}

async fn replace_obs(api: web::Data<API>, req: web::Json<ReplaceObs>) -> impl Responder {
    match Observation::try_from(req.obs.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid observation format"),
        Ok(old) => match api.any_obs_wrt_obs(old).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(new) => HttpResponse::Ok().json(new.to_string()),
        },
    }
}

async fn replace_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.any_row_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}

async fn get_distributio(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.table_distribution(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}

async fn get_kfn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.table_neighborhood_kfn(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}

async fn get_knn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.table_neighborhood_knn(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}

async fn get_any_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceRow>) -> impl Responder {
    match (
        Abstraction::try_from(req.wrt.as_str()),
        Observation::try_from(req.obs.as_str()),
    ) {
        (Err(_), _) => HttpResponse::BadRequest().body("invalid abstraction format"),
        (_, Err(_)) => HttpResponse::BadRequest().body("invalid observation format"),
        (Ok(abs), Ok(obs)) => match api.obs_row_wrt_abs(abs, obs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}

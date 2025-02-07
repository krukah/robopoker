use super::api::API;
use super::request::ReplaceAbs;
use super::request::ReplaceAll;
use super::request::ReplaceObs;
use super::request::ReplaceOne;
use super::request::ReplaceRow;
use super::request::RowWrtObs;
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
        let api = web::Data::new(API::from(crate::db().await));
        log::info!("starting HTTP server");
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
                .route("/replace-obs", web::post().to(replace_obs))
                .route("/nbr-any-wrt-abs", web::post().to(nbr_any_wrt_abs))
                .route("/nbr-obs-wrt-abs", web::post().to(nbr_obs_wrt_abs))
                .route("/nbr-abs-wrt-abs", web::post().to(nbr_abs_wrt_abs))
                .route("/nbr-kfn-wrt-abs", web::post().to(kfn_wrt_abs))
                .route("/nbr-knn-wrt-abs", web::post().to(knn_wrt_abs))
                .route("/nbr-kgn-wrt-abs", web::post().to(kgn_wrt_abs))
                .route("/exp-wrt-str", web::post().to(exp_wrt_str))
                .route("/exp-wrt-abs", web::post().to(exp_wrt_abs))
                .route("/exp-wrt-obs", web::post().to(exp_wrt_obs))
                .route("/hst-wrt-abs", web::post().to(hst_wrt_abs))
                .route("/hst-wrt-obs", web::post().to(hst_wrt_obs))
        })
        .workers(6)
        .bind("127.0.0.1:8888")?
        .run()
        .await
    }
}

// Route handlers

async fn replace_obs(api: web::Data<API>, req: web::Json<ReplaceObs>) -> impl Responder {
    match Observation::try_from(req.obs.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid observation format"),
        Ok(obs) => match api.replace_obs(obs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(new) => HttpResponse::Ok().json(new.equivalent()),
        },
    }
}

async fn exp_wrt_str(api: web::Data<API>, req: web::Json<SetStreets>) -> impl Responder {
    match Street::try_from(req.street.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid street format"),
        Ok(street) => match api.exp_wrt_str(street).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}
async fn exp_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.exp_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}
async fn exp_wrt_obs(api: web::Data<API>, req: web::Json<RowWrtObs>) -> impl Responder {
    match Observation::try_from(req.obs.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid observation format"),
        Ok(obs) => match api.exp_wrt_obs(obs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}

async fn nbr_any_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.nbr_any_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}
async fn nbr_abs_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceOne>) -> impl Responder {
    match (
        Abstraction::try_from(req.wrt.as_str()),
        Abstraction::try_from(req.abs.as_str()),
    ) {
        (Err(_), _) => HttpResponse::BadRequest().body("invalid abstraction format"),
        (_, Err(_)) => HttpResponse::BadRequest().body("invalid abstraction format"),
        (Ok(wrt), Ok(abs)) => match api.nbr_abs_wrt_abs(wrt, abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}
async fn nbr_obs_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceRow>) -> impl Responder {
    match (
        Abstraction::try_from(req.wrt.as_str()),
        Observation::try_from(req.obs.as_str()),
    ) {
        (Err(_), _) => HttpResponse::BadRequest().body("invalid abstraction format"),
        (_, Err(_)) => HttpResponse::BadRequest().body("invalid observation format"),
        (Ok(abs), Ok(obs)) => match api.nbr_obs_wrt_abs(abs, obs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}

async fn kfn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.kfn_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}
async fn knn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.knn_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}
async fn kgn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAll>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(wrt) => {
            let obs = req
                .neighbors
                .iter()
                .map(|string| string.as_str())
                .map(Observation::try_from)
                .filter_map(|result| result.ok())
                .filter(|o| o.street() == wrt.street())
                .chain((0..).map(|_| Observation::from(wrt.street())))
                .take(5)
                .collect::<Vec<_>>();
            match api.kgn_wrt_abs(wrt, obs).await {
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                Ok(rows) => HttpResponse::Ok().json(rows),
            }
        }
    }
}

async fn hst_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.hst_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}

async fn hst_wrt_obs(api: web::Data<API>, req: web::Json<ReplaceObs>) -> impl Responder {
    match Observation::try_from(req.obs.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid observation format"),
        Ok(obs) => match api.hst_wrt_obs(obs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}

use super::api::API;
use super::request::ObsAbsWrtRequest;
use super::request::ReplaceAbsRequest;
use super::request::ReplaceObsRequest;
use super::response::ObsAbsResponse;
use actix_cors::Cors;
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
                .wrap(
                    Cors::default()
                        .allow_any_origin()
                        .allow_any_method()
                        .allow_any_header(),
                )
                .app_data(api.clone())
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
async fn replace_obs(api: web::Data<API>, req: web::Json<ReplaceObsRequest>) -> impl Responder {
    HttpResponse::Ok().json({})
}

async fn replace_abs(api: web::Data<API>, req: web::Json<ReplaceAbsRequest>) -> impl Responder {
    HttpResponse::Ok().json({})
}

async fn get_any_wrt_abs(api: web::Data<API>, req: web::Json<ObsAbsWrtRequest>) -> impl Responder {
    HttpResponse::Ok().json({})
}

async fn get_kfn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbsRequest>) -> impl Responder {
    HttpResponse::Ok().json(Vec::<ObsAbsResponse>::new())
}

async fn get_knn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbsRequest>) -> impl Responder {
    HttpResponse::Ok().json(Vec::<ObsAbsResponse>::new())
}

async fn get_distributio(api: web::Data<API>, req: web::Json<ReplaceAbsRequest>) -> impl Responder {
    HttpResponse::Ok().json(Vec::<ObsAbsResponse>::new())
}

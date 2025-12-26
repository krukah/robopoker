use super::*;
use actix_cors::Cors;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::middleware::Logger;
use actix_web::web;

pub struct Server;

impl Server {
    pub async fn run() -> Result<(), std::io::Error> {
        let state = web::Data::new(Casino::default());
        log::info!("starting hosting server");
        HttpServer::new(move || {
            App::new()
                .wrap(Logger::new("%r %s %Ts"))
                .wrap(
                    Cors::default()
                        .allow_any_origin()
                        .allow_any_method()
                        .allow_any_header(),
                )
                .app_data(state.clone())
                .route("/start", web::post().to(start))
                .route("/enter/{room_id}", web::get().to(enter))
                .route("/leave/{room_id}", web::post().to(leave))
        })
        .workers(4)
        .bind(std::env::var("BIND_ADDR").expect("BIND_ADDR must be set"))?
        .run()
        .await
    }
}

async fn start(casino: web::Data<Casino>) -> impl Responder {
    match casino.start().await {
        Ok(id) => HttpResponse::Ok().json(serde_json::json!({ "room_id": id })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn leave(casino: web::Data<Casino>, path: web::Path<RoomId>) -> impl Responder {
    let id = path.into_inner();
    match casino.close(id).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "status": "left" })),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

async fn enter(
    casino: web::Data<Casino>,
    path: web::Path<RoomId>,
    body: web::Payload,
    req: HttpRequest,
) -> impl Responder {
    let id = path.into_inner();
    match actix_ws::handle(&req, body) {
        Ok((response, session, stream)) => match casino.bridge(id, session, stream).await {
            Ok(()) => response.map_into_left_body(),
            Err(e) => HttpResponse::NotFound()
                .body(e.to_string())
                .map_into_right_body(),
        },
        Err(e) => HttpResponse::InternalServerError()
            .body(e.to_string())
            .map_into_right_body(),
    }
}

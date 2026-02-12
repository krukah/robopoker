use super::*;
use rbp_auth;
use rbp_core::ID;
use rbp_gameroom::Room;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;

pub async fn start(casino: web::Data<Casino>) -> impl Responder {
    match casino.into_inner().start().await {
        Ok(id) => HttpResponse::Ok().json(serde_json::json!({ "room_id": id.to_string() })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
pub async fn leave(casino: web::Data<Casino>, path: web::Path<uuid::Uuid>) -> impl Responder {
    match casino.close(ID::from(path.into_inner())).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "status": "left" })),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}
pub async fn enter(
    casino: web::Data<Casino>,
    tokens: web::Data<rbp_auth::Crypto>,
    path: web::Path<uuid::Uuid>,
    query: web::Query<std::collections::HashMap<String, String>>,
    body: web::Payload,
    req: HttpRequest,
) -> impl Responder {
    let id: ID<Room> = ID::from(path.into_inner());
    query
        .get("token")
        .and_then(|t| tokens.decode(t).ok())
        .filter(|c| !c.expired())
        .inspect(|c| log::info!("authenticated user {} entering room {}", c.usr, id))
        .map(std::mem::drop)
        .unwrap_or_else(|| log::info!("anonymous user entering room {}", id));
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

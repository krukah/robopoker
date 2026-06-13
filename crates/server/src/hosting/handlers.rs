use super::*;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;
use rbp_auth;
use rbp_core::ID;
use rbp_core::Variant;
use rbp_gameroom::Room;

pub async fn start(casino: web::Data<Casino>, body: web::Json<Variant>) -> impl Responder {
    let variant = body.into_inner();
    match casino.into_inner().start(variant).await {
        Ok(id) => HttpResponse::Ok().json(serde_json::json!({
            "room_id": id.to_string(),
            "variant": variant,
        })),
        Err(e) => {
            tracing::error!(error = %e, "room start failed");
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}
pub async fn leave(casino: web::Data<Casino>, path: web::Path<uuid::Uuid>) -> impl Responder {
    match casino.close(ID::from(path.into_inner())).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "status": "left" })),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}
// actix `web::Query<HashMap>` is an opaque extractor type; the hasher param would not propagate usefully.
#[allow(clippy::implicit_hasher)]
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
        .inspect(|c| tracing::debug!(user = %c.usr, room = %id, "authenticated user entering room"))
        .map_or_else(|| tracing::debug!(room = %id, "anonymous user entering room"), std::mem::drop);
    match actix_ws::handle(&req, body) {
        Ok((response, session, stream)) => match casino.bridge(id, session, stream).await {
            Ok(()) => response.map_into_left_body(),
            Err(e) => {
                tracing::warn!(room = %id, error = %e, "bridge failed for room");
                HttpResponse::NotFound().body(e.to_string()).map_into_right_body()
            }
        },
        Err(e) => {
            tracing::warn!(room = %id, error = %e, "websocket upgrade failed for room");
            HttpResponse::InternalServerError()
                .body(e.to_string())
                .map_into_right_body()
        }
    }
}

use super::GameplayAPI;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;
use cowboys::*;

pub async fn summary(api: web::Data<GameplayAPI>, req: web::Json<GetSummary>) -> impl Responder {
    match api
        .summary(req.user, req.limit, req.offset, req.against, req.stakes, req.hero_human, req.against_human)
        .await
    {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}
pub async fn aivat(api: web::Data<GameplayAPI>, req: web::Json<GetSummary>) -> impl Responder {
    match api
        .aivat(req.user, req.limit, req.offset, req.against, req.stakes, req.hero_human, req.against_human)
        .await
    {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}
pub async fn hand(api: web::Data<GameplayAPI>, path: web::Path<uuid::Uuid>) -> impl Responder {
    match api.hand_recap(path.into_inner()).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(r) => HttpResponse::Ok().json(r),
    }
}

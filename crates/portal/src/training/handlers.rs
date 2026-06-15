use super::TrainingAPI;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;
use kicker::*;

pub async fn status(api: web::Data<TrainingAPI>) -> impl Responder {
    match api.status().await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}
pub async fn snapshots(api: web::Data<TrainingAPI>, req: web::Json<GetSnapshots>) -> impl Responder {
    match api.snapshots(req.limit, req.offset).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}
pub async fn stats(api: web::Data<TrainingAPI>) -> impl Responder {
    match api.stats().await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}
pub async fn street_stats(api: web::Data<TrainingAPI>) -> impl Responder {
    match api.street_stats().await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}
pub async fn cold(api: web::Data<TrainingAPI>, req: web::Json<GetColdHot>) -> impl Responder {
    match api.cold(req.limit).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}
pub async fn hot(api: web::Data<TrainingAPI>, req: web::Json<GetColdHot>) -> impl Responder {
    match api.hot(req.limit).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}
pub async fn convergence(api: web::Data<TrainingAPI>, req: web::Json<GetSnapshots>) -> impl Responder {
    match api.convergence(req.limit).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}
pub async fn saturation(api: web::Data<TrainingAPI>) -> impl Responder {
    match api.saturation().await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(s) => HttpResponse::Ok().json(s),
    }
}

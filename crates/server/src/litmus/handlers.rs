//! HTTP handlers for the `/litmus/*` route family.
//!
//! Two endpoints, both POST scenarios.json in the request body:
//!   /litmus/run            → JSON `Vec<Outcome>`
//!   /litmus/run/markdown   → text/markdown report

use super::Backend;
use actix_web::{HttpResponse, Responder, web};
use rbp_litmus::{Litmus, Scenarios};
use std::sync::Arc;

pub async fn run(
    api: web::Data<Arc<Litmus<Backend>>>,
    req: web::Json<Scenarios>,
) -> impl Responder {
    match api.run(&req).await {
        Ok(outcomes) => HttpResponse::Ok().json(outcomes),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn report(
    api: web::Data<Arc<Litmus<Backend>>>,
    req: web::Json<Scenarios>,
) -> impl Responder {
    let api_label = format!("rbp-{} {}", rbp_core::regime(), rbp_core::version());
    match api.report(&req, &api_label).await {
        Ok(md) => HttpResponse::Ok()
            .content_type("text/markdown; charset=utf-8")
            .body(md),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

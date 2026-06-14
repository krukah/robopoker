use super::StrategyAPI;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;
use cowboys::*;

pub async fn policy(api: web::Data<StrategyAPI>, req: web::Json<GetPolicy>) -> impl Responder {
    let started = std::time::Instant::now();
    match Witness::try_build(req.turn, req.seen, req.past.clone()) {
        Err(e) => HttpResponse::BadRequest().body(format!("invalid action sequence: {e}")),
        Ok(recall) => api
            .policy(recall)
            .await
            .map(|opt| opt.map(|p| ApiSolved::blueprint(p, started.elapsed())))
            .map_or_else(
                |e| HttpResponse::InternalServerError().body(e.to_string()),
                |opt| HttpResponse::Ok().json(opt),
            ),
    }
}

pub async fn solve_depth(api: web::Data<StrategyAPI>, req: web::Json<GetPolicy>) -> impl Responder {
    solve(req, |recall| api.solve_depth(recall)).await
}

pub async fn solve_world(api: web::Data<StrategyAPI>, req: web::Json<GetPolicy>) -> impl Responder {
    solve(req, |recall| api.solve_world(recall)).await
}

pub async fn solve_full(api: web::Data<StrategyAPI>, req: web::Json<GetPolicy>) -> impl Responder {
    solve(req, |recall| api.solve_full(recall)).await
}

/// Validates the witness and dispatches to the kind-specific solver.
/// Wrapper exists because the three solve handlers share everything
/// except *which* `StrategyAPI::solve_*` they call.
async fn solve<F, Fut>(req: web::Json<GetPolicy>, dispatch: F) -> HttpResponse
where
    F: FnOnce(Witness) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<ApiSolved>>,
{
    match Witness::try_build(req.turn, req.seen, req.past.clone()) {
        Err(e) => HttpResponse::BadRequest().body(format!("invalid action sequence: {e}")),
        Ok(recall) => dispatch(recall).await.map_or_else(
            |e| HttpResponse::InternalServerError().body(e.to_string()),
            |solved| HttpResponse::Ok().json(solved),
        ),
    }
}

pub async fn range(api: web::Data<StrategyAPI>, req: web::Json<GetPolicy>) -> impl Responder {
    posterior(req, |r| api.range(r))
}

pub async fn signalled(api: web::Data<StrategyAPI>, req: web::Json<GetPolicy>) -> impl Responder {
    posterior(req, |r| api.signalled(r))
}

fn posterior<F>(req: web::Json<GetPolicy>, compute: F) -> HttpResponse
where
    F: FnOnce(Witness) -> anyhow::Result<ApiOpponentRange>,
{
    match Witness::try_build(req.turn, req.seen, req.past.clone()) {
        Err(e) => HttpResponse::BadRequest().body(format!("invalid action sequence: {e}")),
        Ok(recall) => compute(recall).map_or_else(
            |e| HttpResponse::InternalServerError().body(e.to_string()),
            |range| HttpResponse::Ok().json(range),
        ),
    }
}

pub async fn grid_usage(api: web::Data<StrategyAPI>) -> impl Responder {
    match api.grid_usage().await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(rows) => HttpResponse::Ok().json(rows),
    }
}

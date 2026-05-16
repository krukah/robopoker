use super::TopologyAPI;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;
use rbp_cards::*;
use rbp_gameplay::*;

pub async fn replace_obs(
    api: web::Data<TopologyAPI>,
    req: web::Json<ReplaceObs>,
) -> impl Responder {
    match api.replace_obs(req.obs).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(new) => HttpResponse::Ok().json(new),
    }
}
pub async fn exp_wrt_str(
    api: web::Data<TopologyAPI>,
    req: web::Json<SetStreets>,
) -> impl Responder {
    match api.exp_wrt_str(req.street).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(row) => HttpResponse::Ok().json(row),
    }
}
pub async fn exp_wrt_abs(
    api: web::Data<TopologyAPI>,
    req: web::Json<ReplaceAbs>,
) -> impl Responder {
    match api.exp_wrt_abs(req.wrt).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(row) => HttpResponse::Ok().json(row),
    }
}
pub async fn exp_wrt_obs(api: web::Data<TopologyAPI>, req: web::Json<RowWrtObs>) -> impl Responder {
    match api.exp_wrt_obs(req.obs).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(row) => HttpResponse::Ok().json(row),
    }
}
pub async fn nbr_any_wrt_abs(
    api: web::Data<TopologyAPI>,
    req: web::Json<ReplaceAbs>,
) -> impl Responder {
    match api.nbr_any_wrt_abs(req.wrt).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(row) => HttpResponse::Ok().json(row),
    }
}
pub async fn nbr_abs_wrt_abs(
    api: web::Data<TopologyAPI>,
    req: web::Json<ReplaceOne>,
) -> impl Responder {
    match api.nbr_abs_wrt_abs(req.wrt, req.abs).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(row) => HttpResponse::Ok().json(row),
    }
}
pub async fn nbr_obs_wrt_abs(
    api: web::Data<TopologyAPI>,
    req: web::Json<ReplaceRow>,
) -> impl Responder {
    match api.nbr_obs_wrt_abs(req.wrt, req.obs).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(row) => HttpResponse::Ok().json(row),
    }
}
pub async fn kfn_wrt_abs(
    api: web::Data<TopologyAPI>,
    req: web::Json<ReplaceAbs>,
) -> impl Responder {
    match api.kfn_wrt_abs(req.wrt).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(rows) => HttpResponse::Ok().json(rows),
    }
}
pub async fn knn_wrt_abs(
    api: web::Data<TopologyAPI>,
    req: web::Json<ReplaceAbs>,
) -> impl Responder {
    match api.knn_wrt_abs(req.wrt).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(rows) => HttpResponse::Ok().json(rows),
    }
}
pub async fn kgn_wrt_abs(
    api: web::Data<TopologyAPI>,
    req: web::Json<ReplaceAll>,
) -> impl Responder {
    let obs = req
        .neighbors
        .iter()
        .copied()
        .filter(|o| o.street() == req.wrt.street())
        .chain((0..).map(|_| Observation::from(req.wrt.street())))
        .take(5)
        .collect::<Vec<_>>();
    match api.kgn_wrt_abs(req.wrt, obs).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(rows) => HttpResponse::Ok().json(rows),
    }
}
pub async fn hst_wrt_abs(api: web::Data<TopologyAPI>, req: web::Json<AbsHist>) -> impl Responder {
    match api.hst_wrt_abs(req.abs).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(rows) => HttpResponse::Ok().json(rows),
    }
}
pub async fn hst_wrt_obs(api: web::Data<TopologyAPI>, req: web::Json<ObsHist>) -> impl Responder {
    match api.hst_wrt_obs(req.obs).await {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(rows) => HttpResponse::Ok().json(rows),
    }
}

pub async fn distance(api: web::Data<TopologyAPI>, req: web::Json<GetDistance>) -> impl Responder {
    let a_obs = Observation::try_from(req.a.as_str()).ok();
    let a_abs = Abstraction::try_from(req.a.as_str()).ok();
    let b_obs = Observation::try_from(req.b.as_str()).ok();
    let b_abs = Abstraction::try_from(req.b.as_str()).ok();
    let result = match (a_obs, a_abs, b_obs, b_abs) {
        (Some(o1), _, Some(o2), _) => api.obs_distance(o1, o2).await,
        (_, Some(x1), _, Some(x2)) => api.abs_distance(x1, x2).await,
        (Some(o), _, _, Some(x)) => api.obs_abs_distance(o, x).await,
        (_, Some(x), Some(o), _) => api.obs_abs_distance(o, x).await,
        _ => return HttpResponse::BadRequest().body("invalid distance targets"),
    };
    match result {
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        Ok(d) => HttpResponse::Ok().json(d),
    }
}

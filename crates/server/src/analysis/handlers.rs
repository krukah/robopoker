use super::API;
use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;

pub async fn replace_obs(api: web::Data<API>, req: web::Json<ReplaceObs>) -> impl Responder {
    match Observation::try_from(req.obs.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid observation format"),
        Ok(obs) => match api.replace_obs(obs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(new) => HttpResponse::Ok().json(new.to_string()),
        },
    }
}
pub async fn exp_wrt_str(api: web::Data<API>, req: web::Json<SetStreets>) -> impl Responder {
    match Street::try_from(req.street.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid street format"),
        Ok(street) => match api.exp_wrt_str(street).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}
pub async fn exp_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.exp_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}
pub async fn exp_wrt_obs(api: web::Data<API>, req: web::Json<RowWrtObs>) -> impl Responder {
    match Observation::try_from(req.obs.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid observation format"),
        Ok(obs) => match api.exp_wrt_obs(obs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}
pub async fn nbr_any_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.nbr_any_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}
pub async fn nbr_abs_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceOne>) -> impl Responder {
    let wrt = Abstraction::try_from(req.wrt.as_str());
    let abs = Abstraction::try_from(req.abs.as_str());
    match (wrt, abs) {
        (Err(_), _) => HttpResponse::BadRequest().body("invalid abstraction format"),
        (_, Err(_)) => HttpResponse::BadRequest().body("invalid abstraction format"),
        (Ok(wrt), Ok(abs)) => match api.nbr_abs_wrt_abs(wrt, abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(row) => HttpResponse::Ok().json(row),
        },
    }
}
pub async fn nbr_obs_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceRow>) -> impl Responder {
    let wrt = Abstraction::try_from(req.wrt.as_str());
    let obs = Observation::try_from(req.obs.as_str());
    match (wrt, obs) {
        (Err(_), _) => HttpResponse::BadRequest().body("invalid abstraction format"),
        (_, Err(_)) => HttpResponse::BadRequest().body("invalid observation format"),
        (Ok(abs), Ok(obs)) => match api.nbr_obs_wrt_abs(abs, obs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}
pub async fn kfn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.kfn_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}
pub async fn knn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAbs>) -> impl Responder {
    match Abstraction::try_from(req.wrt.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.knn_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}
pub async fn kgn_wrt_abs(api: web::Data<API>, req: web::Json<ReplaceAll>) -> impl Responder {
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
pub async fn hst_wrt_abs(api: web::Data<API>, req: web::Json<AbsHist>) -> impl Responder {
    match Abstraction::try_from(req.abs.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid abstraction format"),
        Ok(abs) => match api.hst_wrt_abs(abs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}
pub async fn hst_wrt_obs(api: web::Data<API>, req: web::Json<ObsHist>) -> impl Responder {
    match Observation::try_from(req.obs.as_str()) {
        Err(_) => HttpResponse::BadRequest().body("invalid observation format"),
        Ok(obs) => match api.hst_wrt_obs(obs).await {
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            Ok(rows) => HttpResponse::Ok().json(rows),
        },
    }
}
pub async fn blueprint(api: web::Data<API>, req: web::Json<GetPolicy>) -> impl Responder {
    let hero = Turn::try_from(req.turn.as_str());
    let seen = Observation::try_from(req.seen.as_str());
    let path = req
        .past
        .iter()
        .map(|string| string.as_str())
        .map(Action::try_from)
        .collect::<Result<Vec<_>, _>>();
    match (hero, seen, path) {
        (Ok(hero), Ok(seen), Ok(path)) => match Partial::try_build(hero, seen, path) {
            Err(e) => HttpResponse::BadRequest().body(format!("invalid action sequence: {}", e)),
            Ok(recall) => match api.policy(recall).await {
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                Ok(Some(strategy)) => HttpResponse::Ok().json(strategy),
                Ok(None) => HttpResponse::Ok().json(serde_json::Value::Null),
            },
        },
        _ => HttpResponse::BadRequest().body("invalid recall format"),
    }
}

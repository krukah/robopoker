use super::*;
use rbp_core::ID;
use rbp_core::Unique;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;
use std::sync::Arc;
use tokio_postgres::Client;

pub async fn register(
    db: web::Data<Arc<Client>>,
    tokens: web::Data<Crypto>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    if req.username.len() < 3 || req.username.len() > 32 {
        return HttpResponse::BadRequest().body("username must be 3-32 characters");
    }
    if req.password.len() < 8 {
        return HttpResponse::BadRequest().body("password must be at least 8 characters");
    }
    match db.exists(&req.username, &req.email).await {
        Ok(false) => {}
        Ok(true) => return HttpResponse::Conflict().body("username or email already exists"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
    let hashword = match password::hash(&req.password) {
        Ok(h) => h,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let member = Member::new(ID::default(), req.username.clone(), req.email.clone());
    if let Err(e) = db.create(&member, &hashword).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }
    let token_hash = Crypto::hash(&format!("{}", member.id()));
    let session = Session::new(ID::default(), member.id(), token_hash);
    if let Err(e) = db.signin(&session).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }
    let claims = Claims::new(member.id(), session.id(), member.username().to_string());
    let token = match tokens.encode(&claims) {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserInfo {
            id: member.id().to_string(),
            username: member.username().to_string(),
        },
    })
}

pub async fn login(
    db: web::Data<Arc<Client>>,
    tokens: web::Data<Crypto>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    let (member, hashword) = match db.lookup(&req.username).await {
        Ok(Some(row)) => row,
        Ok(None) => return HttpResponse::Unauthorized().body("invalid credentials"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    if !password::verify(&req.password, &hashword) {
        return HttpResponse::Unauthorized().body("invalid credentials");
    }
    let token_hash = Crypto::hash(&format!("{}", member.id()));
    let session = Session::new(ID::default(), member.id(), token_hash);
    if let Err(e) = db.signin(&session).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }
    let claims = Claims::new(member.id(), session.id(), member.username().to_string());
    let token = match tokens.encode(&claims) {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserInfo {
            id: member.id().to_string(),
            username: member.username().to_string(),
        },
    })
}

pub async fn logout(db: web::Data<Arc<Client>>, auth: Auth) -> impl Responder {
    match db.revoke(auth.claims().session()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "logged_out"})),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn me(auth: Auth) -> impl Responder {
    HttpResponse::Ok().json(UserInfo {
        id: auth.user().to_string(),
        username: auth.claims().username().to_string(),
    })
}

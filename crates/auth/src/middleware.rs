use super::*;
use rbp_core::ID;
use rbp_database::*;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::dev::Payload;
use actix_web::web;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio_postgres::Client;

/// Extractor for authenticated requests.
/// Validates JWT and checks session is not revoked.
pub struct Auth(pub Claims);

impl Auth {
    pub fn claims(&self) -> &Claims {
        &self.0
    }
    pub fn user(&self) -> ID<Member> {
        self.0.user()
    }
}

impl FromRequest for Auth {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let token_service = req.app_data::<web::Data<Crypto>>().cloned();
        let db = req.app_data::<web::Data<Arc<Client>>>().cloned();
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_owned());
        Box::pin(async move {
            let header = auth_header.ok_or_else(|| {
                actix_web::error::ErrorUnauthorized("missing authorization header")
            })?;
            let token = header.strip_prefix("Bearer ").ok_or_else(|| {
                actix_web::error::ErrorUnauthorized("invalid authorization format")
            })?;
            let service = token_service.ok_or_else(|| {
                actix_web::error::ErrorInternalServerError("token service not configured")
            })?;
            let claims = service
                .decode(token)
                .map_err(|_| actix_web::error::ErrorUnauthorized("invalid token"))?;
            if claims.expired() {
                return Err(actix_web::error::ErrorUnauthorized("token expired"));
            }
            let db = db.ok_or_else(|| {
                actix_web::error::ErrorInternalServerError("database not configured")
            })?;
            let row = db
                .query_opt(
                    const_format::concatcp!("SELECT revoked FROM ", SESSIONS, " WHERE id = $1"),
                    &[&claims.session().inner()],
                )
                .await
                .map_err(|_| actix_web::error::ErrorInternalServerError("database error"))?
                .ok_or_else(|| actix_web::error::ErrorUnauthorized("session not found"))?;
            let revoked: bool = row.get(0);
            if revoked {
                return Err(actix_web::error::ErrorUnauthorized("session revoked"));
            }
            Ok(Auth(claims))
        })
    }
}

/// Optional authentication extractor - does not fail if unauthenticated.
pub struct MaybeAuth(pub Option<Claims>);

impl MaybeAuth {
    pub fn claims(&self) -> Option<&Claims> {
        self.0.as_ref()
    }
    pub fn user(&self) -> Option<ID<Member>> {
        self.0.as_ref().map(|c| c.user())
    }
}

impl FromRequest for MaybeAuth {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let auth_future = Auth::from_request(req, payload);
        Box::pin(async move {
            match auth_future.await {
                Ok(Auth(claims)) => Ok(MaybeAuth(Some(claims))),
                Err(_) => Ok(MaybeAuth(None)),
            }
        })
    }
}

use super::*;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::dev::Payload;
use pokerkit::ID;
use std::future::Future;
use std::pin::Pin;

/// Optional authentication extractor - does not fail if unauthenticated.
pub struct MaybeAuth(pub Option<Claims>);
impl MaybeAuth {
    pub fn claims(&self) -> Option<&Claims> {
        self.0.as_ref()
    }

    pub fn user(&self) -> Option<ID<Member>> {
        self.0.as_ref().map(super::claims::Claims::user)
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

//! Authentication, sessions, and identity management.
//!
//! JWT-based auth ([`Crypto`], [`Claims`]) with Argon2 password hashing.
//! Identity is either a registered [`Member`] or an anonymous [`Lurker`]
//! spectator, unified as [`User`] once authenticated into a [`Session`].
#[cfg(feature = "server")]
mod maybe_auth;
#[cfg(feature = "server")]
pub use maybe_auth::*;
mod auth_response;
mod login_request;
mod user_info;
pub use auth_response::*;
pub use login_request::*;
pub use user_info::*;
mod claims;
mod crypto;
mod dto;
mod identity;
mod lurker;
mod member;
pub mod password;
mod session;

pub use claims::*;
pub use crypto::*;
pub use dto::*;
pub use identity::*;
pub use lurker::*;
pub use member::*;
pub use session::*;

#[cfg(feature = "server")]
mod repository;
#[cfg(feature = "server")]
pub use repository::*;

#[cfg(feature = "server")]
mod handlers;
#[cfg(feature = "server")]
mod middleware;
#[cfg(feature = "server")]
pub use handlers::*;
#[cfg(feature = "server")]
pub use middleware::*;

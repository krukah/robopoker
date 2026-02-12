//! Authentication, sessions, and identity management.
//!
//! JWT-based authentication with Argon2 password hashing. Supports both
//! registered members and anonymous lurkers for spectating games.
//!
//! ## Identity Types
//!
//! - [`Member`] — Registered user with credentials
//! - [`Lurker`] — Anonymous session for spectators
//! - [`User`] — Authenticated user (member or lurker)
//! - [`Session`] — Active login session with expiry
//!
//! ## Security
//!
//! - [`Crypto`] — JWT signing and verification
//! - [`Claims`] — JWT payload structure
//! - [`password`] — Argon2 hashing and verification
mod claims;
mod crypto;
mod identity;
mod lurker;
mod member;
pub mod password;
mod session;
mod dto;

pub use claims::*;
pub use crypto::*;
pub use dto::*;
pub use identity::*;
pub use lurker::*;
pub use member::*;
pub use session::*;

#[cfg(feature = "database")]
mod repository;
#[cfg(feature = "database")]
pub use repository::*;

#[cfg(feature = "server")]
mod handlers;
#[cfg(feature = "server")]
mod middleware;
#[cfg(feature = "server")]
pub use handlers::*;
#[cfg(feature = "server")]
pub use middleware::*;

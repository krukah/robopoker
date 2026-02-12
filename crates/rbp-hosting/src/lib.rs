//! WebSocket game hosting infrastructure.
//!
//! This module provides the server-side machinery for hosting live poker games
//! over WebSocket connections, managing room lifecycles and client sessions.
//!
//! ## Core Types
//!
//! - [`Casino`] — Central registry of active game rooms
//! - [`Client`] — WebSocket connection state for a connected player
//! - [`Handle`] — Room reference for client interactions
//!
//! ## HTTP Handlers
//!
//! The [`handlers`] submodule exposes actix-web routes for room management:
//! start, enter, and leave operations.
mod casino;
mod client;
mod handle;
pub mod handlers;

pub use casino::*;
pub use client::*;
pub use handle::*;

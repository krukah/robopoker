//! Unified backend server: abstraction/clustering queries ([`topology`]),
//! strategy lookups ([`strategy`]), hand-history + AIVAT analysis
//! ([`gameplay`]), WebSocket game hosting ([`hosting`]), and MCCFR training
//! observability ([`training`]).

mod metrics_middleware;
pub use metrics_middleware::*;
mod server;
pub use server::*;

pub mod gameplay;
pub mod hosting;
pub mod litmus;
mod metrics;
pub mod strategy;
pub mod topology;
pub mod training;

pub use gameplay::GameplayAPI;
pub use hosting::Casino;
pub use hosting::RoomHandle;
pub use strategy::StrategyAPI;
pub use topology::CLI;
pub use topology::Query;
pub use topology::TopologyAPI;
pub use training::TrainingAPI;

use actix_cors::Cors;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::middleware::Logger;
use actix_web::web;
use std::sync::Arc;
use tokio_postgres::Client;

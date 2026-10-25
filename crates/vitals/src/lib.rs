//! Telemetry init and metric registry for rbp binaries.
//!
//! `let _guard = vitals::init();` installs a `tracing` subscriber, routes `log::*` through it
//! via the `tracing-log` shim, configures the OTLP metrics exporter, and pre-registers the
//! workspace's metric handles under `metrics::*`.

mod bench;
pub use bench::*;
mod consts;
mod telemetry_guard;
pub(crate) use consts::*;
pub use telemetry_guard::*;

use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::Aggregation;
use opentelemetry_sdk::metrics::Instrument;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::metrics::Stream;
use opentelemetry_sdk::metrics::View;
use opentelemetry_sdk::metrics::new_view;
use opentelemetry_sdk::runtime;
use opentelemetry_semantic_conventions as semcov;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub mod metrics;

pub use opentelemetry::KeyValue;

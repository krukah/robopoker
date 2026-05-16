//! `Litmus` — high-level facade over the litmus pipeline.
//!
//! Generic over an `Ops` impl so the same facade serves the CLI binary
//! (direct DB) and HTTP handlers (server-side wrap of `StrategyAPI`).

use crate::ops::Ops;
use crate::{Catalog, Outcome, Scenarios, compose, evaluate, render};
use rbp_gameplay::{ApiGridUsage, ApiStatus};

pub struct Litmus<O: Ops> {
    ops: O,
}

impl<O: Ops> Litmus<O> {
    pub fn new(ops: O) -> Self {
        Self { ops }
    }

    pub fn ops(&self) -> &O {
        &self.ops
    }

    /// Run the full catalog: expand families, evaluate every concrete case.
    pub async fn run(&self, scenarios: &Scenarios) -> anyhow::Result<Vec<Outcome>> {
        let catalog = Catalog::new(scenarios);
        let cases = compose::resolve(scenarios)?;
        let mut outcomes = Vec::with_capacity(cases.len());
        for case in &cases {
            outcomes.push(evaluate::evaluate(&self.ops, &catalog, case).await);
        }
        Ok(outcomes)
    }

    /// Run the catalog and produce a markdown report.
    pub async fn report(&self, scenarios: &Scenarios, api_label: &str) -> anyhow::Result<String> {
        let outcomes = self.run(scenarios).await?;
        let status = self.ops.status().await.ok();
        let grid_usage = self.ops.grid_usage().await.ok();
        Ok(render::render(
            api_label,
            status.as_ref(),
            scenarios,
            &outcomes,
            grid_usage.as_deref(),
        ))
    }

    pub async fn status(&self) -> anyhow::Result<ApiStatus> {
        self.ops.status().await
    }

    pub async fn grid_usage(&self) -> anyhow::Result<Vec<ApiGridUsage>> {
        self.ops.grid_usage().await
    }
}

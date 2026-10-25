use super::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Wraps an [`ApiStrategy`] with provenance — which source produced
/// it, how many MCCFR iterations the solver ran (1 for blueprint),
/// and how long the request took on the server. Lets the client
/// surface variance across re-runs of stochastic solvers without
/// changing the policy-rendering path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSolved {
    pub kind: Kind,
    pub policy: ApiStrategy,
    pub iterations: u32,
    pub elapsed_ms: u32,
}

impl ApiSolved {
    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn policy(&self) -> &ApiStrategy {
        &self.policy
    }

    pub fn iterations(&self) -> u32 {
        self.iterations
    }

    pub fn elapsed_ms(&self) -> u32 {
        self.elapsed_ms
    }
}
impl ApiSolved {
    /// Constructor for the cheap blueprint path — DB lookup, 1 iter,
    /// `elapsed` from the request handler's timer.
    pub fn blueprint(policy: ApiStrategy, elapsed: Duration) -> Self {
        Self {
            kind: Kind::Blueprint,
            policy,
            iterations: 1,
            elapsed_ms: elapsed.as_millis() as u32,
        }
    }
}

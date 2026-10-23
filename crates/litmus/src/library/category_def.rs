use super::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CategoryDef {
    #[allow(dead_code)]
    pub description: String,
    /// Per-category default tolerance; overridden by case-level `expect`.
    pub default_expect: Option<Expect>,
}

impl CategoryDef {
    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn default_expect(&self) -> Option<&Expect> {
        self.default_expect.as_ref()
    }
}

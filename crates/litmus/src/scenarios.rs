use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level scenarios document.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Scenarios {
    /// Schema docs and field descriptions; runner ignores.
    #[serde(default, rename = "_meta")]
    #[allow(dead_code)]
    pub meta: serde_json::Value,

    pub hands: HashMap<String, HandDef>,

    /// Two-level: street ("preflop"/"flop"/"turn"/"river") → name → entry.
    pub histories: HashMap<String, HashMap<String, HistoryDef>>,

    pub categories: HashMap<String, CategoryDef>,

    #[serde(default)]
    pub tests: Vec<Case>,

    #[serde(default)]
    pub families: Vec<Family>,
}

impl Scenarios {
    pub fn meta(&self) -> &serde_json::Value {
        &self.meta
    }

    pub fn hands(&self) -> &HashMap<String, HandDef> {
        &self.hands
    }

    pub fn histories(&self) -> &HashMap<String, HashMap<String, HistoryDef>> {
        &self.histories
    }

    pub fn categories(&self) -> &HashMap<String, CategoryDef> {
        &self.categories
    }

    pub fn tests(&self) -> &[Case] {
        &self.tests
    }

    pub fn families(&self) -> &[Family] {
        &self.families
    }
}

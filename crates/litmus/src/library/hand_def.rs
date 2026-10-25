use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HandDef {
    /// Hole-card encoding parsed by `Observation::try_from`. e.g. "AcKd".
    pub cards: String,
    #[allow(dead_code)]
    pub name: String,
}

impl HandDef {
    pub fn cards(&self) -> &str {
        &self.cards
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

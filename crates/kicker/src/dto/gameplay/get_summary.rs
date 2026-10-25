use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetSummary {
    pub user: uuid::Uuid,
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub against: Option<uuid::Uuid>,
    #[serde(default)]
    pub stakes: Option<i16>,
    #[serde(default)]
    pub hero_human: bool,
    #[serde(default)]
    pub against_human: bool,
}

impl GetSummary {
    pub fn user(&self) -> uuid::Uuid {
        self.user
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }

    pub fn offset(&self) -> i64 {
        self.offset
    }

    pub fn against(&self) -> Option<uuid::Uuid> {
        self.against
    }

    pub fn stakes(&self) -> Option<i16> {
        self.stakes
    }

    pub fn hero_human(&self) -> bool {
        self.hero_human
    }

    pub fn against_human(&self) -> bool {
        self.against_human
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetHandRecap {
    pub id: uuid::Uuid,
}

impl GetHandRecap {
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }
}

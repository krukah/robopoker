use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Historical {
    #[allow(dead_code)]
    pub baseline_value: Option<f32>,
    pub context: Option<String>,
}

impl Historical {
    pub fn baseline_value(&self) -> Option<f32> {
        self.baseline_value
    }

    pub fn context(&self) -> Option<&String> {
        self.context.as_ref()
    }
}

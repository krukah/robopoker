use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TestKind {
    #[default]
    Single,
    PairDiff,
    Monotonic,
    Exists,
}

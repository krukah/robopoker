use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct Act {
    pub(crate) token: String,
    pub(crate) incr: String,
}

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct NewHand {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) token: Option<String>,
}

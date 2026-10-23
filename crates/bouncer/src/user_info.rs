use serde::Serialize;

#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
}

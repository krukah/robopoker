use super::*;
use rbp_core::ID;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub sid: uuid::Uuid,
    pub usr: String,
    pub iat: i64,
    pub exp: i64,
}

impl Claims {
    pub fn new(user: ID<Member>, session: ID<Session>, username: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_secs() as i64;
        Self {
            sub: user.inner(),
            sid: session.inner(),
            usr: username,
            iat: now,
            exp: now + Crypto::duration().as_secs() as i64,
        }
    }
    pub fn expired(&self) -> bool {
        self.exp
            < std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_secs() as i64
    }
    pub fn user(&self) -> ID<Member> {
        ID::from(self.sub)
    }
    pub fn session(&self) -> ID<Session> {
        ID::from(self.sid)
    }
    pub fn username(&self) -> &str {
        &self.usr
    }
}

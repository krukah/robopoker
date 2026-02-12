use super::*;

const ACCESS_TOKEN_DURATION: std::time::Duration = std::time::Duration::from_secs(15 * 60);

pub struct Crypto {
    encoding: jsonwebtoken::EncodingKey,
    decoding: jsonwebtoken::DecodingKey,
}

impl Crypto {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: jsonwebtoken::EncodingKey::from_secret(secret),
            decoding: jsonwebtoken::DecodingKey::from_secret(secret),
        }
    }
    pub fn from_env() -> Self {
        Self::new(
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| String::default())
                .as_bytes(),
        )
    }
    pub fn encode(&self, claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(&jsonwebtoken::Header::default(), claims, &self.encoding)
    }
    pub fn decode(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        jsonwebtoken::decode::<Claims>(token, &self.decoding, &jsonwebtoken::Validation::default())
            .map(|data| data.claims)
    }
    pub fn hash(token: &str) -> Vec<u8> {
        use sha2::Digest;
        sha2::Sha256::digest(token.as_bytes()).to_vec()
    }
    pub const fn duration() -> std::time::Duration {
        ACCESS_TOKEN_DURATION
    }
}

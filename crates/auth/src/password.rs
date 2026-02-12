use argon2::Argon2;
use argon2::PasswordHash;
use argon2::PasswordHasher;
use argon2::PasswordVerifier;
use argon2::password_hash::SaltString;

fn salt() -> SaltString {
    use rand::Rng;
    let ref mut bytes = [0u8; 16];
    rand::rng().fill(bytes);
    SaltString::encode_b64(bytes).expect("salt")
}

pub fn hash(password: &str) -> Result<String, argon2::password_hash::Error> {
    Argon2::default()
        .hash_password(password.as_bytes(), &salt())
        .map(|h| h.to_string())
}

pub fn verify(password: &str, hashword: &str) -> bool {
    PasswordHash::new(hashword)
        .ok()
        .as_ref()
        .map(|hash| {
            Argon2::default()
                .verify_password(password.as_bytes(), hash)
                .is_ok()
        })
        .unwrap_or(false)
}

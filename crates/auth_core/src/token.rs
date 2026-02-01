use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;

use contracts::UserWithPassword;
use crate::config::AuthConfig;
use crate::models::Claims;

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("failed to hash password: {}", e))?;
    Ok(hash.to_string())
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("invalid password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a JWT access token
pub fn generate_access_token(user: &UserWithPassword, config: &AuthConfig) -> anyhow::Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow::anyhow!("time error: {}", e))?
        .as_secs();

    let claims = Claims {
        sub: user.id.to_string(),
        iss: config.issuer.clone(),
        exp: now + config.token_ttl_seconds,
        iat: now,
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.to_string(),
        org_id: user.organisation_id.map(|id| id.to_string()),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("failed to encode token: {}", e))?;

    Ok(token)
}

/// Generate a random refresh token
pub fn generate_refresh_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.r#gen();
    hex::encode(bytes)
}

/// Hash a refresh token for storage
pub fn hash_refresh_token(token: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Validate and decode a JWT access token
pub fn validate_access_token(token: &str, config: &AuthConfig) -> anyhow::Result<Claims> {
    let mut validation = Validation::default();
    validation.set_issuer(&[&config.issuer]);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|e| anyhow::anyhow!("invalid token: {}", e))?;

    Ok(token_data.claims)
}

// Add hex encoding since we need it
mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

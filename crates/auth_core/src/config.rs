#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub issuer: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        let issuer = std::env::var("AUTH_ISSUER").unwrap_or_else(|_| "apisentinel".to_string());
        Self { issuer }
    }
}

use crate::config::AuthConfig;

pub fn run(config: &AuthConfig) {
    println!("auth service running, issuer: {}", config.issuer);
}

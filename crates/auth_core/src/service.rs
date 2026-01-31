use crate::config::AuthConfig;

pub async fn run(config: &AuthConfig) -> anyhow::Result<()> {
    println!("auth service running, issuer: {}", config.issuer);
    Ok(())
}

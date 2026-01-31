pub async fn start() {
    if let Err(err) = auth_core::run().await {
        eprintln!("auth module error: {err}");
    }
}

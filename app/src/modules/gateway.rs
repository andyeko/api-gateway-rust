pub async fn start() {
    if let Err(err) = gateway_core::run().await {
        eprintln!("gateway module error: {err}");
    }
}

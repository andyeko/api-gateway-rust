pub async fn start() {
    if let Err(err) = admin_core::run().await {
        eprintln!("admin module error: {err}");
    }
}

mod modules;

#[tokio::main]
async fn main() {
    println!("apisentinel app");

    let mut handles = Vec::new();

    // Gateway is the main entry point - it handles routing and can embed other modules
    #[cfg(feature = "gateway")]
    {
        handles.push(tokio::spawn(async {
            modules::gateway::start().await;
        }));
    }

    // Auth runs as a separate service (on its own port)
    #[cfg(feature = "auth")]
    {
        handles.push(tokio::spawn(async {
            modules::auth::start().await;
        }));
    }

    // Admin runs standalone only when gateway is not enabled
    // (otherwise admin is embedded in gateway or proxied by gateway)
    #[cfg(all(feature = "admin", not(feature = "gateway")))]
    {
        handles.push(tokio::spawn(async {
            modules::admin::start().await;
        }));
    }

    if handles.is_empty() {
        eprintln!("No modules enabled. Use --features to enable: gateway, auth, admin");
        return;
    }

    for handle in handles {
        let _ = handle.await;
    }
}

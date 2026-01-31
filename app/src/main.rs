mod modules;

#[tokio::main]
async fn main() {
    println!("apisentinel app");

    let mut handles = Vec::new();

    #[cfg(feature = "gateway")]
    {
        handles.push(tokio::spawn(async {
            modules::gateway::start().await;
        }));
    }

    #[cfg(feature = "auth")]
    {
        handles.push(tokio::spawn(async {
            modules::auth::start().await;
        }));
    }

    #[cfg(all(feature = "admin", not(feature = "gateway")))]
    {
        handles.push(tokio::spawn(async {
            modules::admin::start().await;
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }
}

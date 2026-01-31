mod modules;

#[tokio::main]
async fn main() {
    println!("apisentinel app");

    let mut handles = Vec::new();
    let mut any = false;

    #[cfg(feature = "gateway")]
    {
        any |= true;
        handles.push(tokio::spawn(async {
            modules::gateway::start().await;
        }));
    }

    #[cfg(feature = "auth")]
    {
        any |= true;
        handles.push(tokio::spawn(async {
            modules::auth::start().await;
        }));
    }

    #[cfg(all(feature = "admin", not(feature = "gateway")))]
    {
        any |= true;
        handles.push(tokio::spawn(async {
            modules::admin::start().await;
        }));
    }

    if !any {
        println!("No modules enabled. Enable features: gateway, auth, admin.");
        return;
    }

    for handle in handles {
        let _ = handle.await;
    }
}

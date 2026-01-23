mod modules;

fn main() {
    println!("apisentinel app");

    let mut any = false;

    #[cfg(feature = "gateway")]
    {
        any = true;
        modules::gateway::start();
    }

    #[cfg(feature = "auth")]
    {
        any = true;
        modules::auth::start();
    }

    #[cfg(feature = "admin")]
    {
        any = true;
        modules::admin::start();
    }

    if !any {
        println!("No modules enabled. Enable features: gateway, auth, admin.");
    }
}

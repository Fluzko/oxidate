use std::env;

fn main() {
    // Load .env file if it exists
    dotenv::dotenv().ok();

    // Read credentials from environment and set them as compile-time env vars
    if let Ok(client_id) = env::var("GOOGLE_CLIENT_ID") {
        println!("cargo:rustc-env=GOOGLE_CLIENT_ID={}", client_id);
    } else {
        println!("cargo:warning=GOOGLE_CLIENT_ID not found in environment");
    }

    if let Ok(client_secret) = env::var("GOOGLE_CLIENT_SECRET") {
        println!("cargo:rustc-env=GOOGLE_CLIENT_SECRET={}", client_secret);
    } else {
        println!("cargo:warning=GOOGLE_CLIENT_SECRET not found in environment");
    }

    // Rerun build script if .env changes
    println!("cargo:rerun-if-changed=.env");
}

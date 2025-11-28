mod auth;
mod cli;

use cli::Cli;
use auth::Tokens;

#[tokio::main]
async fn main() {
    let args = Cli::parse_args();

    if args.is_logout() {
        handle_logout();
        return;
    }

    match auth::authenticate().await {
        Ok(_tokens) => {
            println!("Authentication successful!");
            println!("Application ready.");
        }
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_logout() {
    match Tokens::delete() {
        Ok(_) => println!("Successfully logged out. Credentials deleted."),
        Err(e) => eprintln!("Failed to delete credentials: {}", e),
    }
}

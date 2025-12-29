mod auth;
mod cli;
mod calendar;
mod tui;

use cli::Cli;
use auth::Tokens;
use calendar::client::CalendarClient;

#[tokio::main]
async fn main() {
    let args = Cli::parse_args();

    if args.is_logout() {
        handle_logout();
        return;
    }

    // Authenticate first
    let tokens = match auth::authenticate().await {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            std::process::exit(1);
        }
    };

    // Handle login command
    if args.is_login() {
        println!("Authentication successful!");
        println!("Your credentials have been saved.");
        println!("\nRun without arguments to launch the calendar:");
        println!("  cargo run");
        return;
    }

    // Default: Launch TUI
    match CalendarClient::new(tokens) {
        Ok(client) => {
            if let Err(e) = tui::run_tui(client) {
                eprintln!("TUI error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to create calendar client: {}", e);
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

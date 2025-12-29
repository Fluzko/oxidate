use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ai-rust-calendar")]
#[command(about = "A TUI calendar application with Google Calendar integration", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Logout and delete stored credentials
    #[arg(long)]
    pub logout: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Authenticate with Google Calendar
    Login,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn is_logout(&self) -> bool {
        self.logout
    }

    pub fn is_login(&self) -> bool {
        matches!(self.command, Some(Command::Login))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default_has_no_logout() {
        let cli = Cli::parse_from(&["ai-rust-calendar"]);
        assert!(!cli.is_logout());
        assert!(!cli.logout);
    }

    #[test]
    fn test_cli_logout_flag() {
        let cli = Cli::parse_from(&["ai-rust-calendar", "--logout"]);
        assert!(cli.is_logout());
        assert!(cli.logout);
    }

    #[test]
    fn test_cli_is_logout_method() {
        let cli_no_logout = Cli::parse_from(&["ai-rust-calendar"]);
        assert!(!cli_no_logout.is_logout());

        let cli_with_logout = Cli::parse_from(&["ai-rust-calendar", "--logout"]);
        assert!(cli_with_logout.is_logout());
    }

    #[test]
    fn test_cli_login_command() {
        let cli = Cli::parse_from(&["ai-rust-calendar", "login"]);
        assert!(cli.is_login());
        assert!(!cli.is_logout());
    }

    #[test]
    fn test_cli_default_no_command() {
        let cli = Cli::parse_from(&["ai-rust-calendar"]);
        assert!(!cli.is_login());
        assert!(cli.command.is_none());
    }
}

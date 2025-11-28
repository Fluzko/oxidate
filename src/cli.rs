use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ai-rust-calendar")]
#[command(about = "A TUI calendar application with Google Calendar integration", long_about = None)]
pub struct Cli {
    /// Logout and delete stored credentials
    #[arg(long)]
    pub logout: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn is_logout(&self) -> bool {
        self.logout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default_has_no_logout() {
        // When no args are provided, logout should be false
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
}

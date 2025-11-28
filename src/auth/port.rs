use anyhow::{Context, Result};

pub struct PortSelector;

impl PortSelector {
    pub fn find_available() -> Result<u16> {
        portpicker::pick_unused_port()
            .context("No available ports found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_available_port_returns_some_port() {
        let result = PortSelector::find_available();
        assert!(result.is_ok());

        let port = result.unwrap();
        assert!(port > 0);
    }

    #[test]
    fn test_returned_port_is_valid_range() {
        let port = PortSelector::find_available().unwrap();
        // Valid port range is 1-65535, usually unprivileged ports are >= 1024
        assert!(port >= 1024);
    }

    #[test]
    fn test_multiple_calls_can_return_different_ports() {
        let port1 = PortSelector::find_available().unwrap();
        let port2 = PortSelector::find_available().unwrap();

        // Both should be valid ports
        assert!(port1 > 0);
        assert!(port2 > 0);
        // They might be the same or different, but both should be valid
    }
}

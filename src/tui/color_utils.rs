use ratatui::style::Color;

/// Parse hex color string (#RRGGBB) to ratatui Color
/// Returns None if invalid format
pub fn parse_hex_color(hex: &str) -> Option<Color> {
    // Must start with #
    if !hex.starts_with('#') {
        return None;
    }

    // Must be exactly 7 characters (#RRGGBB)
    if hex.len() != 7 {
        return None;
    }

    // Parse RGB components
    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

/// Default gray color for events without calendar color
pub fn default_event_color() -> Color {
    Color::Gray
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_valid() {
        // Test valid hex colors
        assert_eq!(parse_hex_color("#FF0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_hex_color("#00FF00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("#0088aa"), Some(Color::Rgb(0, 136, 170)));
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        // Invalid format tests
        assert_eq!(parse_hex_color("invalid"), None);
        assert_eq!(parse_hex_color("#GG0000"), None); // Invalid hex chars
        assert_eq!(parse_hex_color("#FF"), None); // Too short
        assert_eq!(parse_hex_color("FF0000"), None); // Missing #
    }

    #[test]
    fn test_parse_hex_color_empty() {
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn test_default_event_color_is_gray() {
        assert_eq!(default_event_color(), Color::Gray);
    }
}

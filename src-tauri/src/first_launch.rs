pub fn should_show_window_on_startup(has_credentials: bool, platform: &str) -> bool {
    platform == "windows" && !has_credentials
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_no_credentials_shows_window() {
        assert!(should_show_window_on_startup(false, "windows"));
    }

    #[test]
    fn windows_with_credentials_hides_window() {
        assert!(!should_show_window_on_startup(true, "windows"));
    }

    #[test]
    fn macos_no_credentials_hides_window() {
        assert!(!should_show_window_on_startup(false, "macos"));
    }

    #[test]
    fn macos_with_credentials_hides_window() {
        assert!(!should_show_window_on_startup(true, "macos"));
    }
}

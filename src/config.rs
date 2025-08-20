use std::fmt;
use std::env;
use crate::screen_factory::ScreenFactory;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub interval_seconds: u64,
    pub screen_duration_secs: u64,
    pub enabled_screens: Vec<String>,
    pub daemon_mode: bool,
    pub clear_only: bool,
    pub multiplexer: MultiplexerConfig,
}

#[derive(Debug, Clone)]
pub struct MultiplexerConfig {
    pub enabled: bool,
    pub channel: u8,
    pub address: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 5,
            screen_duration_secs: 10,
            enabled_screens: vec!["overview".to_string()],
            daemon_mode: false,
            clear_only: false,
            multiplexer: MultiplexerConfig::default(),
        }
    }
}

impl Default for MultiplexerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            channel: 0,
            address: 0x70,
        }
    }
}

impl AppConfig {
    pub fn enabled_screens_as_str_refs(&self) -> Vec<&str> {
        self.enabled_screens.iter().map(|s| s.as_str()).collect()
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();
        config.apply_env_vars();
        config
    }

    pub fn apply_env_vars(&mut self) {
        // Interval
        if let Ok(interval_str) = env::var("INFO_DISPLAY_INTERVAL") {
            if let Ok(interval) = interval_str.parse::<u64>() {
                if interval > 0 {
                    self.interval_seconds = interval;
                }
            }
        }

        // Screen duration
        if let Ok(duration_str) = env::var("INFO_DISPLAY_SCREEN_DURATION") {
            if let Ok(duration) = duration_str.parse::<u64>() {
                if duration > 0 {
                    self.screen_duration_secs = duration;
                }
            }
        }

        // Enabled screens
        if let Ok(screens_str) = env::var("INFO_DISPLAY_SCREENS") {
            let screens: Vec<String> = screens_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && ScreenFactory::validate_screen_type(s))
                .collect();
            if !screens.is_empty() {
                self.enabled_screens = screens;
            }
        }

        // Daemon mode
        if let Ok(daemon_str) = env::var("INFO_DISPLAY_DAEMON") {
            self.daemon_mode = daemon_str.to_lowercase() == "true" || daemon_str == "1";
        }

        // Multiplexer config
        if let Ok(mux_enabled_str) = env::var("INFO_DISPLAY_MUX_ENABLED") {
            self.multiplexer.enabled = mux_enabled_str.to_lowercase() == "true" || mux_enabled_str == "1";
        }

        if let Ok(mux_channel_str) = env::var("INFO_DISPLAY_MUX_CHANNEL") {
            if let Ok(channel) = mux_channel_str.parse::<u8>() {
                if channel <= 7 {
                    self.multiplexer.channel = channel;
                }
            }
        }

        if let Ok(mux_addr_str) = env::var("INFO_DISPLAY_MUX_ADDRESS") {
            if let Ok(address) = u8::from_str_radix(&mux_addr_str.trim_start_matches("0x"), 16) {
                self.multiplexer.address = address;
            } else if let Ok(address) = mux_addr_str.parse::<u8>() {
                self.multiplexer.address = address;
            }
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate interval
        if self.interval_seconds == 0 {
            return Err(ConfigError::InvalidInterval);
        }

        // Validate screen duration
        if self.screen_duration_secs == 0 {
            return Err(ConfigError::InvalidScreenDuration);
        }

        // Validate screens
        if self.enabled_screens.is_empty() {
            return Err(ConfigError::NoScreensEnabled);
        }

        for screen in &self.enabled_screens {
            if !ScreenFactory::validate_screen_type(screen) {
                return Err(ConfigError::InvalidScreen(screen.clone()));
            }
        }

        // Validate multiplexer config
        if self.multiplexer.channel > 7 {
            return Err(ConfigError::InvalidMultiplexerChannel(self.multiplexer.channel));
        }

        Ok(())
    }

    pub fn add_screen(&mut self, screen: &str) {
        // Replace default overview with first specific screen
        if self.enabled_screens == vec!["overview"] && screen != "overview" {
            self.enabled_screens = vec![screen.to_string()];
        } else if !self.enabled_screens.contains(&screen.to_string()) {
            self.enabled_screens.push(screen.to_string());
        }
    }

    pub fn enable_multiplexer(&mut self) {
        self.multiplexer.enabled = true;
    }

    pub fn set_multiplexer_channel(&mut self, channel: u8) -> Result<(), ConfigError> {
        if channel > 7 {
            return Err(ConfigError::InvalidMultiplexerChannel(channel));
        }
        self.multiplexer.channel = channel;
        self.multiplexer.enabled = true;
        Ok(())
    }

    pub fn set_multiplexer_address(&mut self, address: u8) {
        self.multiplexer.address = address;
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidInterval,
    InvalidScreenDuration,
    NoScreensEnabled,
    InvalidScreen(String),
    InvalidMultiplexerChannel(u8),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidInterval => write!(f, "Update interval must be greater than 0"),
            ConfigError::InvalidScreenDuration => write!(f, "Screen duration must be greater than 0"),
            ConfigError::NoScreensEnabled => write!(f, "At least one screen must be enabled"),
            ConfigError::InvalidScreen(screen) => write!(f, "Invalid screen type: {}", screen),
            ConfigError::InvalidMultiplexerChannel(channel) => write!(f, "Multiplexer channel must be 0-7, got: {}", channel),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.interval_seconds, 5);
        assert_eq!(config.enabled_screens, vec!["overview"]);
        assert!(!config.multiplexer.enabled);
    }

    #[test]
    fn test_add_screen_replaces_overview() {
        let mut config = AppConfig::default();
        config.add_screen("network");
        assert_eq!(config.enabled_screens, vec!["network"]);
        
        config.add_screen("system");
        assert_eq!(config.enabled_screens, vec!["network", "system"]);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_screen() {
        let mut config = AppConfig::default();
        config.enabled_screens = vec!["invalid".to_string()];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_channel() {
        let mut config = AppConfig::default();
        config.multiplexer.channel = 8;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_var_interval() {
        unsafe {
            env::set_var("INFO_DISPLAY_INTERVAL", "10");
        }
        let config = AppConfig::from_env();
        assert_eq!(config.interval_seconds, 10);
        unsafe {
            env::remove_var("INFO_DISPLAY_INTERVAL");
        }
    }

    #[test]
    fn test_env_var_screens() {
        unsafe {
            env::set_var("INFO_DISPLAY_SCREENS", "network,system,storage");
        }
        let config = AppConfig::from_env();
        assert_eq!(config.enabled_screens, vec!["network", "system", "storage"]);
        unsafe {
            env::remove_var("INFO_DISPLAY_SCREENS");
        }
    }

    #[test]
    fn test_env_var_daemon_mode() {
        unsafe {
            env::set_var("INFO_DISPLAY_DAEMON", "true");
        }
        let config = AppConfig::from_env();
        assert!(config.daemon_mode);
        unsafe {
            env::remove_var("INFO_DISPLAY_DAEMON");
        }
    }

    #[test]
    fn test_env_var_multiplexer() {
        unsafe {
            env::set_var("INFO_DISPLAY_MUX_ENABLED", "true");
            env::set_var("INFO_DISPLAY_MUX_CHANNEL", "3");
            env::set_var("INFO_DISPLAY_MUX_ADDRESS", "0x71");
        }
        let config = AppConfig::from_env();
        assert!(config.multiplexer.enabled);
        assert_eq!(config.multiplexer.channel, 3);
        assert_eq!(config.multiplexer.address, 0x71);
        unsafe {
            env::remove_var("INFO_DISPLAY_MUX_ENABLED");
            env::remove_var("INFO_DISPLAY_MUX_CHANNEL");
            env::remove_var("INFO_DISPLAY_MUX_ADDRESS");
        }
    }
}
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use crate::screens::*;

pub struct ScreenFactory;

impl ScreenFactory {
    pub fn create_screen(screen_type: &str) -> Result<Box<dyn Screen>> {
        match screen_type {
            "network" => Ok(Box::new(NetworkScreen)),
            "system" => Ok(Box::new(SystemScreen)),
            "storage" => Ok(Box::new(StorageScreen)),
            "hardware" => Ok(Box::new(HardwareScreen)),
            "temperature" => Ok(Box::new(TemperatureScreen)),
            "gpio" => Ok(Box::new(GPIOScreen)),
            "overview" => Ok(Box::new(OverviewScreen)),
            _ => Err(anyhow!("Unknown screen type: {}", screen_type)),
        }
    }

    pub fn create_screens(screen_types: &[&str]) -> Result<Vec<Box<dyn Screen>>> {
        screen_types.iter()
            .map(|&screen_type| Self::create_screen(screen_type))
            .collect()
    }

    pub fn get_available_screens() -> Vec<&'static str> {
        vec!["network", "system", "storage", "hardware", "temperature", "gpio", "overview"]
    }

    pub fn get_screen_descriptions() -> HashMap<&'static str, &'static str> {
        let mut descriptions = HashMap::new();
        descriptions.insert("network", "Display hostname, domain, IP address, and MAC address");
        descriptions.insert("system", "Show CPU temperature, uptime, and boot partition");
        descriptions.insert("storage", "Display memory usage and disk usage information");
        descriptions.insert("hardware", "Show Pi model, serial number, and firmware version");
        descriptions.insert("temperature", "Display CPU/GPU temperatures, frequency, and throttling status");
        descriptions.insert("gpio", "Show I2C devices, GPIO states, SPI devices, and 1-Wire sensors");
        descriptions.insert("overview", "Combined view with all essential system information");
        descriptions
    }

    pub fn validate_screen_type(screen_type: &str) -> bool {
        Self::get_available_screens().contains(&screen_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_screen() {
        let screen = ScreenFactory::create_screen("network");
        assert!(screen.is_ok());
        assert_eq!(screen.unwrap().name(), "network");
    }

    #[test]
    fn test_create_invalid_screen() {
        let screen = ScreenFactory::create_screen("invalid");
        assert!(screen.is_err());
    }

    #[test]
    fn test_create_multiple_screens() {
        let screens = ScreenFactory::create_screens(&["network", "system"]);
        assert!(screens.is_ok());
        let screens = screens.unwrap();
        assert_eq!(screens.len(), 2);
        assert_eq!(screens[0].name(), "network");
        assert_eq!(screens[1].name(), "system");
    }

    #[test]
    fn test_validate_screen_types() {
        assert!(ScreenFactory::validate_screen_type("network"));
        assert!(ScreenFactory::validate_screen_type("overview"));
        assert!(!ScreenFactory::validate_screen_type("invalid"));
    }

    #[test]
    fn test_get_available_screens() {
        let screens = ScreenFactory::get_available_screens();
        assert!(screens.contains(&"network"));
        assert!(screens.contains(&"overview"));
        assert_eq!(screens.len(), 7);
    }
}
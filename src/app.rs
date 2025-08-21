use std::thread;
use std::time::Duration;
use sysinfo::System;
use daemonize::Daemonize;

use crate::cli::CliParser;
use crate::config::AppConfig;
use crate::errors::{AppError, Result};
use crate::screen_manager::ScreenManager;
use crate::display::DisplayManager;

pub struct Application {
    config: AppConfig,
    display_manager: Option<DisplayManager>,
    screen_manager: Option<ScreenManager>,
}

impl Application {
    pub fn new() -> Result<Self> {
        let config = CliParser::parse()?;
        Ok(Self {
            config,
            display_manager: None,
            screen_manager: None,
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        // Handle daemon mode
        if self.config.daemon_mode {
            self.start_daemon()?;
        }

        // Handle clear-only mode
        if self.config.clear_only {
            DisplayManager::clear_display(
                self.config.multiplexer.enabled,
                self.config.multiplexer.channel,
                self.config.multiplexer.address,
            ).map_err(|e| AppError::display_init(&format!("Failed to clear display: {}", e)))?;
            return Ok(());
        }

        // Initialize display
        let display_manager = DisplayManager::new(
            self.config.multiplexer.enabled,
            self.config.multiplexer.channel,
            self.config.multiplexer.address,
        ).map_err(|e| AppError::display_init(&format!("Failed to initialize display: {}", e)))?;

        self.display_manager = Some(display_manager);

        // Create screen manager with enabled screens
        let screen_manager = ScreenManager::new(
            self.config.enabled_screens_as_str_refs(),
            self.config.screen_duration_secs,
        ).map_err(|e| AppError::system_info(&format!("Failed to create screen manager: {}", e)))?;

        self.screen_manager = Some(screen_manager);

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        if self.config.clear_only {
            return Ok(());
        }

        let display_manager = self.display_manager.as_mut()
            .ok_or_else(|| AppError::system_info("Display manager not initialized"))?;
        let screen_manager = self.screen_manager.as_mut()
            .ok_or_else(|| AppError::system_info("Screen manager not initialized"))?;

        Application::run_display_loop(&self.config, display_manager, screen_manager)
    }

    fn start_daemon(&self) -> Result<()> {
        let daemonize = Daemonize::new()
            .pid_file("/tmp/info_display.pid")
            .chown_pid_file(true)
            .working_directory("/tmp");

        daemonize.start()
            .map_err(|e| AppError::daemon(&format!("Failed to start daemon: {}", e)))?;
        
        Ok(())
    }

    fn run_display_loop(
        config: &AppConfig,
        display_manager: &mut DisplayManager,
        screen_manager: &mut ScreenManager,
    ) -> Result<()> {
        loop {
            // Initialize system info
            let mut sys = System::new_all();
            sys.refresh_all();

            // Check if we need to switch screens
            if screen_manager.should_switch_screen() {
                screen_manager.next_screen();
            }

            // Render current screen
            let (title, content) = screen_manager.render_current_screen(&sys)
                .map_err(|e| AppError::system_info(&format!("Failed to render screen: {}", e)))?;
                
            display_manager.render_content(&title, &content)
                .map_err(|e| AppError::display_init(&format!("Failed to render to display: {}", e)))?;

            // Wait for next update
            thread::sleep(Duration::from_secs(config.interval_seconds));
        }
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self {
                config: AppConfig::default(),
                display_manager: None,
                screen_manager: None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_default() {
        let app = Application::default();
        assert_eq!(app.config.interval_seconds, 5);
        assert!(!app.config.daemon_mode);
    }

    #[test]
    fn test_application_config_access() {
        let app = Application::default();
        let config = app.config();
        assert_eq!(config.interval_seconds, 5);
    }
}
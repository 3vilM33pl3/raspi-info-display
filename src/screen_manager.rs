use std::time::Duration;
use sysinfo::System;
use anyhow::Result;
use crate::screens::Screen;
use crate::screen_factory::ScreenFactory;

// Screen manager to handle cycling through screens
pub struct ScreenManager {
    screens: Vec<Box<dyn Screen>>,
    current_index: usize,
    last_switch_time: std::time::Instant,
    screen_duration: Duration,
}

impl ScreenManager {
    pub fn new(enabled_screen_names: Vec<&str>, screen_duration_secs: u64) -> Result<Self> {
        let screens = ScreenFactory::create_screens(&enabled_screen_names)
            .unwrap_or_else(|_| {
                // Fallback to overview screen if there's an error
                vec![ScreenFactory::create_screen("overview").unwrap()]
            });
        
        Ok(Self {
            screens,
            current_index: 0,
            last_switch_time: std::time::Instant::now(),
            screen_duration: Duration::from_secs(screen_duration_secs),
        })
    }
    
    pub fn should_switch_screen(&self) -> bool {
        self.screens.len() > 1 && self.last_switch_time.elapsed() >= self.screen_duration
    }
    
    pub fn next_screen(&mut self) {
        if self.screens.len() > 1 {
            self.current_index = (self.current_index + 1) % self.screens.len();
            self.last_switch_time = std::time::Instant::now();
        }
    }
    
    pub fn current_screen(&self) -> Option<&dyn Screen> {
        self.screens.get(self.current_index).map(|s| s.as_ref())
    }
    
    pub fn render_current_screen(&self, sys: &System) -> Result<(String, String)> {
        if let Some(screen) = self.current_screen() {
            let title = screen.title()?;
            let content = screen.render(sys)?;
            Ok((title, content))
        } else {
            Ok(("No Screen".to_string(), "No screens enabled".to_string()))
        }
    }
}
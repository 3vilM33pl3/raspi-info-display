mod tca9548a;
mod system_info;
mod screens;
mod screen_factory;
mod screen_manager;
mod display;
mod cli;
mod config;
mod errors;
mod app;

use errors::Result;
use app::Application;

fn main() -> Result<()> {
    match run() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run() -> Result<()> {
    let mut app = Application::new()?;
    app.initialize()?;
    app.run()
}


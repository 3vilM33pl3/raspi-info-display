use std::env;
use crate::config::{AppConfig, ConfigError};

pub struct CliParser;

impl CliParser {
    pub fn parse() -> Result<AppConfig, ConfigError> {
        let args: Vec<String> = env::args().collect();
        let mut config = AppConfig::from_env(); // Start with environment variables
        
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--clear" => config.clear_only = true,
                "--daemon" | "-d" => config.daemon_mode = true,
                "--interval" | "-i" => {
                    if let Some(value) = Self::get_next_arg(&args, i) {
                        if let Ok(seconds) = value.parse::<u64>() {
                            config.interval_seconds = seconds;
                            i += 1;
                        }
                    }
                }
                "--screen-duration" | "-s" => {
                    if let Some(value) = Self::get_next_arg(&args, i) {
                        if let Ok(seconds) = value.parse::<u64>() {
                            config.screen_duration_secs = seconds;
                            i += 1;
                        }
                    }
                }
                "--screens" => {
                    if let Some(value) = Self::get_next_arg(&args, i) {
                        config.enabled_screens = value.split(',').map(|s| s.to_string()).collect();
                        i += 1;
                    }
                }
                "--network" => config.add_screen("network"),
                "--system" => config.add_screen("system"),
                "--storage" => config.add_screen("storage"),
                "--hardware" => config.add_screen("hardware"),
                "--temperature" => config.add_screen("temperature"),
                "--gpio" => config.add_screen("gpio"),
                "--overview" => config.add_screen("overview"),
                "--mux" => config.enable_multiplexer(),
                "--mux-channel" => {
                    if let Some(value) = Self::get_next_arg(&args, i) {
                        if let Ok(channel) = value.parse::<u8>() {
                            config.set_multiplexer_channel(channel)?;
                            i += 1;
                        }
                    }
                }
                "--mux-address" => {
                    if let Some(value) = Self::get_next_arg(&args, i) {
                        if let Ok(addr) = u8::from_str_radix(value.trim_start_matches("0x"), 16) {
                            config.set_multiplexer_address(addr);
                            i += 1;
                        }
                    }
                }
                "--help" | "-h" => {
                    Self::print_help(&args[0]);
                    std::process::exit(0);
                }
                arg if arg.starts_with("--interval=") => {
                    if let Some(value) = arg.strip_prefix("--interval=") {
                        if let Ok(seconds) = value.parse::<u64>() {
                            config.interval_seconds = seconds;
                        }
                    }
                }
                arg if arg.starts_with("--screen-duration=") => {
                    if let Some(value) = arg.strip_prefix("--screen-duration=") {
                        if let Ok(seconds) = value.parse::<u64>() {
                            config.screen_duration_secs = seconds;
                        }
                    }
                }
                arg if arg.starts_with("--screens=") => {
                    if let Some(value) = arg.strip_prefix("--screens=") {
                        config.enabled_screens = value.split(',').map(|s| s.to_string()).collect();
                    }
                }
                arg if arg.starts_with("--mux-channel=") => {
                    if let Some(value) = arg.strip_prefix("--mux-channel=") {
                        if let Ok(channel) = value.parse::<u8>() {
                            config.set_multiplexer_channel(channel)?;
                        }
                    }
                }
                arg if arg.starts_with("--mux-address=") => {
                    if let Some(value) = arg.strip_prefix("--mux-address=") {
                        if let Ok(addr) = u8::from_str_radix(value.trim_start_matches("0x"), 16) {
                            config.set_multiplexer_address(addr);
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
        
        // Validate the configuration before returning
        config.validate()?;
        Ok(config)
    }
    
    fn get_next_arg(args: &[String], index: usize) -> Option<&String> {
        if index + 1 < args.len() {
            Some(&args[index + 1])
        } else {
            None
        }
    }
    
    fn print_help(program_name: &str) {
        println!("Info Display - System information on OLED display");
        println!("Usage: {} [OPTIONS]", program_name);
        println!();
        println!("Options:");
        println!("  --clear              Clear display and exit");
        println!("  --daemon, -d         Run as daemon");
        println!("  --interval, -i <N>   Update interval in seconds (default: 5)");
        println!("  --screen-duration, -s <N>  Duration each screen is shown (default: 10)");
        println!("  --screens <list>     Comma-separated list of screens (network,system,storage,hardware,temperature,gpio,overview)");
        println!("  --network            Enable network screen");
        println!("  --system             Enable system screen");
        println!("  --storage            Enable storage screen");
        println!("  --hardware           Enable hardware screen");
        println!("  --temperature        Enable temperature screen");
        println!("  --gpio               Enable GPIO/sensor screen");
        println!("  --overview           Enable overview screen (default)");
        println!("  --mux                Use TCA9548A I2C multiplexer");
        println!("  --mux-channel <0-7>  Select multiplexer channel (default: 0)");
        println!("  --mux-address <addr> Set multiplexer I2C address (default: 0x70)");
        println!("  --help, -h           Show this help message");
        println!();
        println!("Environment Variables:");
        println!("  INFO_DISPLAY_INTERVAL=<seconds>         Update interval");
        println!("  INFO_DISPLAY_SCREEN_DURATION=<seconds>  Screen duration");
        println!("  INFO_DISPLAY_SCREENS=<screen1,screen2>  Enabled screens");
        println!("  INFO_DISPLAY_DAEMON=<true|false>        Daemon mode");
        println!("  INFO_DISPLAY_MUX_ENABLED=<true|false>   Enable multiplexer");
        println!("  INFO_DISPLAY_MUX_CHANNEL=<0-7>          Multiplexer channel");
        println!("  INFO_DISPLAY_MUX_ADDRESS=<0xNN>         Multiplexer address");
        println!();
        println!("Examples:");
        println!("  {} --network --system                    # Show network and system screens", program_name);
        println!("  {} --screens network,system,gpio         # Same as above plus GPIO info", program_name);
        println!("  {} --screen-duration 15 --overview       # Show overview screen for 15s each", program_name);
        println!("  {} --mux --mux-channel 3                 # Use multiplexer channel 3", program_name);
        println!("  INFO_DISPLAY_SCREENS=network,system {} # Set screens via environment", program_name);
    }
}
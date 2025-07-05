use embedded_graphics::{
    mono_font::{ascii::{FONT_6X10, FONT_6X12}, iso_8859_16::FONT_7X13_BOLD, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use linux_embedded_hal::I2cdev;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use anyhow::Result;
use get_if_addrs::get_if_addrs;
use sysinfo::{System, Disks};
use std::fs;
use std::env;
use std::thread;
use std::time::Duration;
use daemonize::Daemonize;

// Screen trait for modular display screens
trait Screen {
    fn name(&self) -> &'static str;
    fn title(&self) -> Result<String> {
        Ok(self.name().to_string())
    }
    fn render(&self, sys: &System) -> Result<String>;
}

// Network information screen
struct NetworkScreen;

impl Screen for NetworkScreen {
    fn name(&self) -> &'static str {
        "network"
    }
    
    fn render(&self, _sys: &System) -> Result<String> {
        let hostname = hostname::get()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let domain = get_domain();
        let ip_address = get_ip_address()?;
        
        Ok(format!(
            "{}.{}\n{}",
            hostname, domain, ip_address
        ))
    }
}

// System information screen
struct SystemScreen;

impl Screen for SystemScreen {
    fn name(&self) -> &'static str {
        "system"
    }
    
    fn render(&self, _sys: &System) -> Result<String> {
        let cpu_temp = get_cpu_temp().unwrap_or_else(|_| "N/A".to_string());
        let uptime = get_uptime();
        
        Ok(format!(
            "CPU: {}\nUptime: {}",
            cpu_temp, uptime
        ))
    }
}

// Memory and storage screen
struct StorageScreen;

impl Screen for StorageScreen {
    fn name(&self) -> &'static str {
        "storage"
    }
    
    fn render(&self, sys: &System) -> Result<String> {
        let memory_info = get_memory_info(sys);
        let disk_usage = get_disk_usage();
        
        Ok(format!(
            "Memory: {}\nDisk: {}",
            memory_info, disk_usage
        ))
    }
}

// Combined overview screen (original layout)
struct OverviewScreen;

impl Screen for OverviewScreen {
    fn name(&self) -> &'static str {
        "overview"
    }
    
    fn title(&self) -> Result<String> {
        let hostname = hostname::get()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let domain = get_domain();
        Ok(format!("{}.{}", hostname, domain))
    }
    
    fn render(&self, sys: &System) -> Result<String> {
        let ip_address = get_ip_address()?;
        let cpu_temp = get_cpu_temp().unwrap_or_else(|_| "N/A".to_string());
        let uptime = get_uptime();
        let memory_info = get_memory_info(sys);
        let disk_usage = get_disk_usage();
        
        Ok(format!(
            "{}\ncpu: {}\nuptime: {}\nmemory: {}\ndisk: {}",
            ip_address, cpu_temp, uptime, memory_info, disk_usage
        ))
    }
}

// Screen manager to handle cycling through screens
struct ScreenManager {
    screens: Vec<Box<dyn Screen>>,
    current_index: usize,
    screen_duration: Duration,
    last_switch: std::time::Instant,
}

impl ScreenManager {
    fn new(enabled_screens: Vec<&str>, screen_duration_secs: u64) -> Self {
        let mut screens: Vec<Box<dyn Screen>> = Vec::new();
        
        for screen_name in enabled_screens {
            match screen_name {
                "network" => screens.push(Box::new(NetworkScreen)),
                "system" => screens.push(Box::new(SystemScreen)),
                "storage" => screens.push(Box::new(StorageScreen)),
                "overview" => screens.push(Box::new(OverviewScreen)),
                _ => eprintln!("Unknown screen: {}", screen_name),
            }
        }
        
        // Default to overview if no screens specified
        if screens.is_empty() {
            screens.push(Box::new(OverviewScreen));
        }
        
        Self {
            screens,
            current_index: 0,
            screen_duration: Duration::from_secs(screen_duration_secs),
            last_switch: std::time::Instant::now(),
        }
    }
    
    fn should_switch(&self) -> bool {
        self.screens.len() > 1 && self.last_switch.elapsed() >= self.screen_duration
    }
    
    fn next_screen(&mut self) {
        if self.should_switch() {
            self.current_index = (self.current_index + 1) % self.screens.len();
            self.last_switch = std::time::Instant::now();
        }
    }
    
    fn current_screen(&self) -> &Box<dyn Screen> {
        &self.screens[self.current_index]
    }
}

fn get_ip_address() -> Result<String> {
    // Get all network interfaces
    let interfaces = get_if_addrs()?;
    
    // Look for the first non-loopback interface with an IPv4 address
    for interface in interfaces {
        if !interface.is_loopback() {
            if let std::net::IpAddr::V4(ipv4) = interface.addr.ip() {
                return Ok(ipv4.to_string());
            }
        }
    }
    
    // If no suitable interface is found, return localhost
    Ok("127.0.0.1".to_string())
}

fn get_domain() -> String {
    // Try to get domain from /etc/resolv.conf
    if let Ok(contents) = fs::read_to_string("/etc/resolv.conf") {
        for line in contents.lines() {
            if line.starts_with("search ") {
                return line[7..].trim().to_string();
            }
        }
    }
    
    // Fallback: try to get from hostname -d command
    if let Ok(output) = std::process::Command::new("hostname")
        .arg("-d")
        .output() {
        if output.status.success() {
            let domain = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !domain.is_empty() {
                return domain;
            }
        }
    }
    
    // Default fallback
    "local".to_string()
}

fn get_cpu_temp() -> Result<String> {
    let temp = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?;
    let temp_c = temp.trim().parse::<f32>()? / 1000.0;
    Ok(format!("{:.1}C", temp_c))
}

fn get_memory_info(sys: &System) -> String {
    let total_mem = sys.total_memory() / 1024 / 1024; // Convert to MB
    let used_mem = (sys.total_memory() - sys.free_memory()) / 1024 / 1024;
    format!("{}/{}MB", used_mem, total_mem)
}

fn get_disk_usage() -> String {
    let disks = Disks::new_with_refreshed_list();
    let mut total_size = 0;
    let mut total_used = 0;
    
    for disk in disks.list() {
        total_size += disk.total_space();
        total_used += disk.total_space() - disk.available_space();
    }
    
    let total_gb = total_size / 1024 / 1024 / 1024;
    let used_gb = total_used / 1024 / 1024 / 1024;
    format!("{}/{}GB", used_gb, total_gb)
}

fn get_uptime() -> String {
    let uptime = fs::read_to_string("/proc/uptime")
        .unwrap_or_else(|_| "0".to_string());
    let seconds = uptime.split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    let days = (seconds / 86400.0) as u64;
    let hours = ((seconds % 86400.0) / 3600.0) as u64;
    let minutes = ((seconds % 3600.0) / 60.0) as u64;
    
    format!("{}d {}h {}m", days, hours, minutes)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let mut interval_seconds = 5; // Default to 5 seconds
    let mut clear_only = false;
    let mut daemon_mode = false;
    let mut enabled_screens: Vec<&str> = Vec::new();
    let mut screen_duration_secs = 10; // Default screen duration
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--clear" => clear_only = true,
            "--daemon" | "-d" => daemon_mode = true,
            "--interval" | "-i" => {
                if i + 1 < args.len() {
                    if let Ok(seconds) = args[i + 1].parse::<u64>() {
                        interval_seconds = seconds;
                        i += 1; // Skip next argument
                    }
                }
            }
            "--screen-duration" | "-s" => {
                if i + 1 < args.len() {
                    if let Ok(seconds) = args[i + 1].parse::<u64>() {
                        screen_duration_secs = seconds;
                        i += 1; // Skip next argument
                    }
                }
            }
            "--screens" => {
                if i + 1 < args.len() {
                    enabled_screens = args[i + 1].split(',').collect();
                    i += 1; // Skip next argument
                }
            }
            "--network" => enabled_screens.push("network"),
            "--system" => enabled_screens.push("system"),
            "--storage" => enabled_screens.push("storage"),
            "--overview" => enabled_screens.push("overview"),
            "--help" | "-h" => {
                println!("Info Display - System information on OLED display");
                println!("Usage: {} [OPTIONS]", args[0]);
                println!();
                println!("Options:");
                println!("  --clear              Clear display and exit");
                println!("  --daemon, -d         Run as daemon");
                println!("  --interval, -i <N>   Update interval in seconds (default: 5)");
                println!("  --screen-duration, -s <N>  Duration each screen is shown (default: 10)");
                println!("  --screens <list>     Comma-separated list of screens (network,system,storage,overview)");
                println!("  --network            Enable network screen");
                println!("  --system             Enable system screen");
                println!("  --storage            Enable storage screen");
                println!("  --overview           Enable overview screen (default)");
                println!("  --help, -h           Show this help message");
                println!();
                println!("Examples:");
                println!("  {} --network --system                    # Show network and system screens", args[0]);
                println!("  {} --screens network,system,storage      # Same as above plus storage", args[0]);
                println!("  {} --screen-duration 15 --overview       # Show overview screen for 15s each", args[0]);
                std::process::exit(0);
            }
            arg if arg.starts_with("--interval=") => {
                if let Some(value) = arg.strip_prefix("--interval=") {
                    if let Ok(seconds) = value.parse::<u64>() {
                        interval_seconds = seconds;
                    }
                }
            }
            arg if arg.starts_with("--screen-duration=") => {
                if let Some(value) = arg.strip_prefix("--screen-duration=") {
                    if let Ok(seconds) = value.parse::<u64>() {
                        screen_duration_secs = seconds;
                    }
                }
            }
            arg if arg.starts_with("--screens=") => {
                if let Some(value) = arg.strip_prefix("--screens=") {
                    enabled_screens = value.split(',').collect();
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    // Default to overview screen if no screens specified
    if enabled_screens.is_empty() {
        enabled_screens.push("overview");
    }
    
    // Handle daemon mode
    if daemon_mode {
        let daemonize = Daemonize::new()
            .pid_file("/tmp/info_display.pid")
            .chown_pid_file(true)
            .working_directory("/tmp");

        match daemonize.start() {
            Ok(_) => {}, // Successfully daemonized, continue with normal execution
            Err(e) => {
                eprintln!("Error starting daemon: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    // Handle clear-only mode
    if clear_only {
        let i2c = I2cdev::new("/dev/i2c-1")?;
        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(
            interface,
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode();
        display.init().unwrap();
        display.clear(BinaryColor::Off).unwrap();
        display.flush().unwrap();
        return Ok(());
    }

    // Open I2C bus 1 (adjust to your working bus)
    let i2c = I2cdev::new("/dev/i2c-1")?;

    // Create the I2C interface for SSD1306
    let interface = I2CDisplayInterface::new(i2c);

    // Initialize the display in 128x64 resolution, I2C mode
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().unwrap();

    // Create screen manager with enabled screens
    let mut screen_manager = ScreenManager::new(enabled_screens, screen_duration_secs);

    loop {
        // Clear display
        display.clear(BinaryColor::Off).unwrap();

        // Initialize system info
        let mut sys = System::new_all();
        sys.refresh_all();

        // Check if we need to switch screens
        screen_manager.next_screen();

        // Get current screen content
        let current_screen = screen_manager.current_screen();
        let screen_content = match current_screen.render(&sys) {
            Ok(content) => content,
            Err(e) => format!("Error: {}", e),
        };

        // Split content into lines for rendering
        let lines: Vec<&str> = screen_content.lines().collect();
        
        // Render screen title at top
        let title_style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
        let title = if screen_manager.screens.len() > 1 {
            match current_screen.title() {
                Ok(custom_title) => format!("{}", custom_title),
                Err(_) => format!("{}", current_screen.name()),
            }
        } else {
            match current_screen.title() {
                Ok(custom_title) => custom_title,
                Err(_) => current_screen.name().to_string(),
            }
        };
        Text::new(&title, Point::new(0, 8), title_style).draw(&mut display).unwrap();

        // Render content lines
        let content_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        for (i, line) in lines.iter().enumerate() {
            let y_pos = 22 + (i as i32 * 12); // Start at y=22, 12 pixels per line
            if y_pos < 64 { // Stay within display bounds
                Text::new(line, Point::new(0, y_pos), content_style)
                    .draw(&mut display).unwrap();
            }
        }

        // Flush to the display
        display.flush().unwrap();

        // Sleep for the specified interval
        thread::sleep(Duration::from_secs(interval_seconds));
    }
}

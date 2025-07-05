use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, iso_8859_16::FONT_7X13_BOLD, MonoTextStyle},
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
        let mac_address = get_mac_address();
        
        Ok(format!(
            "{}.{}\n{}\n{}",
            hostname, domain, ip_address, mac_address
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
        let boot_part = get_boot_partition();
        
        // Extract just device name from boot partition
        let boot_device = if let Some(dev_name) = boot_part.split('/').last() {
            dev_name.to_string()
        } else {
            boot_part
        };
        
        Ok(format!(
            "CPU: {}\nUptime: {}\nBoot: {}",
            cpu_temp, uptime, boot_device
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

// Hardware information screen
struct HardwareScreen;

impl Screen for HardwareScreen {
    fn name(&self) -> &'static str {
        "hardware"
    }
    
    fn render(&self, _sys: &System) -> Result<String> {
        let pi_model = get_pi_model();
        let serial = get_serial_number();
        let firmware = get_firmware_version();
        
        // Split model name over two lines for better readability
        let model_lines = if pi_model.len() > 18 {
            // Find a good break point (space, hyphen, or just split)
            let break_point = pi_model[..18].rfind(' ')
                .or_else(|| pi_model[..18].rfind('-'))
                .unwrap_or(15);
            
            let first_line = &pi_model[..break_point];
            let second_line = pi_model[break_point..].trim();
            
            format!("{}\n{}", first_line, second_line)
        } else {
            pi_model
        };
        
        // Truncate serial to last 8 characters for privacy/space
        let short_serial = if serial.len() > 8 {
            format!("...{}", &serial[serial.len()-8..])
        } else {
            serial
        };
        
        Ok(format!(
            "{}\nSerial: {}\nFW: {}",
            model_lines, short_serial, firmware
        ))
    }
}

// Temperature information screen
struct TemperatureScreen;

impl Screen for TemperatureScreen {
    fn name(&self) -> &'static str {
        "temperature"
    }
    
    fn render(&self, _sys: &System) -> Result<String> {
        let cpu_temp = get_cpu_temp().unwrap_or_else(|_| "N/A".to_string());
        let gpu_temp = get_gpu_temp();
        let throttle_status = get_throttle_status();
        let cpu_freq = get_cpu_freq();
        
        Ok(format!(
            "CPU: {}\nGPU: {}\nFreq: {}\nStatus: {}",
            cpu_temp, gpu_temp, cpu_freq, throttle_status
        ))
    }
}

// GPIO and sensor information screen
struct GPIOScreen;

impl Screen for GPIOScreen {
    fn name(&self) -> &'static str {
        "gpio"
    }
    
    fn render(&self, _sys: &System) -> Result<String> {
        let i2c_devices = get_i2c_devices();
        let gpio_states = get_gpio_states();
        let spi_devices = get_spi_devices();
        let wire_sensors = get_1wire_sensors();
        
        // Truncate long device lists for display
        let short_i2c = if i2c_devices.len() > 12 {
            format!("{}...", &i2c_devices[..9])
        } else {
            i2c_devices
        };
        
        let short_gpio = if gpio_states.len() > 15 {
            format!("{}...", &gpio_states[..12])
        } else {
            gpio_states
        };
        
        Ok(format!(
            "I2C: {}\nGPIO: {}\nSPI: {}\n1W: {}",
            short_i2c, short_gpio, spi_devices, wire_sensors
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
                "hardware" => screens.push(Box::new(HardwareScreen)),
                "temperature" => screens.push(Box::new(TemperatureScreen)),
                "gpio" => screens.push(Box::new(GPIOScreen)),
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

fn get_mac_address() -> String {
    // First try to get MAC from the same interface that has the IP
    if let Ok(interfaces) = get_if_addrs() {
        for interface in interfaces {
            if !interface.is_loopback() {
                if let std::net::IpAddr::V4(_) = interface.addr.ip() {
                    // Found the interface with IP, now get its MAC
                    let interface_name = &interface.name;
                    
                    // Try reading MAC from /sys/class/net/{interface}/address
                    let mac_path = format!("/sys/class/net/{}/address", interface_name);
                    if let Ok(mac) = fs::read_to_string(&mac_path) {
                        let mac_addr = mac.trim().to_uppercase();
                        // Only return if it's not a placeholder MAC
                        if !mac_addr.starts_with("00:00:00") && mac_addr != "00:00:00:00:00:00" {
                            return mac_addr;
                        }
                    }
                }
            }
        }
    }
    
    // Fallback: try to find eth0 MAC specifically
    if let Ok(mac) = fs::read_to_string("/sys/class/net/eth0/address") {
        let mac_addr = mac.trim().to_uppercase();
        if !mac_addr.starts_with("00:00:00") && mac_addr != "00:00:00:00:00:00" {
            return mac_addr;
        }
    }
    
    "Unknown".to_string()
}

fn get_cpu_temp() -> Result<String> {
    let temp = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?;
    let temp_c = temp.trim().parse::<f32>()? / 1000.0;
    Ok(format!("{:.1}C", temp_c))
}

fn get_gpu_temp() -> String {
    if let Ok(output) = std::process::Command::new("vcgencmd")
        .arg("measure_temp")
        .output() {
        if output.status.success() {
            let temp_output = String::from_utf8_lossy(&output.stdout);
            // Output format: "temp=45.2'C"
            if let Some(temp_part) = temp_output.split('=').nth(1) {
                if let Some(temp_value) = temp_part.split('\'').next() {
                    return format!("{}C", temp_value);
                }
            }
        }
    }
    "N/A".to_string()
}

fn get_throttle_status() -> String {
    if let Ok(output) = std::process::Command::new("vcgencmd")
        .arg("get_throttled")
        .output() {
        if output.status.success() {
            let throttle_output = String::from_utf8_lossy(&output.stdout);
            // Output format: "throttled=0x0"
            if let Some(value_part) = throttle_output.split('=').nth(1) {
                let throttle_value = value_part.trim();
                return match throttle_value {
                    "0x0" => "OK".to_string(),
                    "0x1" => "Under-voltage".to_string(),
                    "0x2" => "ARM freq cap".to_string(),
                    "0x4" => "Throttled".to_string(),
                    "0x8" => "Soft temp limit".to_string(),
                    "0x10000" => "Under-volt occurred".to_string(),
                    "0x20000" => "ARM freq cap occurred".to_string(),
                    "0x40000" => "Throttling occurred".to_string(),
                    "0x80000" => "Soft temp limit occurred".to_string(),
                    _ => format!("Status: {}", throttle_value),
                };
            }
        }
    }
    "Unknown".to_string()
}

fn get_cpu_freq() -> String {
    if let Ok(output) = std::process::Command::new("vcgencmd")
        .arg("measure_clock")
        .arg("arm")
        .output() {
        if output.status.success() {
            let freq_output = String::from_utf8_lossy(&output.stdout);
            // Output format: "frequency(48)=1500000000"
            if let Some(freq_part) = freq_output.split('=').nth(1) {
                if let Ok(freq_hz) = freq_part.trim().parse::<u64>() {
                    let freq_mhz = freq_hz / 1_000_000;
                    return format!("{}MHz", freq_mhz);
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_i2c_devices() -> String {
    let mut devices = Vec::new();
    
    // Check for I2C devices on bus 1 (most common on Pi)
    if let Ok(output) = std::process::Command::new("i2cdetect")
        .args(["-y", "1"])
        .output() {
        if output.status.success() {
            let detect_output = String::from_utf8_lossy(&output.stdout);
            for line in detect_output.lines().skip(1) { // Skip header
                for (_col, addr) in line.split_whitespace().skip(1).enumerate() {
                    if addr != "--" && addr.len() == 2 {
                        let device_addr = format!("0x{}", addr);
                        devices.push(device_addr);
                    }
                }
            }
        }
    }
    
    if devices.is_empty() {
        "None".to_string()
    } else {
        devices.join(",")
    }
}

fn get_gpio_states() -> String {
    // Check some commonly used GPIO pins (avoid system pins)
    let check_pins = [18, 19, 20, 21, 22, 23, 24, 25];
    let mut states = Vec::new();
    
    for pin in check_pins {
        let gpio_path = format!("/sys/class/gpio/gpio{}/value", pin);
        if let Ok(value) = fs::read_to_string(&gpio_path) {
            let state = if value.trim() == "1" { "H" } else { "L" };
            states.push(format!("{}:{}", pin, state));
        } else {
            // Pin might not be exported, try to check direction
            let dir_path = format!("/sys/class/gpio/gpio{}/direction", pin);
            if fs::metadata(&dir_path).is_ok() {
                states.push(format!("{}:?", pin));
            }
        }
    }
    
    if states.is_empty() {
        "No active pins".to_string()
    } else {
        states.join(" ")
    }
}

fn get_spi_devices() -> String {
    let mut spi_devices = Vec::new();
    
    // Check for SPI devices
    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let filename = entry.file_name();
            let name = filename.to_string_lossy();
            if name.starts_with("spidev") {
                spi_devices.push(name.to_string());
            }
        }
    }
    
    if spi_devices.is_empty() {
        "None".to_string()
    } else {
        spi_devices.join(",")
    }
}

fn get_1wire_sensors() -> String {
    let w1_path = "/sys/bus/w1/devices";
    let mut sensors = Vec::new();
    
    if let Ok(entries) = fs::read_dir(w1_path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let sensor_name = name.to_string_lossy();
            // Skip the master device
            if !sensor_name.starts_with("w1_bus_master") {
                // Try to read temperature if it's a temperature sensor
                let temp_path = format!("{}/{}/w1_slave", w1_path, sensor_name);
                if let Ok(contents) = fs::read_to_string(&temp_path) {
                    if contents.contains("YES") {
                        if let Some(temp_line) = contents.lines().last() {
                            if let Some(temp_pos) = temp_line.find("t=") {
                                if let Ok(temp_raw) = temp_line[temp_pos + 2..].parse::<i32>() {
                                    let temp_c = temp_raw as f32 / 1000.0;
                                    sensors.push(format!("{:.1}C", temp_c));
                                    continue;
                                }
                            }
                        }
                    }
                }
                // If we can't read temperature, just show the device type
                if sensor_name.len() > 3 {
                    sensors.push(sensor_name[..3].to_string());
                }
            }
        }
    }
    
    if sensors.is_empty() {
        "None".to_string()
    } else {
        sensors.join(",")
    }
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

fn get_pi_model() -> String {
    if let Ok(contents) = fs::read_to_string("/proc/device-tree/model") {
        // Remove null terminator and clean up
        contents.trim_end_matches('\0').to_string()
    } else if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
        // Fallback: look for Model in cpuinfo
        for line in contents.lines() {
            if line.starts_with("Model") {
                if let Some(model) = line.split(':').nth(1) {
                    return model.trim().to_string();
                }
            }
        }
        "Unknown Pi Model".to_string()
    } else {
        "Unknown Pi Model".to_string()
    }
}

fn get_serial_number() -> String {
    if let Ok(contents) = fs::read_to_string("/proc/device-tree/serial-number") {
        contents.trim_end_matches('\0').to_string()
    } else if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
        // Fallback: look for Serial in cpuinfo
        for line in contents.lines() {
            if line.starts_with("Serial") {
                if let Some(serial) = line.split(':').nth(1) {
                    return serial.trim().to_string();
                }
            }
        }
        "Unknown".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn get_firmware_version() -> String {
    if let Ok(output) = std::process::Command::new("vcgencmd")
        .arg("version")
        .output() {
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            // Extract just the version part
            for line in version_output.lines() {
                if line.contains("version ") {
                    if let Some(version) = line.split("version ").nth(1) {
                        return version.split(' ').next().unwrap_or("Unknown").to_string();
                    }
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_boot_partition() -> String {
    if let Ok(output) = std::process::Command::new("findmnt")
        .args(["-n", "-o", "SOURCE", "/boot"])
        .output() {
        if output.status.success() {
            let boot_device = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !boot_device.is_empty() {
                return boot_device;
            }
        }
    }
    
    // Fallback: check /proc/mounts
    if let Ok(contents) = fs::read_to_string("/proc/mounts") {
        for line in contents.lines() {
            if line.contains(" /boot ") {
                if let Some(device) = line.split_whitespace().next() {
                    return device.to_string();
                }
            }
        }
    }
    
    "Unknown".to_string()
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
            "--hardware" => enabled_screens.push("hardware"),
            "--temperature" => enabled_screens.push("temperature"),
            "--gpio" => enabled_screens.push("gpio"),
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
                println!("  --screens <list>     Comma-separated list of screens (network,system,storage,hardware,temperature,gpio,overview)");
                println!("  --network            Enable network screen");
                println!("  --system             Enable system screen");
                println!("  --storage            Enable storage screen");
                println!("  --hardware           Enable hardware screen");
                println!("  --temperature        Enable temperature screen");
                println!("  --gpio               Enable GPIO/sensor screen");
                println!("  --overview           Enable overview screen (default)");
                println!("  --help, -h           Show this help message");
                println!();
                println!("Examples:");
                println!("  {} --network --system                    # Show network and system screens", args[0]);
                println!("  {} --screens network,system,gpio         # Same as above plus GPIO info", args[0]);
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

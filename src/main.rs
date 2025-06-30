use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, iso_8859_9::FONT_7X14_BOLD, MonoTextStyle},
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
    
    for i in 1..args.len() {
        match args[i].as_str() {
            "--clear" => clear_only = true,
            "--daemon" | "-d" => daemon_mode = true,
            "--interval" | "-i" => {
                if i + 1 < args.len() {
                    if let Ok(seconds) = args[i + 1].parse::<u64>() {
                        interval_seconds = seconds;
                    }
                }
            }
            arg if arg.starts_with("--interval=") => {
                if let Some(value) = arg.strip_prefix("--interval=") {
                    if let Ok(seconds) = value.parse::<u64>() {
                        interval_seconds = seconds;
                    }
                }
            }
            _ => {}
        }
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

    loop {
        // Clear display
        display.clear(BinaryColor::Off).unwrap();

        // Initialize system info
        let mut sys = System::new_all();
        sys.refresh_all();

        // Get system information
        let hostname = hostname::get()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let domain = get_domain();
        let ip_address = get_ip_address().unwrap();
        let cpu_temp = get_cpu_temp().unwrap_or_else(|_| "N/A".to_string());
        let memory_info = get_memory_info(&sys);
        let disk_usage = get_disk_usage();
        let uptime = get_uptime();

        let yellow_text = format!(
            "{}.{}",
            hostname, domain
        );

        let blue_text = format!(
            "{}\ncpu: {}\nuptime: {}\nmemory: {}\ndisk: {}",
            ip_address, cpu_temp, uptime, memory_info, disk_usage
        );

        let style = MonoTextStyle::new(&FONT_7X14_BOLD, BinaryColor::On);
        Text::new(&yellow_text, Point::new(0, 8), style).draw(&mut display).unwrap();

        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        Text::new(&blue_text, Point::new(0, 22), style).draw(&mut display).unwrap();

        // Flush to the display
        display.flush().unwrap();

        // Sleep for the specified interval
        thread::sleep(Duration::from_secs(interval_seconds));
    }
}

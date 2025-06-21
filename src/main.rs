use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
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
    format!("Mem: {}/{}MB", used_mem, total_mem)
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
    format!("Disk: {}/{}GB", used_gb, total_gb)
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
    if args.len() > 1 && args[1] == "--clear" {
        let i2c = I2cdev::new("/dev/i2c-1")?;
        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(
            interface,
            DisplaySize128x32,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode();
        display.init().unwrap();
        display.clear(BinaryColor::Off).unwrap();
        display.flush().unwrap();
        return Ok(());
    }

    // Open I2C bus 13 (adjust to your working bus)
    let i2c = I2cdev::new("/dev/i2c-1")?;

    // Create the I2C interface for SSD1306
    let interface = I2CDisplayInterface::new(i2c);

    // Initialize the display in 128x64 resolution, I2C mode
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x32,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().unwrap();

    // Clear display
    display.clear(BinaryColor::Off).unwrap();

    // Set up font and style
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

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

    let text = format!(
        "{}.{}\n{} {}\nuptime: {}\nmemory: {}\ndisk: {}",
        hostname, domain, ip_address, cpu_temp, uptime, memory_info, disk_usage
    );
    
    Text::new(&text, Point::new(0, 8), style).draw(&mut display).unwrap();

    // Flush to the display
    display.flush().unwrap();

    Ok(())
}

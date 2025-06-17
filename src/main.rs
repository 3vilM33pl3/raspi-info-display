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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open I2C bus 13 (adjust to your working bus)
    let i2c = I2cdev::new("/dev/i2c-1")?; // or "/dev/i2c-14"

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

    // Get IP address
    let ip_address = get_ip_address().unwrap();

    // Get hostname
    let hostname = hostname::get()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    // Draw text
    let text = format!("Hostname: {}\nIP: {}\n", hostname, ip_address);
    Text::new(&text, Point::new(0, 16), style).draw(&mut display).unwrap();

    // Flush to the display
    display.flush().unwrap();

    Ok(())
}

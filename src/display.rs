use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, iso_8859_16::FONT_7X13_BOLD, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use linux_embedded_hal::I2cdev;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use std::sync::{Arc, Mutex};
use crate::tca9548a::Tca9548a;

pub struct DisplayManager {
    display: Ssd1306<I2CInterface<I2cdev>, DisplaySize128x64, ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>>,
    _mux_handle: Option<Arc<Mutex<Tca9548a>>>,
}

impl DisplayManager {
    pub fn new(use_multiplexer: bool, mux_channel: u8, mux_address: u8) -> Result<Self, Box<dyn std::error::Error>> {
        let (display, mux_handle) = if use_multiplexer {
            println!("Using TCA9548A multiplexer on address 0x{:02X}, channel {}", mux_address, mux_channel);
            
            // Create shared I2C bus and multiplexer
            let i2c_shared = Arc::new(Mutex::new(I2cdev::new("/dev/i2c-1")?));
            let mut mux = Tca9548a::with_address(Arc::clone(&i2c_shared), mux_address);
            mux.select_channel(mux_channel)?;
            
            // Store mux in Arc<Mutex> to keep it alive
            let mux_handle = Arc::new(Mutex::new(mux));
            
            // Create a new I2C connection for the display
            // (the channel is already selected on the multiplexer)
            let i2c = I2cdev::new("/dev/i2c-1")?;
            let interface = I2CDisplayInterface::new(i2c);
            
            let mut display = Ssd1306::new(
                interface,
                DisplaySize128x64,
                DisplayRotation::Rotate0,
            )
            .into_buffered_graphics_mode();
            
            display.init().map_err(|e| format!("Failed to initialize display on multiplexer channel {}: {:?}", mux_channel, e))?;
            (display, Some(mux_handle))
        } else {
            // Standard I2C connection
            let i2c = I2cdev::new("/dev/i2c-1")?;
            let interface = I2CDisplayInterface::new(i2c);
            
            let mut display = Ssd1306::new(
                interface,
                DisplaySize128x64,
                DisplayRotation::Rotate0,
            )
            .into_buffered_graphics_mode();
            
            display.init().map_err(|e| format!("Failed to initialize display on I2C bus: {:?}. Check if display is connected or use --mux flag if using multiplexer.", e))?;
            (display, None)
        };

        Ok(DisplayManager {
            display,
            _mux_handle: mux_handle,
        })
    }

    pub fn clear_display(use_multiplexer: bool, mux_channel: u8, mux_address: u8) -> Result<(), Box<dyn std::error::Error>> {
        if use_multiplexer {
            // Setup multiplexer and select channel
            let i2c = Arc::new(Mutex::new(I2cdev::new("/dev/i2c-1")?));
            let mut mux = Tca9548a::with_address(Arc::clone(&i2c), mux_address);
            mux.select_channel(mux_channel)?;
            drop(mux);
            
            // Now use regular I2C (the channel is already selected)
            let i2c = I2cdev::new("/dev/i2c-1")?;
            let interface = I2CDisplayInterface::new(i2c);
            let mut display = Ssd1306::new(
                interface,
                DisplaySize128x64,
                DisplayRotation::Rotate0,
            )
            .into_buffered_graphics_mode();
            display.init().map_err(|e| format!("Failed to initialize display on multiplexer channel {} for clearing: {:?}", mux_channel, e))?;
            display.clear(BinaryColor::Off).unwrap();
            display.flush().unwrap();
        } else {
            let i2c = I2cdev::new("/dev/i2c-1")?;
            let interface = I2CDisplayInterface::new(i2c);
            let mut display = Ssd1306::new(
                interface,
                DisplaySize128x64,
                DisplayRotation::Rotate0,
            )
            .into_buffered_graphics_mode();
            display.init().map_err(|e| format!("Failed to initialize display for clearing: {:?}. Check if display is connected or use --mux flag if using multiplexer.", e))?;
            display.clear(BinaryColor::Off).unwrap();
            display.flush().unwrap();
        }
        Ok(())
    }

    pub fn render_content(&mut self, title: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Clear display
        self.display.clear(BinaryColor::Off).unwrap();
        
        // Draw title (bold, at the top)
        let title_style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);
        Text::new(title, Point::new(0, 12), title_style).draw(&mut self.display).unwrap();
        
        // Draw content lines
        let content_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        for (i, line) in content.lines().enumerate() {
            let y_pos = 25 + (i as i32 * 12);
            if y_pos < 64 { // Make sure we don't exceed display height
                Text::new(line, Point::new(0, y_pos), content_style).draw(&mut self.display).unwrap();
            }
        }
        
        // Flush to display
        self.display.flush().unwrap();
        Ok(())
    }
}
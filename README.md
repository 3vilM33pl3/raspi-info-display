# Info Display

A Rust application that displays system information on an SSD1306 OLED display connected to a Raspberry Pi via I2C.

## Features

- Displays hostname and IP address on an SSD1306 OLED display
- Uses I2C communication for display control
- Supports 128x32 pixel resolution
- Automatically detects the first available non-loopback network interface

## Hardware Requirements

- Raspberry Pi (any model with I2C support)
- SSD1306 OLED display (128x32 or 128x64)
- I2C connection between Raspberry Pi and display

## Software Requirements

- Rust (latest stable version)
- I2C enabled on Raspberry Pi
- Required system packages for I2C support

## Installation

1. Enable I2C on your Raspberry Pi:
   ```bash
   sudo raspi-config
   # Navigate to Interface Options -> I2C -> Enable
   ```

2. Install required system packages:
   ```bash
   sudo apt-get update
   sudo apt-get install -y i2c-tools
   ```

3. Clone the repository:
   ```bash
   git clone [your-repository-url]
   cd info_display
   ```

4. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

1. Connect your SSD1306 display to the Raspberry Pi:
   - VCC to 3.3V
   - GND to GND
   - SCL to SCL (GPIO 3)
   - SDA to SDA (GPIO 2)

2. Run the application:
   ```bash
   sudo ./target/release/info_display
   ```

Note: The application requires root privileges to access the I2C bus.

## Configuration

The default I2C bus is set to `/dev/i2c-1`. If your display is connected to a different bus, modify the bus path in `src/main.rs`.

## Dependencies

- rppal: Raspberry Pi peripheral access library
- embedded-hal: Hardware abstraction layer for embedded systems
- linux-embedded-hal: Linux implementation of embedded-hal
- ssd1306: Driver for SSD1306 OLED displays
- embedded-graphics: Graphics library for embedded systems
- anyhow: Error handling
- get_if_addrs: Network interface information
- hostname: Hostname retrieval


## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

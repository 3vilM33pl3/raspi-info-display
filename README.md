# Info Display

A modular Rust application that displays comprehensive system information on an SSD1306 OLED display connected to a Raspberry Pi via I2C.

## Features

- **Modular Screen System**: Choose from multiple information screens that cycle automatically
- **Network Information**: Hostname, domain, IP address, and MAC address
- **System Monitoring**: CPU temperature, uptime, and boot partition information
- **Storage Metrics**: Memory usage and disk usage across all mounted filesystems
- **Hardware Details**: Pi model, serial number, and firmware version
- **Temperature Monitoring**: CPU/GPU temperatures, frequency, and throttling status
- **GPIO/Sensor Support**: I2C devices, GPIO pin states, SPI devices, and 1-Wire sensors
- **Overview Screen**: Combined view with all essential information
- **Daemon Mode**: Run as a background service with systemd integration
- **Configurable Display**: Customizable update intervals and screen rotation timing
- **128x64 OLED Support**: Optimized for SSD1306 displays via I2C

## Hardware Requirements

- Raspberry Pi (any model with I2C support)
- SSD1306 OLED display (128x64 pixels, 128x32 compatible)
- I2C connection between Raspberry Pi and display
- Optional: Additional sensors (1-Wire temperature sensors, I2C devices, etc.)

## Software Requirements

- Rust (latest stable version)
- I2C enabled on Raspberry Pi
- System packages: `i2c-tools` (for GPIO screen functionality)
- Optional: `vcgencmd` (for temperature and hardware monitoring)

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

### Hardware Setup

#### Raspberry Pi 5 Wiring Diagram

Connect your SSD1306 OLED display to the Raspberry Pi 5 as follows:

```
SSD1306 OLED Display        Raspberry Pi 5 GPIO Header
┌─────────────────┐         ┌─────────────────────────────┐
│                 │    ┌────┤  1 [3.3V]        [5V] 2     │◄─── VCC
│  ┌─────────┐    │    │┌───┤  3 [SDA]         [5V] 4     │◄─── SDA
│  │ DISPLAY │    │    ││┌──┤  5 [SCL]        [GND] 6     │◄─── SCL
│  │         │    │    │││  │  7 [GPIO4]   [GPIO14] 8     │
│  │ 128x64  │    │    │││  │  9 [GND]     [GPIO15] 10    │
│  │         │    │    │││  │ 11 [GPIO17]  [GPIO18] 12    │
│  │ PIXELS  │    │    │││  │ 13 [GPIO27]     [GND] 14    │◄─── GND
│  └─────────┘    │    │││  │ 15 [GPIO22]  [GPIO23] 16    │
│                 │    │││  │ 17 [3.3V]    [GPIO24] 18    │
│ VCC SDA SCL GND │    │││  │ 19 [GPIO10]     [GND] 20    │
│  │   │   │   │  │    │││  │ 21 [GPIO9]   [GPIO25] 22    │
└──┼───┼───┼───┼──┘    │││  │ 23 [GPIO11]   [GPIO8] 24    │
   │   │   │   └───────┼┼┼──┤ 25 [GND]      [GPIO7] 26    │
   │   │   └───────────┼┼┘  │ 27 [ID_SD]    [ID_SC] 28    │
   │   └───────────────┼┘   │ 29 [GPIO5]      [GND] 30    │
   └───────────────────┘    │ 31 [GPIO6]   [GPIO12] 32    │
                            │ 33 [GPIO13]     [GND] 34    │
                            │ 35 [GPIO19]  [GPIO16] 36    │
                            │ 37 [GPIO26]  [GPIO20] 38    │
                            │ 39 [GND]     [GPIO21] 40    │
                            └─────────────────────────────┘
```

#### Connection Summary

| SSD1306 Pin | Function | Raspberry Pi 5 Pin | GPIO Number |
|-------------|----------|-------------------|-------------|
| VCC         | Power    | Pin 1 (3.3V)      | -           |
| GND         | Ground   | Pin 14 (GND)      | -           |
| SCL         | I2C Clock| Pin 5             | GPIO 3      |
| SDA         | I2C Data | Pin 3             | GPIO 2      |

#### Important Notes

- **Use 3.3V**: Always connect VCC to 3.3V (Pin 1), never to 5V
- **I2C Address**: Most SSD1306 displays use address 0x3C or 0x3D
- **Pull-up Resistors**: Built into the Pi 5, no external resistors needed
- **Wire Length**: Keep I2C wires short (< 30cm) for reliable communication

### Basic Usage

Run with default overview screen:
```bash
sudo ./target/release/info_display
```

### Screen Selection

Choose specific screens to display:
```bash
# Single screen
sudo ./target/release/info_display --network

# Multiple screens (cycles through them)
sudo ./target/release/info_display --network --system --temperature

# Using comma-separated format
sudo ./target/release/info_display --screens network,system,storage,hardware,temperature,gpio

# All available screens
sudo ./target/release/info_display --screens network,system,storage,hardware,temperature,gpio,overview
```

### Available Screens

- **`--network`**: Network information (hostname, domain, IP, MAC address)
- **`--system`**: System information (CPU temp, uptime, boot partition)
- **`--storage`**: Storage information (memory and disk usage)
- **`--hardware`**: Hardware information (Pi model, serial, firmware)
- **`--temperature`**: Temperature monitoring (CPU/GPU temps, frequency, throttling)
- **`--gpio`**: GPIO and sensor information (I2C devices, GPIO states, SPI, 1-Wire)
- **`--overview`**: Combined overview (default, shows key information from all screens)

### Configuration Options

```bash
# Set update interval (how often data refreshes)
sudo ./target/release/info_display --interval 10 --network

# Set screen rotation duration (for multiple screens)
sudo ./target/release/info_display --screen-duration 15 --network --system

# Run as daemon
sudo ./target/release/info_display --daemon --network --system

# Clear display and exit
sudo ./target/release/info_display --clear
```

### Daemon Mode and Service

Install as a systemd service:
```bash
# Build Debian package
./build_package.sh

# Install package
sudo dpkg -i target/debian/info-display_*.deb

# Start service
sudo systemctl start info-display.service

# Enable on boot
sudo systemctl enable info-display.service

# Check status
sudo systemctl status info-display.service
```

**Note**: The application requires root privileges to access the I2C bus and system monitoring features.

## How It Works

### Modular Screen Architecture

The application uses a trait-based modular screen system:

- **Screen Trait**: Each screen implements a `Screen` trait with `name()`, `title()`, and `render()` methods
- **Screen Manager**: Handles cycling through enabled screens based on timing configuration
- **Dynamic Content**: Each screen gathers real-time system information when displayed
- **Flexible Display**: Screens can show custom titles (e.g., hostname for overview screen)

### Information Gathering

The application collects data from various system sources:

- **File System**: Reads from `/proc/`, `/sys/`, and `/dev/` for system information
- **Network Interfaces**: Uses `get_if_addrs` crate to discover network configuration
- **System Commands**: Executes `vcgencmd`, `i2cdetect`, `findmnt` for hardware details
- **System Info Crate**: Leverages `sysinfo` for memory and process information

### Display Management

- **I2C Communication**: Uses `linux-embedded-hal` and `ssd1306` crates for display control
- **Graphics Rendering**: Employs `embedded-graphics` for text and layout
- **Screen Cycling**: Automatically rotates through enabled screens at configurable intervals
- **Real-time Updates**: Refreshes data at specified intervals (default: 5 seconds)

### Data Sources by Screen

- **Network**: `/proc/net/`, network interfaces, `/sys/class/net/*/address`
- **System**: `/sys/class/thermal/`, `/proc/uptime`, `findmnt` output
- **Storage**: `sysinfo` crate, mounted filesystem data
- **Hardware**: `/proc/device-tree/`, `/proc/cpuinfo`, `vcgencmd` commands
- **Temperature**: `/sys/class/thermal/`, `vcgencmd measure_temp`, throttling status
- **GPIO/Sensors**: `/sys/class/gpio/`, `i2cdetect`, `/sys/bus/w1/devices/`, `/dev/spidev*`

## Configuration

The default I2C bus is set to `/dev/i2c-1`. If your display is connected to a different bus, modify the bus path in `src/main.rs`.

## Dependencies

### Core Libraries
- **linux-embedded-hal**: Linux implementation of embedded-hal for I2C communication
- **ssd1306**: Driver for SSD1306 OLED displays
- **embedded-graphics**: Graphics library for rendering text and shapes
- **anyhow**: Simplified error handling and propagation

### System Information
- **sysinfo**: Cross-platform system information (memory, CPU, disks)
- **get_if_addrs**: Network interface discovery and information
- **hostname**: System hostname retrieval
- **chrono**: Date and time handling
- **daemonize**: Process daemonization support

### Development
- **cargo-deb**: Debian package generation (dev dependency)

## Examples

### Basic Monitoring Setup
```bash
# Monitor network and system info with 20-second screen rotation
sudo ./target/release/info_display --network --system --screen-duration 20
```

### Complete System Dashboard
```bash
# Show all screens with 15-second intervals
sudo ./target/release/info_display --screens network,system,storage,hardware,temperature,gpio --screen-duration 15
```

### Development/Debugging Setup
```bash
# GPIO and temperature monitoring for development
sudo ./target/release/info_display --gpio --temperature --interval 3
```

## Troubleshooting

### Display Issues
- **Enable I2C**: `sudo raspi-config` → Interface Options → I2C → Enable
- **Check I2C connection**: `sudo i2cdetect -y 1` (should show device at 0x3c or 0x3d)
- **Verify wiring**: Follow the diagram above exactly
  - VCC → Pin 1 (3.3V) - **NEVER use 5V**
  - GND → Pin 14 (GND)
  - SCL → Pin 5 (GPIO 3)
  - SDA → Pin 3 (GPIO 2)
- **Pi 5 specific**: Ensure you're using the correct GPIO pins (layout is the same as Pi 4)
- **Test I2C**: `sudo i2cdetect -y 1` should show your display (usually 0x3c)

### Permission Issues
- Run with `sudo` (required for I2C and system access)
- For systemd service: the service runs as root automatically

### Screen-Specific Issues
- **GPIO screen shows "None"**: Install `i2c-tools`, check GPIO export
- **Temperature readings "N/A"**: Ensure `vcgencmd` is available
- **1-Wire sensors not detected**: Enable 1-Wire: `dtoverlay=w1-gpio` in `/boot/config.txt`

## Performance Notes

- **Update Interval**: Lower intervals (1-2 seconds) may impact system performance
- **Screen Count**: More screens use slightly more CPU during transitions
- **I2C Bus**: Shares bus with other I2C devices; avoid conflicts
- **Memory Usage**: Minimal (~2-5MB RAM usage)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Adding New Screens

1. Implement the `Screen` trait with `name()`, `title()`, and `render()` methods
2. Add the screen to the `ScreenManager` match statement
3. Add command-line option parsing
4. Update help text and documentation
5. Test with various configurations

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust application that displays system information on an SSD1306 OLED display connected to a Raspberry Pi via I2C. The service displays hostname, IP address, CPU temperature, memory usage, disk usage, and uptime.

## Build Commands

- **Build debug**: `cargo build`
- **Build release**: `cargo build --release`
- **Check code**: `cargo check`
- **Clean build artifacts**: `cargo clean`
- **Build Debian package**: `./build_package.sh` or `cargo deb`

## Testing

This project currently has no test suite. When adding tests, use:
- `cargo test` - Run all tests
- `cargo test --release` - Run tests in release mode

## Application Architecture

### Core Components

- **Single binary application** (`src/main.rs`) - modular design with screen system
- **Screen Trait System**: Modular screens implementing the `Screen` trait for different info displays
- **Screen Manager**: Handles cycling through enabled screens and timing
- **System Information Gathering**: Functions to collect hostname, IP, CPU temp, memory, disk usage, uptime
- **Display Management**: SSD1306 OLED display control via I2C using embedded-graphics
- **Service Management**: Daemon mode support with systemd integration

### Key Functions

- `get_ip_address()` - Finds first non-loopback IPv4 interface
- `get_domain()` - Reads domain from `/etc/resolv.conf` or hostname command
- `get_mac_address()` - Gets MAC address of primary ethernet interface  
- `get_cpu_temp()` - Reads temperature from `/sys/class/thermal/thermal_zone0/temp`
- `get_memory_info()` - Uses sysinfo crate for memory statistics
- `get_disk_usage()` - Aggregates disk usage across all mounted filesystems
- `get_uptime()` - Parses `/proc/uptime` for system uptime
- `get_pi_model()` - Reads Pi model from `/proc/device-tree/model` or `/proc/cpuinfo`
- `get_serial_number()` - Reads serial from `/proc/device-tree/serial-number` or `/proc/cpuinfo`
- `get_firmware_version()` - Gets firmware version via `vcgencmd version`
- `get_boot_partition()` - Finds boot partition via `findmnt` or `/proc/mounts`
- `get_gpu_temp()` - Gets GPU temperature via `vcgencmd measure_temp`
- `get_throttle_status()` - Gets throttling status via `vcgencmd get_throttled`
- `get_cpu_freq()` - Gets current CPU frequency via `vcgencmd measure_clock arm`
- `get_i2c_devices()` - Scans I2C bus 1 for connected devices via `i2cdetect`
- `get_gpio_states()` - Reads GPIO pin states from `/sys/class/gpio/`
- `get_spi_devices()` - Lists available SPI devices from `/dev/spidev*`
- `get_1wire_sensors()` - Reads 1-Wire sensor data from `/sys/bus/w1/devices/`

### Hardware Requirements

- Raspberry Pi with I2C enabled
- SSD1306 OLED display (128x64 pixels)
- I2C connection on `/dev/i2c-1`

## Command Line Interface

- `--clear` - Clear display and exit
- `--daemon` or `-d` - Run as daemon
- `--interval N` or `-i N` - Update interval in seconds (default: 5)
- `--screen-duration N` or `-s N` - Duration each screen is shown in seconds (default: 10)
- `--screens <list>` - Comma-separated list of screens to enable
- `--network` - Enable network information screen
- `--system` - Enable system information screen (CPU, uptime, boot partition)
- `--storage` - Enable storage information screen (memory, disk)
- `--hardware` - Enable hardware information screen (Pi model, serial, firmware)
- `--temperature` - Enable temperature information screen (CPU/GPU temps, throttling)
- `--gpio` - Enable GPIO/sensor information screen (I2C, SPI, GPIO states, 1-Wire)
- `--overview` - Enable overview screen (all info combined, default)
- `--help` or `-h` - Show help message

## Screen Types

The application supports multiple modular screens that can be enabled individually:

- **Network Screen**: Displays hostname, domain, IP address, and MAC address
- **System Screen**: Shows CPU temperature, system uptime, and boot partition
- **Storage Screen**: Shows memory usage and disk usage
- **Hardware Screen**: Shows Pi model, serial number, and firmware version
- **Temperature Screen**: Shows CPU/GPU temperatures, CPU frequency, and throttling status
- **GPIO/Sensor Screen**: Shows I2C devices, GPIO pin states, SPI devices, and 1-Wire sensors
- **Overview Screen**: Combined view with all information (original layout)

When multiple screens are enabled, the application cycles through them at the specified screen duration interval.

## Deployment

The application is packaged as a Debian package with systemd service:

- **Service file**: `debian/systemd/info-display.service`
- **Install**: `sudo dpkg -i target/debian/info-display_*.deb`
- **Start**: `sudo systemctl start info-display.service`
- **Status**: `sudo systemctl status info-display.service`

The service runs as root (required for I2C access) with security restrictions in the systemd unit file.

## Development Notes

- Requires root privileges for I2C bus access
- Display initialization and rendering happens in main loop
- Error handling uses anyhow crate for simplified error propagation
- No configuration files - all settings via command line arguments
- Uses blocking I/O and thread::sleep for timing
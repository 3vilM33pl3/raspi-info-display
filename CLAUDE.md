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

- **Single binary application** (`src/main.rs`) - monolithic design with all functionality in one file
- **System Information Gathering**: Functions to collect hostname, IP, CPU temp, memory, disk usage, uptime
- **Display Management**: SSD1306 OLED display control via I2C using embedded-graphics
- **Service Management**: Daemon mode support with systemd integration

### Key Functions

- `get_ip_address()` - Finds first non-loopback IPv4 interface
- `get_domain()` - Reads domain from `/etc/resolv.conf` or hostname command  
- `get_cpu_temp()` - Reads temperature from `/sys/class/thermal/thermal_zone0/temp`
- `get_memory_info()` - Uses sysinfo crate for memory statistics
- `get_disk_usage()` - Aggregates disk usage across all mounted filesystems
- `get_uptime()` - Parses `/proc/uptime` for system uptime

### Hardware Requirements

- Raspberry Pi with I2C enabled
- SSD1306 OLED display (128x64 pixels)
- I2C connection on `/dev/i2c-1`

## Command Line Interface

- `--clear` - Clear display and exit
- `--daemon` or `-d` - Run as daemon
- `--interval N` or `-i N` - Update interval in seconds (default: 5)

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
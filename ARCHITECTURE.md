# Architecture Documentation

## Overview

The raspi-info-display application is a modular Rust application designed to display system information on SSD1306 OLED displays connected to a Raspberry Pi via I2C. The application supports multiple displays through the TCA9548A I2C multiplexer.

## Core Architecture

### Module Structure

```
src/
├── main.rs           # Main application entry point and screen implementations
├── tca9548a.rs       # TCA9548A I2C multiplexer driver
└── lib.rs            # Library module exports
```

### Key Components

#### 1. Screen Trait System
The application uses a trait-based architecture for defining information screens:

```rust
trait Screen {
    fn name(&self) -> &'static str;
    fn title(&self) -> Result<String>;
    fn render(&self, sys: &System) -> Result<String>;
}
```

Each screen implements this trait to provide:
- `name()`: A unique identifier for the screen
- `title()`: The display title (shown at the top of the OLED)
- `render()`: The content to display (system information)

#### 2. Screen Manager
The `ScreenManager` handles:
- Screen rotation based on configured duration
- Tracking current screen index
- Managing the list of enabled screens

#### 3. Display Pipeline
1. **I2C Initialization**: Sets up communication with the OLED display
2. **Multiplexer Selection** (optional): Selects the appropriate channel on TCA9548A
3. **Screen Rendering**: Cycles through enabled screens, rendering content
4. **Display Update**: Flushes rendered content to the OLED display

### Current Screens

- **NetworkScreen**: Hostname, domain, IP address, MAC address
- **SystemScreen**: CPU temp, uptime, memory usage, boot partition
- **StorageScreen**: Disk usage across all mounted filesystems
- **HardwareScreen**: Pi model, serial number, firmware version
- **TemperatureScreen**: CPU/GPU temps, frequency, throttling status
- **GpioScreen**: I2C devices, GPIO states, SPI devices, 1-Wire sensors
- **OverviewScreen**: Combined view of essential information

## Adding a New Information Screen

### Step-by-Step Guide

#### 1. Create the Screen Structure
Add a new struct for your screen in `main.rs`:

```rust
struct MyCustomScreen;
```

#### 2. Implement the Screen Trait
Implement the required methods for your screen:

```rust
impl Screen for MyCustomScreen {
    fn name(&self) -> &'static str {
        "custom"  // Unique identifier for CLI arguments
    }
    
    fn title(&self) -> Result<String> {
        Ok("Custom Info".to_string())  // Display title
    }
    
    fn render(&self, sys: &System) -> Result<String> {
        // Gather your custom information
        let custom_data = get_custom_data()?;
        
        // Format for display (max ~5 lines for 128x64 display)
        Ok(format!(
            "Data 1: {}\nData 2: {}\nData 3: {}",
            custom_data.field1,
            custom_data.field2,
            custom_data.field3
        ))
    }
}
```

#### 3. Add Helper Functions (if needed)
Create any helper functions to gather your custom data:

```rust
fn get_custom_data() -> Result<CustomData> {
    // Read from files, run commands, query APIs, etc.
    let data = fs::read_to_string("/proc/custom")?;
    // Process and return data
    Ok(CustomData { ... })
}
```

#### 4. Register the Screen in ScreenManager
Update the `ScreenManager::new()` method to include your screen:

```rust
impl ScreenManager {
    fn new(enabled_screens: Vec<&str>, screen_duration_secs: u64) -> Self {
        let mut screens: Vec<Box<dyn Screen>> = Vec::new();
        
        for screen_name in enabled_screens {
            match screen_name {
                "network" => screens.push(Box::new(NetworkScreen)),
                "system" => screens.push(Box::new(SystemScreen)),
                // ... other screens ...
                "custom" => screens.push(Box::new(MyCustomScreen)),  // Add your screen
                _ => {}
            }
        }
        // ... rest of implementation
    }
}
```

#### 5. Add Command-Line Support
Update the argument parser in `main()` to recognize your screen:

```rust
match args[i].as_str() {
    // ... existing options ...
    "--custom" => enabled_screens.push("custom"),
    // ... rest of options ...
}
```

Also update the help text:

```rust
"--help" | "-h" => {
    // ... existing help ...
    println!("  --custom             Enable custom information screen");
    // ...
}
```

#### 6. Update the --screens Parser
Ensure your screen name is recognized in the comma-separated list:

```rust
"--screens" => {
    if i + 1 < args.len() {
        enabled_screens = args[i + 1].split(',').collect();
        // Now supports: --screens network,system,custom
    }
}
```

### Best Practices for New Screens

#### Display Constraints
- **Line Limit**: 128x64 displays can show ~5 lines of content with FONT_6X10
- **Character Width**: Approximately 21 characters per line
- **Title Space**: The title takes the first line, leaving 4 lines for content

#### Data Gathering Tips
1. **File Reading**: Use `fs::read_to_string()` for `/proc` and `/sys` files
2. **Command Execution**: Use `std::process::Command` for system commands
3. **Error Handling**: Return "N/A" or "Unknown" for unavailable data
4. **Caching**: Consider caching expensive operations if update interval is short

#### Example Data Sources
- **System Files**: `/proc/meminfo`, `/proc/cpuinfo`, `/sys/class/thermal/*`
- **Commands**: `vcgencmd`, `hostname`, `df`, `free`, `uptime`
- **GPIO**: `/sys/class/gpio/*`, `/sys/bus/i2c/devices/*`
- **Network**: `get_if_addrs` crate, `/proc/net/*`

### Testing Your New Screen

1. **Compile and Test**:
   ```bash
   cargo build --release
   sudo ./target/release/info_display --custom
   ```

2. **Test with Multiple Screens**:
   ```bash
   sudo ./target/release/info_display --custom --network --system
   ```

3. **Test with Multiplexer** (if using TCA9548A):
   ```bash
   sudo ./target/release/info_display --mux --mux-channel 0 --custom
   ```

## TCA9548A Multiplexer Support

### Architecture
The multiplexer support is implemented through:
1. **tca9548a.rs Module**: Provides channel selection and multiplexer control
2. **Shared I2C Bus**: Uses `Arc<Mutex<I2cdev>>` for thread-safe sharing
3. **Channel Selection**: Sets the appropriate channel before display initialization

### Usage Flow
1. Initialize shared I2C bus
2. Create Tca9548a instance with specified address
3. Select desired channel (0-7)
4. Initialize display on selected channel
5. Keep multiplexer handle alive during display operations

### Adding Multiplexer Support to Custom Screens
No additional work needed! The multiplexer is handled at the display initialization level, transparent to screen implementations.

## Configuration Philosophy

The application follows these design principles:
1. **Zero Configuration**: Works out-of-the-box with sensible defaults
2. **CLI-Driven**: All options configurable via command-line arguments
3. **Modular Screens**: Easy to add/remove screens without affecting others
4. **Fail-Safe**: Gracefully handles missing sensors or unavailable data
5. **Resource Efficient**: Minimal CPU and memory usage

## Future Extension Points

### Potential Enhancements
1. **Configuration File**: Add support for a config file (e.g., `/etc/info-display.conf`)
2. **Screen Templates**: Create a macro for easier screen definition
3. **Dynamic Loading**: Support for plugin-based screens
4. **Remote Data**: Add network-based data sources (APIs, MQTT, etc.)
5. **Graphics**: Add support for simple graphs or charts
6. **Multi-Display Layouts**: Different content on different multiplexer channels

### Adding External Data Sources
To add support for external data sources:
1. Add necessary dependencies to `Cargo.toml`
2. Create async helper functions if needed
3. Handle network timeouts gracefully
4. Cache results to avoid blocking the display loop

## Debugging Tips

### Common Issues
1. **I2C Permission Denied**: Run with `sudo` or add user to `i2c` group
2. **Display Not Found**: Check I2C address (usually 0x3C or 0x3D)
3. **Multiplexer Issues**: Verify with `i2cdetect -y 1`
4. **Screen Not Showing**: Ensure screen name matches in trait and CLI parser

### Debug Commands
```bash
# Check I2C devices
sudo i2cdetect -y 1

# Test multiplexer channels
sudo ./scripts/test-tca9548a.sh

# Run with single screen for testing
sudo ./target/release/info_display --custom --interval 1

# Check system logs
journalctl -u info-display.service -f
```

## Performance Considerations

### Resource Usage
- **Memory**: ~2-5MB RSS typical usage
- **CPU**: <1% average, spikes during screen updates
- **I2C Bus**: ~100-400 kHz communication speed

### Optimization Tips
1. Cache expensive operations between updates
2. Use lazy evaluation for rarely-changing data
3. Minimize string allocations in render loop
4. Consider longer update intervals for battery-powered setups

## Contributing

When contributing new screens:
1. Follow the existing code style
2. Add documentation for new data sources
3. Test on actual hardware if possible
4. Consider display constraints (21x5 characters)
5. Handle errors gracefully (no panics)
6. Update this document with your additions
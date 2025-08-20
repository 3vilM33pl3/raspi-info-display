use anyhow::Result;
use sysinfo::System;
use crate::system_info::*;

// Screen trait for modular display screens
pub trait Screen {
    fn name(&self) -> &'static str;
    fn title(&self) -> Result<String> {
        Ok(self.name().to_string())
    }
    fn render(&self, sys: &System) -> Result<String>;
}

// Network information screen
pub struct NetworkScreen;

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
pub struct SystemScreen;

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
pub struct StorageScreen;

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
pub struct OverviewScreen;

impl Screen for OverviewScreen {
    fn name(&self) -> &'static str {
        "overview"
    }
    
    fn title(&self) -> Result<String> {
        // Use hostname as title for overview screen
        Ok(hostname::get()
            .unwrap()
            .to_string_lossy()
            .into_owned())
    }
    
    fn render(&self, sys: &System) -> Result<String> {
        let ip_address = get_ip_address()?;
        let cpu_temp = get_cpu_temp().unwrap_or_else(|_| "N/A".to_string());
        let memory_info = get_memory_info(sys);
        let disk_usage = get_disk_usage();
        let uptime = get_uptime();
        
        Ok(format!(
            "{}\n{}\n{}\n{}\nUp: {}",
            ip_address, cpu_temp, memory_info, disk_usage, uptime
        ))
    }
}

// Hardware information screen
pub struct HardwareScreen;

impl Screen for HardwareScreen {
    fn name(&self) -> &'static str {
        "hardware"
    }
    
    fn render(&self, _sys: &System) -> Result<String> {
        let pi_model = get_pi_model();
        let serial = get_serial_number();
        let firmware = get_firmware_version();
        
        // Extract model name (remove "Raspberry Pi" prefix if present)
        let short_model = if pi_model.starts_with("Raspberry Pi ") {
            pi_model.strip_prefix("Raspberry Pi ").unwrap_or(&pi_model)
        } else {
            &pi_model
        };
        
        // Truncate serial to last 8 characters if longer
        let short_serial = if serial.len() > 8 {
            &serial[serial.len() - 8..]
        } else {
            &serial
        };
        
        // Extract year from firmware version if it contains a date
        let short_firmware = if firmware.contains("202") {
            // Try to find a 4-digit year
            if let Some(year_pos) = firmware.find("202") {
                firmware[year_pos..year_pos + 4].to_string()
            } else {
                firmware
            }
        } else {
            firmware
        };
        
        Ok(format!(
            "Model: {}\nSerial: {}\nFW: {}",
            short_model, short_serial, short_firmware
        ))
    }
}

// Temperature information screen
pub struct TemperatureScreen;

impl Screen for TemperatureScreen {
    fn name(&self) -> &'static str {
        "temperature"
    }
    
    fn render(&self, _sys: &System) -> Result<String> {
        let cpu_temp = get_cpu_temp().unwrap_or_else(|_| "N/A".to_string());
        let gpu_temp = get_gpu_temp();
        let cpu_freq = get_cpu_freq();
        let throttle = get_throttle_status();
        
        // Truncate throttle status if too long
        let short_throttle = if throttle.len() > 20 {
            format!("{}...", &throttle[..17])
        } else {
            throttle
        };
        
        Ok(format!(
            "CPU: {} ({})\nGPU: {}\nThrottle: {}",
            cpu_temp, cpu_freq, gpu_temp, short_throttle
        ))
    }
}

// GPIO and sensor information screen
pub struct GPIOScreen;

impl Screen for GPIOScreen {
    fn name(&self) -> &'static str {
        "gpio"
    }
    
    fn render(&self, _sys: &System) -> Result<String> {
        let i2c_devices = get_i2c_devices();
        let gpio_states = get_gpio_states();
        let spi_devices = get_spi_devices();
        let wire_sensors = get_1wire_sensors();
        
        // Truncate long lists
        let short_i2c = if i2c_devices.len() > 15 {
            format!("{}...", &i2c_devices[..12])
        } else {
            i2c_devices
        };
        
        let short_gpio = if gpio_states.len() > 20 {
            format!("{}...", &gpio_states[..17])
        } else {
            gpio_states
        };
        
        Ok(format!(
            "I2C: {}\nGPIO: {}\nSPI: {}\n1-Wire: {}",
            short_i2c, short_gpio, spi_devices, wire_sensors
        ))
    }
}
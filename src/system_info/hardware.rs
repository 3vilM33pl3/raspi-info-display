use std::fs;

pub fn get_pi_model() -> String {
    // Try reading from device tree first
    if let Ok(model) = fs::read_to_string("/proc/device-tree/model") {
        let model_clean = model.replace('\0', "").trim().to_string();
        if !model_clean.is_empty() {
            return model_clean;
        }
    }
    
    // Fallback to cpuinfo
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("Model") {
                if let Some(model) = line.split(':').nth(1) {
                    return model.trim().to_string();
                }
            }
        }
    }
    
    "Unknown".to_string()
}

pub fn get_serial_number() -> String {
    // Try reading from device tree first
    if let Ok(serial) = fs::read_to_string("/proc/device-tree/serial-number") {
        let serial_clean = serial.replace('\0', "").trim().to_string();
        if !serial_clean.is_empty() {
            return serial_clean;
        }
    }
    
    // Fallback to cpuinfo
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("Serial") {
                if let Some(serial) = line.split(':').nth(1) {
                    return serial.trim().to_string();
                }
            }
        }
    }
    
    "Unknown".to_string()
}

pub fn get_firmware_version() -> String {
    match std::process::Command::new("vcgencmd")
        .arg("version")
        .output()
    {
        Ok(output) => {
            let version_str = String::from_utf8_lossy(&output.stdout);
            // Extract just the date part from the firmware version
            for line in version_str.lines() {
                if line.contains("version") {
                    if let Some(date_start) = line.find("(") {
                        if let Some(date_end) = line.find(")") {
                            return line[date_start + 1..date_end].to_string();
                        }
                    }
                }
            }
            "Unknown".to_string()
        }
        Err(_) => "N/A".to_string()
    }
}

pub fn get_boot_partition() -> String {
    // Try using findmnt first
    if let Ok(output) = std::process::Command::new("findmnt")
        .arg("-n")
        .arg("-o")
        .arg("SOURCE")
        .arg("/boot")
        .output()
    {
        let device = String::from_utf8_lossy(&output.stdout);
        if !device.trim().is_empty() {
            return device.trim().to_string();
        }
    }
    
    // Fallback to /proc/mounts
    if let Ok(contents) = fs::read_to_string("/proc/mounts") {
        for line in contents.lines() {
            if line.contains(" /boot ") {
                if let Some(device) = line.split_whitespace().next() {
                    return device.to_string();
                }
            }
        }
    }
    
    "Unknown".to_string()
}
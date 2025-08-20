use anyhow::Result;
use std::fs;

pub fn get_cpu_temp() -> Result<String> {
    let temp_str = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?;
    let temp: i32 = temp_str.trim().parse()?;
    let temp_celsius = temp / 1000;
    Ok(format!("{}°C", temp_celsius))
}

pub fn get_gpu_temp() -> String {
    match std::process::Command::new("vcgencmd")
        .arg("measure_temp")
        .output()
    {
        Ok(output) => {
            let temp_str = String::from_utf8_lossy(&output.stdout);
            if let Some(temp_part) = temp_str.strip_prefix("temp=") {
                if let Some(temp_val) = temp_part.strip_suffix("'C\n") {
                    if let Ok(temp_float) = temp_val.parse::<f32>() {
                        return format!("{:.1}°C", temp_float);
                    }
                }
            }
            "N/A".to_string()
        }
        Err(_) => "N/A".to_string()
    }
}

pub fn get_throttle_status() -> String {
    match std::process::Command::new("vcgencmd")
        .arg("get_throttled")
        .output()
    {
        Ok(output) => {
            let throttle_str = String::from_utf8_lossy(&output.stdout);
            if let Some(hex_part) = throttle_str.strip_prefix("throttled=0x") {
                if let Ok(throttle_val) = u32::from_str_radix(hex_part.trim(), 16) {
                    if throttle_val == 0 {
                        return "None".to_string();
                    } else {
                        let mut status = Vec::new();
                        if throttle_val & 0x1 != 0 { status.push("Under-voltage"); }
                        if throttle_val & 0x2 != 0 { status.push("ARM freq capped"); }
                        if throttle_val & 0x4 != 0 { status.push("Currently throttled"); }
                        if throttle_val & 0x8 != 0 { status.push("Soft temp limit"); }
                        return status.join(", ");
                    }
                }
            }
            "Unknown".to_string()
        }
        Err(_) => "N/A".to_string()
    }
}

pub fn get_cpu_freq() -> String {
    match std::process::Command::new("vcgencmd")
        .arg("measure_clock")
        .arg("arm")
        .output()
    {
        Ok(output) => {
            let freq_str = String::from_utf8_lossy(&output.stdout);
            if let Some(freq_part) = freq_str.strip_prefix("frequency(48)=") {
                if let Ok(freq_hz) = freq_part.trim().parse::<u64>() {
                    let freq_mhz = freq_hz / 1_000_000;
                    return format!("{} MHz", freq_mhz);
                }
            }
            "N/A".to_string()
        }
        Err(_) => "N/A".to_string()
    }
}

pub fn get_i2c_devices() -> String {
    match std::process::Command::new("i2cdetect")
        .arg("-y")
        .arg("1")
        .output()
    {
        Ok(output) => {
            let detect_str = String::from_utf8_lossy(&output.stdout);
            let mut devices = Vec::new();
            
            for line in detect_str.lines() {
                if line.starts_with(char::is_numeric) {
                    for addr in line.split_whitespace().skip(1) {
                        if addr != "--" && addr.len() == 2 {
                            if let Ok(_) = u8::from_str_radix(addr, 16) {
                                devices.push(format!("0x{}", addr));
                            }
                        }
                    }
                }
            }
            
            if devices.is_empty() {
                "None".to_string()
            } else {
                devices.join(", ")
            }
        }
        Err(_) => "N/A".to_string()
    }
}

pub fn get_gpio_states() -> String {
    let mut states = Vec::new();
    
    // Check some common GPIO pins
    let pins = [2, 3, 4, 17, 18, 27, 22, 23, 24, 25];
    
    for pin in &pins {
        let export_path = format!("/sys/class/gpio/gpio{}/value", pin);
        if let Ok(value) = fs::read_to_string(export_path) {
            let state = if value.trim() == "1" { "H" } else { "L" };
            states.push(format!("{}: {}", pin, state));
        }
    }
    
    if states.is_empty() {
        "None exported".to_string()
    } else {
        states.join(", ")
    }
}

pub fn get_spi_devices() -> String {
    match fs::read_dir("/dev") {
        Ok(entries) => {
            let mut devices = Vec::new();
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("spidev") {
                    devices.push(name);
                }
            }
            if devices.is_empty() {
                "None".to_string()
            } else {
                devices.join(", ")
            }
        }
        Err(_) => "N/A".to_string()
    }
}

pub fn get_1wire_sensors() -> String {
    match fs::read_dir("/sys/bus/w1/devices") {
        Ok(entries) => {
            let mut sensors = Vec::new();
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name != "w1_bus_master1" {
                    // Try to read temperature if it's a temperature sensor
                    let temp_path = format!("/sys/bus/w1/devices/{}/w1_slave", name);
                    if let Ok(content) = fs::read_to_string(temp_path) {
                        if content.contains("YES") {
                            if let Some(temp_pos) = content.find("t=") {
                                if let Ok(temp_raw) = content[temp_pos + 2..].trim().parse::<i32>() {
                                    let temp_c = temp_raw as f32 / 1000.0;
                                    sensors.push(format!("{}: {:.1}°C", &name[..8], temp_c));
                                    continue;
                                }
                            }
                        }
                    }
                    // If not a temperature sensor or can't read temp, just show the ID
                    sensors.push(name);
                }
            }
            if sensors.is_empty() {
                "None".to_string()
            } else {
                sensors.join(", ")
            }
        }
        Err(_) => "None".to_string()
    }
}
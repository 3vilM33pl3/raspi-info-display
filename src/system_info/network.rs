use anyhow::Result;
use get_if_addrs::get_if_addrs;
use std::fs;

pub fn get_ip_address() -> Result<String> {
    let interfaces = get_if_addrs()?;
    
    for interface in interfaces {
        if !interface.is_loopback() && interface.ip().is_ipv4() {
            return Ok(interface.ip().to_string());
        }
    }
    
    Ok("N/A".to_string())
}

pub fn get_domain() -> String {
    // Try to read from /etc/resolv.conf first
    if let Ok(contents) = fs::read_to_string("/etc/resolv.conf") {
        for line in contents.lines() {
            if line.trim().starts_with("search ") {
                if let Some(domain) = line.split_whitespace().nth(1) {
                    return domain.to_string();
                }
            }
        }
    }
    
    // Fallback to hostname command
    if let Ok(output) = std::process::Command::new("hostname").arg("-d").output() {
        let domain = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !domain.is_empty() {
            return domain;
        }
    }
    
    "local".to_string()
}

pub fn get_mac_address() -> String {
    // Look for the first ethernet interface
    let interfaces = ["eth0", "enp", "ens"];
    
    for interface_prefix in &interfaces {
        // Try to find interface that starts with this prefix
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(interface_prefix) {
                    let mac_path = format!("/sys/class/net/{}/address", name);
                    if let Ok(mac) = fs::read_to_string(mac_path) {
                        return mac.trim().to_uppercase();
                    }
                }
            }
        }
    }
    
    // Fallback: try eth0 specifically
    if let Ok(mac) = fs::read_to_string("/sys/class/net/eth0/address") {
        return mac.trim().to_uppercase();
    }
    
    "N/A".to_string()
}
use std::fs;

pub fn get_uptime() -> String {
    if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
        if let Some(uptime_seconds_str) = uptime_str.split_whitespace().next() {
            if let Ok(uptime_seconds) = uptime_seconds_str.parse::<f64>() {
                let days = (uptime_seconds / 86400.0) as u32;
                let hours = ((uptime_seconds % 86400.0) / 3600.0) as u32;
                let minutes = ((uptime_seconds % 3600.0) / 60.0) as u32;
                
                if days > 0 {
                    return format!("{}d{}h{}m", days, hours, minutes);
                } else if hours > 0 {
                    return format!("{}h{}m", hours, minutes);
                } else {
                    return format!("{}m", minutes);
                }
            }
        }
    }
    
    "Unknown".to_string()
}
use sysinfo::{System, Disks};

pub fn get_memory_info(sys: &System) -> String {
    let used_mem = sys.used_memory();
    let total_mem = sys.total_memory();
    let used_mb = used_mem / 1024 / 1024;
    let total_mb = total_mem / 1024 / 1024;
    format!("{}/{}MB", used_mb, total_mb)
}

pub fn get_disk_usage() -> String {
    let disks = Disks::new_with_refreshed_list();
    let mut total_space = 0;
    let mut used_space = 0;
    
    for disk in &disks {
        total_space += disk.total_space();
        used_space += disk.total_space() - disk.available_space();
    }
    
    if total_space > 0 {
        let used_gb = used_space / 1024 / 1024 / 1024;
        let total_gb = total_space / 1024 / 1024 / 1024;
        format!("{}/{}GB", used_gb, total_gb)
    } else {
        "N/A".to_string()
    }
}
// src/core/platform/resources.rs
use sysinfo::System;

/// System resources information
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemResources {
    pub memory: MemoryInfo,
    pub cpu: CpuInfo,
    pub disk: Vec<DiskInfo>,
    pub system: SystemInfo,
}

/// Memory information
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub total_formatted: String,
    pub used_formatted: String,
    pub available_formatted: String,
    pub usage_percent: f32,
}

/// CPU information
#[derive(Debug, Clone, serde::Serialize)]
pub struct CpuInfo {
    pub cores: usize,
    pub usage: f32,
    pub frequency: Option<u64>,
    pub brand: Option<String>,
}

/// Disk information
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub total_formatted: String,
    pub used_formatted: String,
    pub available_formatted: String,
    pub usage_percent: f32,
    pub file_system: Option<String>,
}

/// System information
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemInfo {
    pub name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub uptime: u64,
    pub uptime_formatted: String,
    pub hostname: Option<String>,
}

/// Get system resources
pub fn get_system_resources() -> SystemResources {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let memory = get_memory_info(&sys);
    let cpu = get_cpu_info(&sys);
    let disk = get_disk_info();
    let system = get_system_info();
    
    SystemResources {
        memory,
        cpu,
        disk,
        system,
    }
}

/// Get memory information
pub fn get_memory_info(sys: &System) -> MemoryInfo {
    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = total - used;
    let usage_percent = if total > 0 {
        (used as f32 / total as f32) * 100.0
    } else {
        0.0
    };
    
    MemoryInfo {
        total,
        used,
        available,
        total_formatted: format_bytes(total),
        used_formatted: format_bytes(used),
        available_formatted: format_bytes(available),
        usage_percent,
    }
}

/// Get CPU information
pub fn get_cpu_info(sys: &System) -> CpuInfo {
    let cores = sys.cpus().len();
    let usage = sys.global_cpu_usage();
    
    // Get frequency from first CPU if available
    let frequency = sys.cpus().first().map(|cpu| cpu.frequency());
    
    // Get brand from first CPU if available
    let brand = sys.cpus().first().and_then(|cpu| {
        let brand_str = cpu.brand();
        if brand_str.is_empty() {
            None
        } else {
            Some(brand_str.to_string())
        }
    });
    
    CpuInfo {
        cores,
        usage,
        frequency,
        brand,
    }
}

/// Get disk information - simplified version without disk access
pub fn get_disk_info() -> Vec<DiskInfo> {
    // Return a placeholder since disk information is not available
    vec![DiskInfo {
        name: "Disk information not available".to_string(),
        mount_point: "".to_string(),
        total: 0,
        used: 0,
        available: 0,
        total_formatted: "0 B".to_string(),
        used_formatted: "0 B".to_string(),
        available_formatted: "0 B".to_string(),
        usage_percent: 0.0,
        file_system: None,
    }]
}

/// Get system information
pub fn get_system_info() -> SystemInfo {
    // These are associated functions in newer sysinfo
    let name = System::name().map(|s| s.to_string());
    let kernel_version = System::kernel_version().map(|s| s.to_string());
    let os_version = System::os_version().map(|s| s.to_string());
    let uptime = System::uptime();
    let hostname = System::host_name().map(|s| s.to_string());
    
    SystemInfo {
        name,
        kernel_version,
        os_version,
        uptime,
        uptime_formatted: format_uptime(uptime),
        hostname,
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format uptime to human-readable string
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", seconds)
    }
}

/// Quick resource snapshot (lightweight)
pub fn get_quick_resources() -> serde_json::Value {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.refresh_cpu_all();
    
    let memory = get_memory_info(&sys);
    let cpu = get_cpu_info(&sys);
    
    serde_json::json!({
        "memory": {
            "total": memory.total_formatted,
            "used": memory.used_formatted,
            "available": memory.available_formatted,
            "usage_percent": memory.usage_percent,
        },
        "cpu": {
            "usage": cpu.usage,
            "cores": cpu.cores,
        },
        "timestamp": chrono::Local::now().to_rfc3339(),
    })
}

/// Check if system has enough resources for a model
pub fn check_model_compatibility(
    model_size_gb: f64,
) -> Result<ModelCompatibility, String> {
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    let total_memory = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0); // Convert to GB
    let available_memory = sys.available_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    
    // Check if there's enough memory
    let enough_memory = available_memory >= model_size_gb * 1.5; // 1.5x buffer for overhead
    let recommended_memory = model_size_gb * 1.5;
    
    let status = if enough_memory {
        "sufficient".to_string()
    } else if available_memory >= model_size_gb {
        "limited".to_string()
    } else {
        "insufficient".to_string()
    };
    
    Ok(ModelCompatibility {
        enough_memory,
        total_memory_gb: total_memory,
        available_memory_gb: available_memory,
        model_size_gb,
        recommended_memory_gb: recommended_memory,
        status,
    })
}

/// Model compatibility result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelCompatibility {
    pub enough_memory: bool,
    pub total_memory_gb: f64,
    pub available_memory_gb: f64,
    pub model_size_gb: f64,
    pub recommended_memory_gb: f64,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes_across_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn formats_uptime_for_each_unit() {
        assert_eq!(format_uptime(45), "45s");
        assert_eq!(format_uptime(90), "1m");
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(3661), "1h 1m");
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }
}
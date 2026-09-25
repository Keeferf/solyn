// src/core/platform/detector.rs
use std::env;

/// Detect the current operating system
pub fn detect_operating_system() -> String {
    #[cfg(target_os = "windows")]
    { "windows".to_string() }
    #[cfg(target_os = "linux")]
    { "linux".to_string() }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    { "unknown".to_string() }
}

/// Get the platform family (windows, unix, etc.)
pub fn get_platform_family() -> String {
    #[cfg(target_os = "windows")]
    { "windows".to_string() }
    #[cfg(target_os = "linux")]
    { "unix".to_string() }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    { "unknown".to_string() }
}

/// Check if the current platform is Windows
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// Check if the current platform is Linux
pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

/// Get a display name for the current platform
pub fn get_platform_display_name() -> String {
    #[cfg(target_os = "windows")]
    { 
        if let Ok(version) = get_windows_version() {
            format!("Windows {}", version)
        } else {
            "Windows".to_string()
        }
    }
    #[cfg(target_os = "linux")]
    { 
        if let Ok(distro) = get_linux_distro() {
            distro
        } else {
            "Linux".to_string()
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    { "Unknown".to_string() }
}

/// Get an icon/emoji for the current platform
pub fn get_platform_icon() -> String {
    #[cfg(target_os = "windows")]
    { "🪟".to_string() }
    #[cfg(target_os = "linux")]
    { "🐧".to_string() }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    { "💻".to_string() }
}

/// Get Windows version (Windows only)
#[cfg(target_os = "windows")]
fn get_windows_version() -> Result<String, String> {
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;
    
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .map_err(|e| format!("Failed to open registry key: {}", e))?;
    
    let product_name: String = key.get_value("ProductName")
        .unwrap_or_else(|_| "Unknown".to_string());
    
    let release_id: String = key.get_value("ReleaseId")
        .unwrap_or_else(|_| "Unknown".to_string());
    
    Ok(format!("{} (Build {})", product_name, release_id))
}

/// Get Linux distribution (Linux only)
#[cfg(target_os = "linux")]
fn get_linux_distro() -> Result<String, String> {
    use std::fs;
    
    // Try /etc/os-release first
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let name = line.replace("PRETTY_NAME=", "")
                    .trim_matches('"')
                    .to_string();
                return Ok(name);
            }
        }
    }
    
    // Try /etc/lsb-release
    if let Ok(content) = fs::read_to_string("/etc/lsb-release") {
        for line in content.lines() {
            if line.starts_with("DISTRIB_DESCRIPTION=") {
                let name = line.replace("DISTRIB_DESCRIPTION=", "")
                    .trim_matches('"')
                    .to_string();
                return Ok(name);
            }
        }
    }
    
    // Fallback: try uname
    use std::process::Command;
    let output = Command::new("uname")
        .args(&["-s", "-r"])
        .output()
        .map_err(|e| format!("Failed to get system info: {}", e))?;
    
    let info = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(info)
}

/// Get detailed platform information
pub fn get_platform_info_detailed() -> PlatformInfo {
    let os = detect_operating_system();
    let family = get_platform_family();
    let display_name = get_platform_display_name();
    let icon = get_platform_icon();
    
    let mut details = std::collections::HashMap::new();
    details.insert("os".to_string(), os.clone());
    details.insert("family".to_string(), family);
    details.insert("display_name".to_string(), display_name);
    details.insert("icon".to_string(), icon);
    
    // Add architecture
    details.insert("architecture".to_string(), std::env::consts::ARCH.to_string());
    
    // Add additional platform-specific info
    #[cfg(target_os = "windows")]
    {
        if let Ok(version) = get_windows_version() {
            details.insert("version".to_string(), version);
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(distro) = get_linux_distro() {
            details.insert("distribution".to_string(), distro);
        }
    }
    
    PlatformInfo { details }
}

/// Platform information structure
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlatformInfo {
    pub details: std::collections::HashMap<String, String>,
}

impl PlatformInfo {
    /// Get the operating system name
    pub fn os(&self) -> Option<&str> {
        self.details.get("os").map(|s| s.as_str())
    }
    
    /// Get the platform family
    pub fn family(&self) -> Option<&str> {
        self.details.get("family").map(|s| s.as_str())
    }
    
    /// Get the display name
    pub fn display_name(&self) -> Option<&str> {
        self.details.get("display_name").map(|s| s.as_str())
    }
    
    /// Get the platform icon
    pub fn icon(&self) -> Option<&str> {
        self.details.get("icon").map(|s| s.as_str())
    }
    
    /// Get the architecture
    pub fn architecture(&self) -> Option<&str> {
        self.details.get("architecture").map(|s| s.as_str())
    }
}

/// Check if running in a container (Docker, etc.)
pub fn is_running_in_container() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(content) = fs::read_to_string("/proc/1/cgroup") {
            content.lines().any(|line| {
                line.contains("docker") || 
                line.contains("lxc") || 
                line.contains("kubepods")
            })
        } else {
            false
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Get terminal info (for display purposes)
pub fn get_terminal_info() -> String {
    if let Ok(term) = env::var("TERM") {
        if !term.is_empty() {
            return term;
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(session) = env::var("SESSIONNAME") {
            if !session.is_empty() {
                return "Windows Terminal".to_string();
            }
        }
        "Command Prompt".to_string()
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        "Unknown Terminal".to_string()
    }
}

/// Get user shell
pub fn get_user_shell() -> String {
    #[cfg(target_os = "windows")]
    {
        env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        env::var("SHELL").unwrap_or_else(|_| "unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_flags_are_consistent() {
        let os = detect_operating_system();
        assert_eq!(os == "windows", is_windows());
        assert_eq!(os == "linux", is_linux());
        assert!(matches!(os.as_str(), "windows" | "linux" | "unknown"));
    }

    #[test]
    fn family_matches_os() {
        let family = get_platform_family();
        if is_windows() {
            assert_eq!(family, "windows");
        } else if is_linux() {
            assert_eq!(family, "unix");
        }
    }

    #[test]
    fn platform_info_exposes_expected_fields() {
        let info = get_platform_info_detailed();
        assert!(info.os().is_some());
        assert_eq!(
            info.family().unwrap().to_string(),
            get_platform_family()
        );
        assert!(info.architecture().is_some());
        assert!(info.display_name().is_some());
        assert!(info.icon().is_some());
    }
}
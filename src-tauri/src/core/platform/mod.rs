pub mod detector;
pub mod resources;

pub use detector::{
    detect_operating_system,
    get_platform_family,
    is_windows,
    is_linux,
    get_platform_icon,
    get_platform_display_name,
};
pub use resources::{
    SystemResources,
    get_system_resources,
    get_memory_info,
    get_cpu_info,
    get_disk_info,
};
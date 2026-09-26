pub mod detector;
pub mod resources;

pub use detector::{
    detect_operating_system, get_platform_display_name, get_platform_family, get_platform_icon,
    is_linux, is_windows,
};
pub use resources::{
    get_cpu_info, get_disk_info, get_memory_info, get_system_resources, SystemResources,
};

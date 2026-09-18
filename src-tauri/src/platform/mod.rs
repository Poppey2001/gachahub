use std::fmt::{Display, Formatter};

pub enum Platform { Windows, Linux, MacOs, Other }

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Platform::Windows => "Windows",
            Platform::Linux => "Linux",
            Platform::MacOs => "macOS",
            Platform::Other => "Other",
        })
    }
}

pub fn current_platform() -> Platform {
    if cfg!(target_os = "windows") { Platform::Windows }
    else if cfg!(target_os = "linux") { Platform::Linux }
    else if cfg!(target_os = "macos") { Platform::MacOs }
    else { Platform::Other }
}

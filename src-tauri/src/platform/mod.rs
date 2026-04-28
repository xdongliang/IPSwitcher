use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub display_name: String,
    pub is_active: bool,
}

/// Trait for platform-specific network management operations.
pub trait NetworkManager {
    /// List all available network interfaces/services.
    fn list_interfaces(&self) -> Result<Vec<NetworkInterface>, AppError>;

    /// Get current IPv4 address for the given interface.
    fn get_current_config(&self, interface: &str) -> Result<CurrentNetworkConfig, AppError>;

    /// Apply static IP configuration to the given interface.
    fn apply_static_config(
        &self,
        interface: &str,
        ip: &str,
        mask: &str,
        gateway: &str,
        dns: &[String],
    ) -> Result<(), AppError>;

    /// Set the given interface to use DHCP.
    fn set_dhcp(&self, interface: &str) -> Result<(), AppError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentNetworkConfig {
    pub interface: String,
    pub ip_address: Option<String>,
    pub subnet_mask: Option<String>,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub is_dhcp: bool,
}

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOSNetworkManager;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsNetworkManager;

/// Factory to get the appropriate network manager for the current OS.
#[cfg(target_os = "macos")]
pub fn get_network_manager() -> impl NetworkManager {
    MacOSNetworkManager
}

#[cfg(target_os = "windows")]
pub fn get_network_manager() -> impl NetworkManager {
    WindowsNetworkManager
}

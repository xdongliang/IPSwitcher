use crate::error::AppError;
use crate::platform::{get_network_manager, NetworkInterface, NetworkManager};

#[tauri::command]
pub fn list_network_interfaces() -> Result<Vec<NetworkInterface>, AppError> {
    get_network_manager().list_interfaces()
}

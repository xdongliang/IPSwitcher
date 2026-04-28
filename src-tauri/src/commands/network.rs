use tauri::{AppHandle, State};

use crate::admin::{self, elevate_network_command};
use crate::error::AppError;
use crate::models::profile::{IpMode, Profile};
use crate::platform::{get_network_manager, CurrentNetworkConfig, NetworkManager};
use crate::storage::sqlite::ProfileRepository;
use crate::tray;

/// Core logic to apply a profile's network configuration.
/// Shared between the Tauri command and startup auto-apply.
pub fn do_apply_profile(profile: &Profile) -> Result<String, AppError> {
    let target_interface = profile
        .interface_name
        .clone()
        .ok_or_else(|| AppError::Validation("请指定要应用配置的网络接口".into()))?;

    let mgr = get_network_manager();

    match profile.ip_mode {
        IpMode::Manual => {
            let ip = profile.ip_address.as_deref().ok_or_else(|| {
                AppError::Validation("手动模式下必须提供IP地址".into())
            })?;
            let mask = profile.subnet_mask.as_deref().ok_or_else(|| {
                AppError::Validation("手动模式下必须提供子网掩码".into())
            })?;
            let gateway = profile.gateway.as_deref().ok_or_else(|| {
                AppError::Validation("手动模式下必须提供默认网关".into())
            })?;

            if !admin::is_elevated() {
                elevate_network_command(
                    &target_interface,
                    "Manual",
                    Some(ip),
                    Some(mask),
                    Some(gateway),
                    &profile.dns_servers,
                )?;
            } else {
                mgr.apply_static_config(
                    &target_interface,
                    ip,
                    mask,
                    gateway,
                    &profile.dns_servers,
                )?;
            }
            Ok(format!(
                "已切换到方案 \"{}\" — 静态IP: {} (接口: {})",
                profile.name, ip, target_interface
            ))
        }
        IpMode::Dhcp => {
            if !admin::is_elevated() {
                elevate_network_command(
                    &target_interface,
                    "Dhcp",
                    None,
                    None,
                    None,
                    &[],
                )?;
            } else {
                mgr.set_dhcp(&target_interface)?;
            }
            Ok(format!(
                "已切换到方案 \"{}\" — DHCP (接口: {})",
                profile.name, target_interface
            ))
        }
    }
}

#[tauri::command]
pub fn get_current_network_config(
    interface: Option<String>,
) -> Result<CurrentNetworkConfig, AppError> {
    let interfaces = get_network_manager().list_interfaces()?;
    let target = if let Some(ref iface) = interface {
        iface.clone()
    } else {
        interfaces
            .iter()
            .find(|i| i.is_active)
            .or_else(|| interfaces.first())
            .map(|i| i.name.clone())
            .ok_or_else(|| AppError::Network("未找到可用的网络接口".into()))?
    };

    get_network_manager().get_current_config(&target)
}

#[tauri::command]
pub fn apply_profile(
    app: AppHandle,
    repo: State<'_, ProfileRepository>,
    profile_id: String,
    interface: Option<String>,
) -> Result<String, AppError> {
    let mut profile = repo.get_by_id(&profile_id)?;
    if interface.is_some() {
        profile.interface_name = interface;
    }

    let msg = do_apply_profile(&profile)?;

    let _ = repo.set_active_profile_id(Some(&profile_id));
    let _ = tray::rebuild_tray_menu(&app);

    Ok(msg)
}

#[tauri::command]
pub fn get_active_profile_id(
    repo: State<'_, ProfileRepository>,
) -> Result<Option<String>, AppError> {
    repo.get_active_profile_id()
}

#[tauri::command]
pub fn set_active_profile_id(
    app: AppHandle,
    repo: State<'_, ProfileRepository>,
    profile_id: Option<String>,
) -> Result<(), AppError> {
    repo.set_active_profile_id(profile_id.as_deref())?;
    let _ = tray::rebuild_tray_menu(&app);
    Ok(())
}

#[tauri::command]
pub fn check_admin_status() -> Result<bool, AppError> {
    Ok(admin::is_elevated())
}

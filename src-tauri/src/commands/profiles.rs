use tauri::{AppHandle, State};
use uuid::Uuid;
use chrono::Utc;

use crate::error::AppError;
use crate::models::profile::{IpMode, Profile};
use crate::storage::sqlite::ProfileRepository;
use crate::tray;

#[tauri::command]
pub fn list_profiles(
    repo: State<'_, ProfileRepository>,
) -> Result<Vec<Profile>, AppError> {
    repo.list_all()
}

#[tauri::command]
pub fn get_profile(
    repo: State<'_, ProfileRepository>,
    id: String,
) -> Result<Profile, AppError> {
    repo.get_by_id(&id)
}

#[tauri::command]
pub fn create_profile(
    app: AppHandle,
    repo: State<'_, ProfileRepository>,
    name: String,
    ip_mode: String,
    ip_address: Option<String>,
    subnet_mask: Option<String>,
    gateway: Option<String>,
    dns_servers: Vec<String>,
    interface_name: Option<String>,
) -> Result<Profile, AppError> {
    let mode = match ip_mode.as_str() {
        "Manual" => IpMode::Manual,
        _ => IpMode::Dhcp,
    };

    let now = Utc::now().to_rfc3339();
    let profile = Profile {
        id: Uuid::new_v4().to_string(),
        name,
        ip_mode: mode,
        ip_address,
        subnet_mask,
        gateway,
        dns_servers,
        interface_name,
        created_at: now.clone(),
        updated_at: now,
    };

    profile
        .validate()
        .map_err(|e| AppError::Validation(e))?;

    repo.insert(&profile)?;
    let _ = tray::rebuild_tray_menu(&app);
    Ok(profile)
}

#[tauri::command]
pub fn update_profile(
    app: AppHandle,
    repo: State<'_, ProfileRepository>,
    id: String,
    name: String,
    ip_mode: String,
    ip_address: Option<String>,
    subnet_mask: Option<String>,
    gateway: Option<String>,
    dns_servers: Vec<String>,
    interface_name: Option<String>,
) -> Result<Profile, AppError> {
    let mode = match ip_mode.as_str() {
        "Manual" => IpMode::Manual,
        _ => IpMode::Dhcp,
    };

    let existing = repo.get_by_id(&id)?;

    let now = Utc::now().to_rfc3339();
    let profile = Profile {
        id,
        name,
        ip_mode: mode,
        ip_address,
        subnet_mask,
        gateway,
        dns_servers,
        interface_name,
        created_at: existing.created_at,
        updated_at: now,
    };

    profile
        .validate()
        .map_err(|e| AppError::Validation(e))?;

    repo.update(&profile)?;
    let _ = tray::rebuild_tray_menu(&app);
    Ok(profile)
}

#[tauri::command]
pub fn delete_profile(
    app: AppHandle,
    repo: State<'_, ProfileRepository>,
    id: String,
) -> Result<(), AppError> {
    // Clear active profile if deleting the active one
    if let Ok(Some(active_id)) = repo.get_active_profile_id() {
        if active_id == id {
            let _ = repo.set_active_profile_id(None);
        }
    }
    repo.delete(&id)?;
    let _ = tray::rebuild_tray_menu(&app);
    Ok(())
}

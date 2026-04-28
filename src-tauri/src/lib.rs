mod admin;
mod commands;
mod error;
mod macos_activate;
mod models;
mod platform;
mod storage;
mod tray;

use storage::sqlite::ProfileRepository;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let repo = ProfileRepository::new().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(repo)
        .setup(|app| {
            // macOS: hide Dock icon by setting Accessory activation policy.
            // Tauri defaults to Regular which overrides LSUIElement in Info.plist.
            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                let _ = app.handle().set_activation_policy(ActivationPolicy::Accessory);
            }

            if let Err(e) = tray::build_tray(app.handle()) {
                log::error!("Failed to build tray icon: {e}");
            }

            // Auto-apply last active profile on startup
            let repo = app.handle().state::<ProfileRepository>();
            if let Ok(Some(active_id)) = repo.get_active_profile_id() {
                if let Ok(profile) = repo.get_by_id(&active_id) {
                    match commands::network::do_apply_profile(&profile) {
                        Ok(msg) => log::info!("Startup auto-apply: {msg}"),
                        Err(e) => log::warn!("Startup auto-apply failed: {e}"),
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::profiles::list_profiles,
            commands::profiles::get_profile,
            commands::profiles::create_profile,
            commands::profiles::update_profile,
            commands::profiles::delete_profile,
            commands::interfaces::list_network_interfaces,
            commands::network::get_current_network_config,
            commands::network::apply_profile,
            commands::network::check_admin_status,
            commands::network::get_active_profile_id,
            commands::network::set_active_profile_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

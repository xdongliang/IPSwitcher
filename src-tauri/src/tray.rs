use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

use crate::macos_activate::activate_app;
use crate::storage::sqlite::ProfileRepository;

const TRAY_ID: &str = "main-tray";

/// Dedicated tray icon PNG (black on transparent, 32x32).
const TRAY_ICON_PNG: &[u8] = include_bytes!("../icons/tray-icon-32.png");

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        activate_app();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Build the full tray menu including profile switch items.
fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    let repo = app.state::<ProfileRepository>();
    let profiles = repo.list_all().unwrap_or_default();
    let active_id = repo.get_active_profile_id().unwrap_or(None);

    let mut menu_builder = MenuBuilder::new(app);

    if !profiles.is_empty() {
        for profile in &profiles {
            let is_active = active_id.as_deref() == Some(profile.id.as_str());
            let label = if is_active {
                format!("\u{2713} {}", profile.name)
            } else {
                format!("    {}", profile.name)
            };
            let item = MenuItemBuilder::with_id(
                &format!("switch_profile:{}", profile.id),
                label,
            )
            .build(app)?;
            menu_builder = menu_builder.item(&item);
        }
        menu_builder = menu_builder.separator();
    }

    let new_profile_item = MenuItemBuilder::with_id("new_profile", "新建方案...")
        .build(app)?;

    let show_item = MenuItemBuilder::with_id("show_window", "显示主窗口")
        .build(app)?;

    let quit_item = MenuItemBuilder::with_id("quit", "退出 IPSwitcher")
        .build(app)?;

    menu_builder
        .item(&new_profile_item)
        .separator()
        .item(&show_item)
        .separator()
        .item(&quit_item)
        .build()
}

pub fn build_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let icon = Image::from_bytes(TRAY_ICON_PNG)?;
    let menu = build_menu(app)?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .tooltip("IPSwitcher")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "show_window" => {
                    show_main_window(app);
                }
                "new_profile" => {
                    show_main_window(app);
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("tray-new-profile", ());
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                id if id.starts_with("switch_profile:") => {
                    let profile_id = id.strip_prefix("switch_profile:").unwrap_or("");
                    show_main_window(app);
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("tray-switch-profile", profile_id.to_string());
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        activate_app();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Rebuild the tray menu to reflect current profile list.
pub fn rebuild_tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

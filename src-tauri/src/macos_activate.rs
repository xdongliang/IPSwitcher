#[cfg(target_os = "macos")]
mod imp {
    use objc2::{class, msg_send};
    use objc2::runtime::AnyObject;

    /// Call [NSApp activateIgnoringOtherApps: YES] to bring the app's window
    /// to the foreground. This is required when using ActivationPolicy::Accessory
    /// because the app is treated as a background app and window.set_focus()
    /// alone will not bring it in front of other apps.
    pub fn activate_app() {
        unsafe {
            let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
            let _: () = msg_send![app, activateIgnoringOtherApps: true];
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    /// No-op on non-macOS platforms
    pub fn activate_app() {}
}

pub use imp::activate_app;

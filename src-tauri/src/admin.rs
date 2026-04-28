use crate::error::AppError;

/// Check if the current process is running with elevated privileges (root/admin).
pub fn is_elevated() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        // "net session" requires admin privileges on Windows
        std::process::Command::new("net")
            .arg("session")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        false
    }
}

/// Execute a network configuration command with elevated privileges.
pub fn elevate_network_command(
    service_name: &str,
    ip_mode: &str,
    ip: Option<&str>,
    mask: Option<&str>,
    gateway: Option<&str>,
    dns: &[String],
) -> Result<String, AppError> {
    #[cfg(target_os = "macos")]
    {
        let cmd = build_macos_command(service_name, ip_mode, ip, mask, gateway, dns);
        run_macos_elevated(&cmd)
    }
    #[cfg(target_os = "windows")]
    {
        let cmd = build_windows_command(service_name, ip_mode, ip, mask, gateway, dns);
        run_windows_elevated(&cmd)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err(AppError::Network("不支持的操作系统".into()))
    }
}

#[cfg(target_os = "macos")]
fn build_macos_command(
    service: &str,
    ip_mode: &str,
    ip: Option<&str>,
    mask: Option<&str>,
    gateway: Option<&str>,
    dns: &[String],
) -> String {
    if ip_mode == "Manual" {
        let ip = ip.unwrap_or("0.0.0.0");
        let mask = mask.unwrap_or("255.255.255.0");
        let gateway = gateway.unwrap_or("0.0.0.0");
        format!(
            "networksetup -setmanual \"{}\" {} {} {} && networksetup -setdnsservers \"{}\" {}",
            service, ip, mask, gateway, service, dns.join(" ")
        )
    } else {
        format!(
            "networksetup -setdhcp \"{}\" && networksetup -setdnsservers \"{}\" Empty",
            service, service
        )
    }
}

#[cfg(target_os = "macos")]
fn run_macos_elevated(command: &str) -> Result<String, AppError> {
    use std::process::Command;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "do shell script \"{}\" with administrator privileges",
            command.replace('"', "\\\"")
        ))
        .output()
        .map_err(|e| AppError::Network(format!("无法执行提权命令: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("User canceled") || stderr.contains("(-128)") {
            Err(AppError::PermissionDenied("用户取消了权限授权".into()))
        } else {
            Err(AppError::Network(format!("执行命令失败: {}", stderr)))
        }
    }
}

#[cfg(target_os = "windows")]
fn build_windows_command(
    service: &str,
    ip_mode: &str,
    ip: Option<&str>,
    mask: Option<&str>,
    gateway: Option<&str>,
    dns: &[String],
) -> String {
    if ip_mode == "Manual" {
        let ip = ip.unwrap_or("0.0.0.0");
        let mask = mask.unwrap_or("255.255.255.0");
        let gateway = gateway.unwrap_or("0.0.0.0");
        let mut cmd = format!(
            "netsh interface ip set address name=\"{}\" static {} {} {}",
            service, ip, mask, gateway
        );
        if let Some((first, rest)) = dns.split_first() {
            cmd.push_str(&format!(
                " && netsh interface ip set dns name=\"{}\" static {}",
                service, first
            ));
            for (i, d) in rest.iter().enumerate() {
                cmd.push_str(&format!(
                    " && netsh interface ip add dns name=\"{}\" {} index={}",
                    service,
                    d,
                    i + 2
                ));
            }
        }
        cmd
    } else {
        format!(
            "netsh interface ip set address name=\"{}\" source=dhcp && netsh interface ip set dns name=\"{}\" source=dhcp",
            service, service
        )
    }
}

#[cfg(target_os = "windows")]
fn run_windows_elevated(command: &str) -> Result<String, AppError> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Start-Process cmd -ArgumentList '/c {}' -Verb RunAs -Wait -WindowStyle Hidden",
                command
            ),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| AppError::Network(format!("无法执行提权命令: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(AppError::Network(format!(
            "执行命令失败: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

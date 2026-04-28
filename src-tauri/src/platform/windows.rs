use std::os::windows::process::CommandExt;
use std::process::Command;

use crate::error::AppError;
use crate::platform::{CurrentNetworkConfig, NetworkInterface, NetworkManager};

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 创建隐藏控制台窗口的 netsh 命令
fn netsh_command() -> Command {
    let mut cmd = Command::new("netsh");
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

pub struct WindowsNetworkManager;

impl NetworkManager for WindowsNetworkManager {
    fn list_interfaces(&self) -> Result<Vec<NetworkInterface>, AppError> {
        let output = netsh_command()
            .args(["interface", "ip", "show", "interfaces"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();

        // Parse netsh output: skip header lines, parse table rows
        for line in stdout.lines().skip(1) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("---") {
                continue;
            }

            // netsh output columns: Idx, Met, MTU, State, Name
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 5 {
                let name = parts[4..].join(" ");
                let state = parts[3];
                interfaces.push(NetworkInterface {
                    name: name.clone(),
                    display_name: name,
                    is_active: state == "connected" || state == "Connected",
                });
            }
        }

        Ok(interfaces)
    }

    fn get_current_config(&self, interface: &str) -> Result<CurrentNetworkConfig, AppError> {
        let output = netsh_command()
            .args(["interface", "ip", "show", "config", &format!("name=\"{}\"", interface)])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut config = CurrentNetworkConfig {
            interface: interface.to_string(),
            ip_address: None,
            subnet_mask: None,
            gateway: None,
            dns_servers: Vec::new(),
            is_dhcp: false,
        };

        for line in stdout.lines() {
            let trimmed = line.trim();

            if trimmed.contains("DHCP enabled") && trimmed.contains("Yes") {
                config.is_dhcp = true;
            }

            if trimmed.starts_with("IP Address:") {
                config.ip_address = trimmed
                    .split_once(':')
                    .map(|(_, v)| v.trim().to_string());
            }
            if trimmed.starts_with("Subnet Prefix:") {
                // Windows shows prefix length, but we store what's shown
                // For simplicity, leave subnet mask parsing to the raw value
                config.subnet_mask = trimmed
                    .split_once(':')
                    .map(|(_, v)| v.trim().to_string());
            }
            if trimmed.starts_with("Default Gateway:") {
                config.gateway = trimmed
                    .split_once(':')
                    .map(|(_, v)| v.trim().to_string());
            }
            if trimmed.starts_with("DNS Servers:") {
                continue; // Next lines will be DNS entries
            }
        }

        Ok(config)
    }

    fn apply_static_config(
        &self,
        interface: &str,
        ip: &str,
        mask: &str,
        gateway: &str,
        dns: &[String],
    ) -> Result<(), AppError> {
        // Step 1: Set static IP address
        let status = netsh_command()
            .args([
                "interface", "ip", "set", "address",
                &format!("name=\"{}\"", interface),
                "static", ip, mask, gateway,
            ])
            .status()?;

        if !status.success() {
            return Err(AppError::Network(
                "设置静态IP失败，请确认拥有管理员权限".into()
            ));
        }

        // Step 2: Set DNS servers
        if let Some((first, rest)) = dns.split_first() {
            // Primary DNS
            let status = netsh_command()
                .args([
                    "interface", "ip", "set", "dns",
                    &format!("name=\"{}\"", interface),
                    "static", first,
                ])
                .status()?;

            if !status.success() {
                return Err(AppError::Network("设置主DNS服务器失败".into()));
            }

            // Additional DNS servers
            for (i, d) in rest.iter().enumerate() {
                let status = netsh_command()
                    .args([
                        "interface", "ip", "add", "dns",
                        &format!("name=\"{}\"", interface),
                        d,
                        &format!("index={}", i + 2),
                    ])
                    .status()?;

                if !status.success() {
                    // Non-fatal: primary DNS is set
                    log::warn!("设置备用DNS服务器失败: {}", d);
                }
            }
        }

        Ok(())
    }

    fn set_dhcp(&self, interface: &str) -> Result<(), AppError> {
        // Set address to DHCP
        let status = netsh_command()
            .args([
                "interface", "ip", "set", "address",
                &format!("name=\"{}\"", interface),
                "source=dhcp",
            ])
            .status()?;

        if !status.success() {
            return Err(AppError::Network("切换到DHCP失败".into()));
        }

        // Set DNS to DHCP
        let _ = netsh_command()
            .args([
                "interface", "ip", "set", "dns",
                &format!("name=\"{}\"", interface),
                "source=dhcp",
            ])
            .status();

        Ok(())
    }
}

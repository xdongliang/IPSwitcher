use std::process::Command;

use crate::error::AppError;
use crate::platform::{CurrentNetworkConfig, NetworkInterface, NetworkManager};

pub struct MacOSNetworkManager;

impl NetworkManager for MacOSNetworkManager {
    fn list_interfaces(&self) -> Result<Vec<NetworkInterface>, AppError> {
        let output = Command::new("networksetup")
            .arg("-listallnetworkservices")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Network(format!(
                "无法列出网络服务: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let interfaces: Vec<NetworkInterface> = stdout
            .lines()
            .filter(|line| {
                // Skip the informational header line
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with("An asterisk")
                    && !trimmed.starts_with("(*)")
            })
            .map(|line| {
                let name = line.trim().trim_start_matches("(*) ").to_string();
                NetworkInterface {
                    display_name: name.clone(),
                    name: name.clone(),
                    is_active: line.contains("(*)"),
                }
            })
            .collect();

        Ok(interfaces)
    }

    fn get_current_config(&self, interface: &str) -> Result<CurrentNetworkConfig, AppError> {
        let output = Command::new("networksetup")
            .arg("-getinfo")
            .arg(interface)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!(
            "{}\n{}",
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );

        let mut config = CurrentNetworkConfig {
            interface: interface.to_string(),
            ip_address: None,
            subnet_mask: None,
            gateway: None,
            dns_servers: Vec::new(),
            is_dhcp: false,
        };

        let mut in_dns_section = false;

        for line in combined.lines() {
            let trimmed = line.trim();

            if trimmed.contains("DHCP Configuration") || trimmed.contains("Manual Configuration") {
                // e.g. "Manual Configuration" or "DHCP Configuration"
                if let Some(parts) = trimmed.split_once(':') {
                    if parts.1.trim() == "DHCP" {
                        config.is_dhcp = true;
                    }
                }
            }

            if trimmed.starts_with("IP address:") {
                config.ip_address = trimmed
                    .split_once(':')
                    .map(|(_, v)| v.trim().to_string());
            }
            if trimmed.starts_with("Subnet mask:") {
                config.subnet_mask = trimmed
                    .split_once(':')
                    .map(|(_, v)| v.trim().to_string());
            }
            if trimmed.starts_with("Router:") {
                config.gateway = trimmed
                    .split_once(':')
                    .map(|(_, v)| v.trim().to_string());
            }

            if trimmed.starts_with("DNS:") {
                in_dns_section = true;
                continue;
            }

            if in_dns_section && !trimmed.is_empty() && !trimmed.contains(':') {
                config.dns_servers.push(trimmed.to_string());
            } else if in_dns_section && trimmed.contains(':') {
                in_dns_section = false;
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
        // Step 1: Set static IP
        let status = Command::new("networksetup")
            .arg("-setmanual")
            .arg(interface)
            .arg(ip)
            .arg(mask)
            .arg(gateway)
            .status()?;

        if !status.success() {
            return Err(AppError::Network(format!(
                "设置静态IP失败，请确认拥有管理员权限"
            )));
        }

        // Step 2: Set DNS servers
        if !dns.is_empty() {
            let mut cmd = Command::new("networksetup");
            cmd.arg("-setdnsservers").arg(interface);
            for d in dns {
                cmd.arg(d);
            }
            let status = cmd.status()?;
            if !status.success() {
                return Err(AppError::Network(format!(
                    "设置DNS服务器失败"
                )));
            }
        }

        Ok(())
    }

    fn set_dhcp(&self, interface: &str) -> Result<(), AppError> {
        let status = Command::new("networksetup")
            .arg("-setdhcp")
            .arg(interface)
            .status()?;

        if !status.success() {
            return Err(AppError::Network(format!(
                "切换到DHCP失败，请确认拥有管理员权限"
            )));
        }

        // Also clear DNS to use DHCP-provided DNS
        let _ = Command::new("networksetup")
            .arg("-setdnsservers")
            .arg(interface)
            .arg("Empty")
            .status();

        Ok(())
    }
}

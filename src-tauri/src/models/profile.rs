use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IpMode {
    Manual,
    Dhcp,
}

impl std::fmt::Display for IpMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpMode::Manual => write!(f, "手动配置"),
            IpMode::Dhcp => write!(f, "自动获取(DHCP)"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub ip_mode: IpMode,
    pub ip_address: Option<String>,
    pub subnet_mask: Option<String>,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub interface_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Profile {
    pub fn validate(&self) -> Result<(), String> {
        // Name: non-empty, max 64 chars
        if self.name.trim().is_empty() {
            return Err("方案名称不能为空".into());
        }
        if self.name.len() > 64 {
            return Err("方案名称不能超过64个字符".into());
        }

        match self.ip_mode {
            IpMode::Manual => {
                // IP address required
                let ip = self.ip_address.as_ref().ok_or("手动模式下IP地址为必填项")?;
                validate_ipv4(ip)?;

                // Subnet mask required
                let mask = self.subnet_mask.as_ref().ok_or("手动模式下子网掩码为必填项")?;
                validate_ipv4(mask)?;

                // Gateway required
                let gateway = self.gateway.as_ref().ok_or("手动模式下默认网关为必填项")?;
                validate_ipv4(gateway)?;

                // DNS servers validation
                if self.dns_servers.is_empty() {
                    return Err("手动模式下至少需要配置一个DNS服务器".into());
                }
                for dns in &self.dns_servers {
                    validate_ipv4(dns)?;
                }
            }
            IpMode::Dhcp => {
                if self.ip_address.is_some() {
                    return Err("DHCP模式下不应设置IP地址".into());
                }
                if self.subnet_mask.is_some() {
                    return Err("DHCP模式下不应设置子网掩码".into());
                }
                if self.gateway.is_some() {
                    return Err("DHCP模式下不应设置默认网关".into());
                }
            }
        }

        Ok(())
    }
}

fn validate_ipv4(addr: &str) -> Result<(), String> {
    addr.parse::<Ipv4Addr>()
        .map_err(|_| format!("无效的IPv4地址: {}", addr))?;
    Ok(())
}

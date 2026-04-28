import { useState, useEffect, useCallback } from "react";
import type { Profile, IpMode, NetworkInterface } from "../types";
import InterfaceSelector from "./InterfaceSelector";
import DnsEditor from "./DnsEditor";
import SwitchConfirmDialog from "./SwitchConfirmDialog";

const DEFAULT_DNS = ["114.114.114.114", "8.8.8.8"];

interface ProfileFormProps {
  profile: Profile | null; // null means creating new
  profiles: Profile[];
  interfaces: NetworkInterface[];
  onSave: (data: ProfileFormData) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onApply: (profile: Profile, iface: string) => Promise<void>;
  onCancel: () => void;
}

export interface ProfileFormData {
  id?: string;
  name: string;
  ip_mode: IpMode;
  ip_address: string;
  subnet_mask: string;
  gateway: string;
  dns_servers: string[];
  interface_name: string;
}

export default function ProfileForm({
  profile,
  profiles,
  interfaces,
  onSave,
  onDelete,
  onApply,
  onCancel,
}: ProfileFormProps) {
  const isNew = profile === null;

  const [name, setName] = useState("");
  const [ipMode, setIpMode] = useState<IpMode>("manual");
  const [ipAddress, setIpAddress] = useState("");
  const [subnetMask, setSubnetMask] = useState("");
  const [gateway, setGateway] = useState("");
  const [dnsServers, setDnsServers] = useState<string[]>(DEFAULT_DNS);
  const [interfaceName, setInterfaceName] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showConfirm, setShowConfirm] = useState(false);

  // Populate form when profile changes
  useEffect(() => {
    if (profile) {
      setName(profile.name);
      setIpMode(profile.ip_mode);
      setIpAddress(profile.ip_address || "");
      setSubnetMask(profile.subnet_mask || "");
      setGateway(profile.gateway || "");
      setDnsServers(
        profile.dns_servers.length > 0 ? profile.dns_servers : DEFAULT_DNS,
      );
      setInterfaceName(profile.interface_name || "");
    } else {
      setName("");
      setIpMode("manual");
      setIpAddress("");
      setSubnetMask("");
      setGateway("");
      setDnsServers(DEFAULT_DNS);
      setInterfaceName("");
    }
    setError(null);
  }, [profile?.id]);

  const validate = useCallback((): string | null => {
    if (!name.trim()) return "方案名称不能为空";
    if (name.trim().length > 64) return "方案名称不能超过64个字符";

    // Check name uniqueness
    const existingName = profiles.find(
      (p) => p.name === name.trim() && p.id !== profile?.id,
    );
    if (existingName) return "方案名称已存在";

    if (!interfaceName) return "请选择网络接口";

    if (ipMode === "manual") {
      if (!ipAddress.trim()) return "请输入IP地址";
      if (!isValidIPv4(ipAddress.trim())) return "IP地址格式无效";
      if (!subnetMask.trim()) return "请输入子网掩码";
      if (!isValidIPv4(subnetMask.trim())) return "子网掩码格式无效";
      if (!gateway.trim()) return "请输入默认网关";
      if (!isValidIPv4(gateway.trim())) return "网关地址格式无效";
      if (dnsServers.length === 0) return "至少需要配置一个DNS服务器";
      for (const dns of dnsServers) {
        if (!isValidIPv4(dns.trim())) return `DNS地址格式无效: ${dns}`;
      }
    }

    return null;
  }, [name, ipMode, ipAddress, subnetMask, gateway, dnsServers, interfaceName, profiles, profile]);

  const handleSave = async () => {
    const validationError = validate();
    if (validationError) {
      setError(validationError);
      return;
    }

    setError(null);
    setSaving(true);
    try {
      await onSave({
        id: profile?.id,
        name: name.trim(),
        ip_mode: ipMode,
        ip_address: ipAddress.trim(),
        subnet_mask: subnetMask.trim(),
        gateway: gateway.trim(),
        dns_servers: dnsServers.map((d) => d.trim()).filter((d) => d),
        interface_name: interfaceName,
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleApply = () => {
    const validationError = validate();
    if (validationError) {
      setError(validationError);
      return;
    }
    setError(null);
    setShowConfirm(true);
  };

  const handleConfirmApply = async () => {
    setShowConfirm(false);
    if (!profile) return;
    try {
      const validationError = validate();
      if (validationError) {
        setError(validationError);
        return;
      }

      const data = {
        id: profile.id,
        name: name.trim(),
        ip_mode: ipMode,
        ip_address: ipAddress.trim(),
        subnet_mask: subnetMask.trim(),
        gateway: gateway.trim(),
        dns_servers: dnsServers.map((d) => d.trim()).filter((d) => d),
        interface_name: interfaceName,
      };

      // Save first
      await onSave(data);

      // Then apply network config
      await onApply(
        {
          ...profile,
          name: data.name,
          ip_mode: data.ip_mode,
          ip_address: data.ip_address || null,
          subnet_mask: data.subnet_mask || null,
          gateway: data.gateway || null,
          dns_servers: data.dns_servers,
          interface_name: data.interface_name || null,
        },
        data.interface_name,
      );
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async () => {
    if (!profile) return;
    setSaving(true);
    try {
      await onDelete(profile.id);
      onCancel();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const isManual = ipMode === "manual";

  return (
    <div className="profile-form">
      <h2 className="panel-title">
        {isNew ? "新建配置方案" : "编辑配置方案"}
      </h2>

      {error && <div className="alert alert-error">{error}</div>}

      <div className="form-body">
        <div className="form-group">
          <label className="form-label">方案名称</label>
          <input
            type="text"
            className="form-input"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="例如: 家庭网络、公司网络"
            maxLength={64}
          />
        </div>

        <InterfaceSelector
          interfaces={interfaces}
          value={interfaceName}
          onChange={setInterfaceName}
        />

        <div className="form-group">
          <label className="form-label">IP 获取方式</label>
          <div className="radio-group">
            <label className="radio-label">
              <input
                type="radio"
                name="ipMode"
                checked={isManual}
                onChange={() => setIpMode("manual")}
              />
              手动配置
            </label>
            <label className="radio-label">
              <input
                type="radio"
                name="ipMode"
                checked={!isManual}
                onChange={() => setIpMode("dhcp")}
              />
              自动获取(DHCP)
            </label>
          </div>
        </div>

        {isManual && (
          <div className="manual-fields">
            <h3 className="section-title">手动配置</h3>

            <div className="form-group">
              <label className="form-label">IP 地址</label>
              <input
                type="text"
                className="form-input"
                value={ipAddress}
                onChange={(e) => setIpAddress(e.target.value)}
                placeholder="192.168.1.100"
              />
            </div>

            <div className="form-group">
              <label className="form-label">子网掩码</label>
              <input
                type="text"
                className="form-input"
                value={subnetMask}
                onChange={(e) => setSubnetMask(e.target.value)}
                placeholder="255.255.255.0"
              />
            </div>

            <div className="form-group">
              <label className="form-label">默认网关(路由器)</label>
              <input
                type="text"
                className="form-input"
                value={gateway}
                onChange={(e) => setGateway(e.target.value)}
                placeholder="192.168.1.1"
              />
            </div>

            <DnsEditor servers={dnsServers} onChange={setDnsServers} />
          </div>
        )}
      </div>

      <div className="form-actions">
        {!isNew && (
          <button
            className="btn btn-danger"
            onClick={handleDelete}
            disabled={saving}
          >
            删除方案
          </button>
        )}
        <div className="form-actions-right">
          <button className="btn" onClick={onCancel}>
            {isNew ? "取消" : "取消编辑"}
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? "保存中..." : "保存"}
          </button>
          {!isNew && (
            <button
              className="btn btn-apply"
              onClick={handleApply}
              disabled={saving || !interfaceName}
            >
              立即应用
            </button>
          )}
        </div>
      </div>

      {showConfirm && profile && (
        <SwitchConfirmDialog
          profile={{
            ...profile,
            name: name.trim(),
            ip_mode: ipMode,
            ip_address: ipAddress.trim() || null,
            subnet_mask: subnetMask.trim() || null,
            gateway: gateway.trim() || null,
            dns_servers: dnsServers.map((d) => d.trim()).filter((d) => d),
            interface_name: interfaceName || null,
          }}
          targetInterface={interfaceName}
          interfaces={interfaces}
          onConfirm={handleConfirmApply}
          onCancel={() => setShowConfirm(false)}
        />
      )}
    </div>
  );
}

function isValidIPv4(addr: string): boolean {
  const parts = addr.split(".");
  if (parts.length !== 4) return false;
  for (const part of parts) {
    const num = parseInt(part, 10);
    if (isNaN(num) || num < 0 || num > 255) return false;
    if (part !== num.toString()) return false;
  }
  return true;
}

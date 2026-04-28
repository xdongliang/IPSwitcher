import type { CurrentNetworkConfig } from "../types";

interface StatusBarProps {
  config: CurrentNetworkConfig | null;
  loading: boolean;
  activeProfileName: string | null;
}

export default function StatusBar({ config, loading, activeProfileName }: StatusBarProps) {
  if (loading) {
    return (
      <div className="status-bar">
        <span className="status-text">正在获取网络状态...</span>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="status-bar">
        <span className="status-text status-unknown">无法获取网络状态</span>
        <span className="status-dot dot-unknown" />
      </div>
    );
  }

  const modeText = config.is_dhcp ? "DHCP" : "静态";
  const ipText = config.ip_address || "无IP";

  return (
    <div className="status-bar">
      <span className="status-text">
        {activeProfileName && (
          <span className="active-profile-label">方案: {activeProfileName} | </span>
        )}
        当前: {config.interface} → {ipText} ({modeText})
        {config.gateway && ` | 网关: ${config.gateway}`}
      </span>
      <span className={`status-dot ${config.ip_address ? "dot-connected" : "dot-disconnected"}`} />
    </div>
  );
}

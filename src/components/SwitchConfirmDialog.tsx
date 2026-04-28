import type { Profile, NetworkInterface } from "../types";

interface SwitchConfirmDialogProps {
  profile: Profile;
  targetInterface: string;
  interfaces: NetworkInterface[];
  onConfirm: () => void;
  onCancel: () => void;
}

export default function SwitchConfirmDialog({
  profile,
  targetInterface,
  interfaces,
  onConfirm,
  onCancel,
}: SwitchConfirmDialogProps) {
  const ifaceName =
    interfaces.find((i) => i.name === targetInterface)?.display_name ||
    targetInterface;

  return (
    <div className="dialog-overlay" onClick={onCancel}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <h3 className="dialog-title">确认切换网络配置</h3>

        <div className="dialog-body">
          <div className="confirm-row">
            <span className="confirm-label">方案:</span>
            <span className="confirm-value">{profile.name}</span>
          </div>
          <div className="confirm-row">
            <span className="confirm-label">接口:</span>
            <span className="confirm-value">{ifaceName}</span>
          </div>
          <div className="confirm-row">
            <span className="confirm-label">模式:</span>
            <span className="confirm-value">
              {profile.ip_mode === "manual" ? "手动配置" : "自动获取(DHCP)"}
            </span>
          </div>

          {profile.ip_mode === "manual" && (
            <>
              <div className="confirm-row">
                <span className="confirm-label">IP 地址:</span>
                <span className="confirm-value">
                  {profile.ip_address}
                </span>
              </div>
              <div className="confirm-row">
                <span className="confirm-label">子网掩码:</span>
                <span className="confirm-value">
                  {profile.subnet_mask}
                </span>
              </div>
              <div className="confirm-row">
                <span className="confirm-label">网关:</span>
                <span className="confirm-value">
                  {profile.gateway}
                </span>
              </div>
              <div className="confirm-row">
                <span className="confirm-label">DNS:</span>
                <span className="confirm-value">
                  {profile.dns_servers.join(", ")}
                </span>
              </div>
            </>
          )}

          <p className="confirm-warning">
            此操作将修改计算机网络配置，可能需要管理员权限。
          </p>
        </div>

        <div className="dialog-actions">
          <button className="btn" onClick={onCancel}>
            取消
          </button>
          <button className="btn btn-primary" onClick={onConfirm}>
            确认切换
          </button>
        </div>
      </div>
    </div>
  );
}

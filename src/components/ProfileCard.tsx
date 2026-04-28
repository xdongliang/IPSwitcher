import type { Profile } from "../types";

interface ProfileCardProps {
  profile: Profile;
  isSelected: boolean;
  isActive: boolean;
  onSelect: () => void;
  onSwitch: () => void;
}

export default function ProfileCard({
  profile,
  isSelected,
  isActive,
  onSelect,
  onSwitch,
}: ProfileCardProps) {
  const modeLabel = profile.ip_mode === "manual" ? "手动" : "DHCP";
  const modeClass = profile.ip_mode === "manual" ? "mode-manual" : "mode-dhcp";
  const summary =
    profile.ip_mode === "manual"
      ? profile.ip_address || "未设置"
      : "自动获取";

  return (
    <div
      className={`profile-card ${isSelected ? "selected" : ""} ${isActive ? "active" : ""}`}
      onClick={onSelect}
    >
      <div className="profile-card-header">
        <span className="profile-name">
          {isActive && <span className="active-badge">当前</span>}
          {profile.name}
        </span>
        <span className={`profile-mode ${modeClass}`}>{modeLabel}</span>
      </div>
      <div className="profile-card-info">
        <span className="profile-summary">{summary}</span>
        {profile.interface_name && (
          <span className="profile-interface">
            {profile.interface_name}
          </span>
        )}
      </div>
      <button
        className="btn btn-switch"
        onClick={(e) => {
          e.stopPropagation();
          onSwitch();
        }}
      >
        切换
      </button>
    </div>
  );
}

import type { Profile } from "../types";
import ProfileCard from "./ProfileCard";

interface ProfileListProps {
  profiles: Profile[];
  selectedId: string | null;
  activeProfileId: string | null;
  loading: boolean;
  onSelect: (id: string) => void;
  onSwitch: (profile: Profile) => void;
  onNew: () => void;
}

export default function ProfileList({
  profiles,
  selectedId,
  activeProfileId,
  loading,
  onSelect,
  onSwitch,
  onNew,
}: ProfileListProps) {
  return (
    <div className="profile-list">
      <div className="profile-list-header">
        <h2 className="panel-title">配置方案</h2>
        <button className="btn btn-primary btn-new" onClick={onNew}>
          + 新建
        </button>
      </div>
      <div className="profile-list-content">
        {loading && profiles.length === 0 ? (
          <div className="loading-state">加载中...</div>
        ) : profiles.length === 0 ? (
          <div className="empty-state">
            <p>暂无配置方案</p>
            <p className="empty-hint">点击"+ 新建"创建第一个方案</p>
          </div>
        ) : (
          profiles.map((profile) => (
            <ProfileCard
              key={profile.id}
              profile={profile}
              isSelected={profile.id === selectedId}
              isActive={profile.id === activeProfileId}
              onSelect={() => onSelect(profile.id)}
              onSwitch={() => onSwitch(profile)}
            />
          ))
        )}
      </div>
    </div>
  );
}

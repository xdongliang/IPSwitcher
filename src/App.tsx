import { useState, useCallback, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import ProfileList from "./components/ProfileList";
import ProfileForm, { ProfileFormData } from "./components/ProfileForm";
import StatusBar from "./components/StatusBar";
import UpdateChecker from "./components/UpdateChecker";
import { useProfiles } from "./hooks/useProfiles";
import { useNetwork } from "./hooks/useNetwork";
import type { Profile } from "./types";

export default function App() {
  const {
    profiles,
    activeProfileId,
    loading: profilesLoading,
    error: profileError,
    createProfile,
    updateProfile,
    deleteProfile,
    fetchProfiles,
    fetchActiveProfileId,
  } = useProfiles();

  const {
    interfaces,
    currentConfig,
    loading: networkLoading,
    error: networkError,
    applyProfile,
  } = useNetwork();

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [appVersion, setAppVersion] = useState<string>("");

  useEffect(() => {
    getVersion().then((v) => {
      setAppVersion(v);
      getCurrentWindow().setTitle(`IPSwitcher v${v}`);
    });
  }, []);

  const handleSelectProfile = useCallback((id: string) => {
    setSelectedId(id);
    setIsCreating(false);
  }, []);

  const handleNewProfile = useCallback(() => {
    setSelectedId(null);
    setIsCreating(true);
  }, []);

  const handleCancel = useCallback(() => {
    setIsCreating(false);
    setSelectedId(null);
  }, []);

  const handleSave = useCallback(
    async (data: ProfileFormData) => {
      if (data.id) {
        await updateProfile({
          id: data.id,
          name: data.name,
          ip_mode: data.ip_mode,
          ip_address: data.ip_address || undefined,
          subnet_mask: data.subnet_mask || undefined,
          gateway: data.gateway || undefined,
          dns_servers: data.dns_servers,
          interface_name: data.interface_name || undefined,
        });
      } else {
        const created = await createProfile({
          name: data.name,
          ip_mode: data.ip_mode,
          ip_address: data.ip_address || undefined,
          subnet_mask: data.subnet_mask || undefined,
          gateway: data.gateway || undefined,
          dns_servers: data.dns_servers,
          interface_name: data.interface_name || undefined,
        });
        setSelectedId(created.id);
        setIsCreating(false);
      }
    },
    [createProfile, updateProfile],
  );

  const handleDelete = useCallback(
    async (id: string) => {
      await deleteProfile(id);
      setSelectedId(null);
      setIsCreating(false);
    },
    [deleteProfile],
  );

  const handleSwitch = useCallback(
    async (profile: Profile) => {
      const iface = profile.interface_name || undefined;
      if (!iface) {
        setSelectedId(profile.id);
        return;
      }
      try {
        await applyProfile(profile.id, iface);
        fetchProfiles();
        fetchActiveProfileId();
      } catch (e) {
        console.error("Switch failed:", e);
      }
    },
    [applyProfile, fetchProfiles, fetchActiveProfileId],
  );

  const handleConfirmApply = useCallback(
    async (profile: Profile, iface: string) => {
      try {
        await applyProfile(profile.id, iface);
        fetchProfiles();
        fetchActiveProfileId();
      } catch (e) {
        console.error("Apply failed:", e);
      }
    },
    [applyProfile, fetchProfiles, fetchActiveProfileId],
  );

  // Listen for tray events
  useEffect(() => {
    const setup = async () => {
      const unlistenNew = await listen("tray-new-profile", () => {
        setSelectedId(null);
        setIsCreating(true);
      });

      const unlistenSwitch = await listen<string>("tray-switch-profile", (event) => {
        const profileId = event.payload;
        if (profileId) {
          const profile = profiles.find((p) => p.id === profileId);
          if (profile) {
            handleSwitch(profile);
          }
        }
      });

      return () => {
        unlistenNew();
        unlistenSwitch();
      };
    };

    const cleanup = setup();
    return () => {
      cleanup.then((fn) => fn());
    };
  }, [profiles, handleSwitch]);

  const selectedProfile = selectedId
    ? profiles.find((p) => p.id === selectedId) || null
    : null;

  const showForm = isCreating || selectedProfile !== null;

  const activeProfile = activeProfileId
    ? profiles.find((p) => p.id === activeProfileId) || null
    : null;

  return (
    <div className="app-container">
      <UpdateChecker />
      <div className="app-main">
        <aside className="sidebar">
          <ProfileList
            profiles={profiles}
            selectedId={selectedId}
            activeProfileId={activeProfileId}
            loading={profilesLoading}
            onSelect={handleSelectProfile}
            onSwitch={handleSwitch}
            onNew={handleNewProfile}
          />
        </aside>
        <main className="main-content">
          {showForm ? (
            <ProfileForm
              profile={selectedProfile}
              profiles={profiles}
              interfaces={interfaces}
              onSave={handleSave}
              onDelete={handleDelete}
              onApply={handleConfirmApply}
              onCancel={handleCancel}
            />
          ) : (
            <div className="welcome-placeholder">
              <h2>IPSwitcher</h2>
              <p>网络配置切换工具</p>
              {appVersion && <p className="welcome-version">v{appVersion}</p>}
              <p className="welcome-hint">
                从左侧选择一个配置方案，或点击"+ 新建"创建新方案
              </p>
              {(profileError || networkError) && (
                <div className="alert alert-error">
                  {profileError || networkError}
                </div>
              )}
            </div>
          )}
        </main>
      </div>
      <StatusBar
        config={currentConfig}
        loading={networkLoading}
        activeProfileName={activeProfile?.name || null}
        version={appVersion}
      />
    </div>
  );
}

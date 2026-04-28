import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Profile, IpMode } from "../types";

export function useProfiles() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [activeProfileId, setActiveProfileId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchActiveProfileId = useCallback(async () => {
    try {
      const result = await invoke<string | null>("get_active_profile_id");
      setActiveProfileId(result);
    } catch (e) {
      console.error("Failed to fetch active profile id:", e);
    }
  }, []);

  const fetchProfiles = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<Profile[]>("list_profiles");
      setProfiles(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const createProfile = useCallback(
    async (data: {
      name: string;
      ip_mode: IpMode;
      ip_address?: string;
      subnet_mask?: string;
      gateway?: string;
      dns_servers: string[];
      interface_name?: string;
    }) => {
      setError(null);
      try {
        const result = await invoke<Profile>("create_profile", {
          name: data.name,
          ipMode: data.ip_mode === "manual" ? "Manual" : "Dhcp",
          ipAddress: data.ip_address || null,
          subnetMask: data.subnet_mask || null,
          gateway: data.gateway || null,
          dnsServers: data.dns_servers,
          interfaceName: data.interface_name || null,
        });
        setProfiles((prev) => [result, ...prev]);
        return result;
      } catch (e) {
        const msg = String(e);
        setError(msg);
        throw new Error(msg);
      }
    },
    [],
  );

  const updateProfile = useCallback(
    async (data: {
      id: string;
      name: string;
      ip_mode: IpMode;
      ip_address?: string;
      subnet_mask?: string;
      gateway?: string;
      dns_servers: string[];
      interface_name?: string;
    }) => {
      setError(null);
      try {
        const result = await invoke<Profile>("update_profile", {
          id: data.id,
          name: data.name,
          ipMode: data.ip_mode === "manual" ? "Manual" : "Dhcp",
          ipAddress: data.ip_address || null,
          subnetMask: data.subnet_mask || null,
          gateway: data.gateway || null,
          dnsServers: data.dns_servers,
          interfaceName: data.interface_name || null,
        });
        setProfiles((prev) =>
          prev.map((p) => (p.id === result.id ? result : p)),
        );
        return result;
      } catch (e) {
        const msg = String(e);
        setError(msg);
        throw new Error(msg);
      }
    },
    [],
  );

  const deleteProfile = useCallback(async (id: string) => {
    setError(null);
    try {
      await invoke("delete_profile", { id });
      setProfiles((prev) => prev.filter((p) => p.id !== id));
      setActiveProfileId((prev) => (prev === id ? null : prev));
    } catch (e) {
      const msg = String(e);
      setError(msg);
      throw new Error(msg);
    }
  }, []);

  useEffect(() => {
    fetchProfiles();
    fetchActiveProfileId();
  }, [fetchProfiles, fetchActiveProfileId]);

  return {
    profiles,
    activeProfileId,
    loading,
    error,
    fetchProfiles,
    fetchActiveProfileId,
    createProfile,
    updateProfile,
    deleteProfile,
  };
}

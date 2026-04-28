import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { NetworkInterface, CurrentNetworkConfig } from "../types";

export function useNetwork() {
  const [interfaces, setInterfaces] = useState<NetworkInterface[]>([]);
  const [currentConfig, setCurrentConfig] =
    useState<CurrentNetworkConfig | null>(null);
  const [isAdmin, setIsAdmin] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchInterfaces = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<NetworkInterface[]>(
        "list_network_interfaces",
      );
      setInterfaces(result);
    } catch (e) {
      // Ignore errors fetching interfaces
      console.error("Failed to fetch interfaces:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchCurrentConfig = useCallback(async (iface?: string) => {
    try {
      const result = await invoke<CurrentNetworkConfig>(
        "get_current_network_config",
        { interface: iface || null },
      );
      setCurrentConfig(result);
    } catch (e) {
      console.error("Failed to fetch current config:", e);
    }
  }, []);

  const checkAdmin = useCallback(async () => {
    try {
      const result = await invoke<boolean>("check_admin_status");
      setIsAdmin(result);
    } catch (e) {
      console.error("Failed to check admin:", e);
    }
  }, []);

  const applyProfile = useCallback(
    async (profileId: string, iface?: string) => {
      setError(null);
      try {
        const result = await invoke<string>("apply_profile", {
          profileId,
          interface: iface || null,
        });
        // Refresh current config after applying
        if (iface) {
          await fetchCurrentConfig(iface);
        }
        return result;
      } catch (e) {
        const msg = String(e);
        setError(msg);
        throw new Error(msg);
      }
    },
    [fetchCurrentConfig],
  );

  useEffect(() => {
    fetchInterfaces();
    checkAdmin();
  }, [fetchInterfaces, checkAdmin]);

  useEffect(() => {
    // Auto-fetch current config every 30 seconds
    const activeIface = interfaces.find((i) => i.is_active) || interfaces[0];
    if (activeIface) {
      fetchCurrentConfig(activeIface.name);
      const interval = setInterval(() => {
        fetchCurrentConfig(activeIface.name);
      }, 30000);
      return () => clearInterval(interval);
    }
  }, [interfaces, fetchCurrentConfig]);

  return {
    interfaces,
    currentConfig,
    isAdmin,
    loading,
    error,
    fetchInterfaces,
    fetchCurrentConfig,
    applyProfile,
  };
}

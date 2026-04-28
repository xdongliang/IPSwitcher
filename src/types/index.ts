export interface Profile {
  id: string;
  name: string;
  ip_mode: "manual" | "dhcp";
  ip_address: string | null;
  subnet_mask: string | null;
  gateway: string | null;
  dns_servers: string[];
  interface_name: string | null;
  created_at: string;
  updated_at: string;
}

export interface NetworkInterface {
  name: string;
  display_name: string;
  is_active: boolean;
}

export interface CurrentNetworkConfig {
  interface: string;
  ip_address: string | null;
  subnet_mask: string | null;
  gateway: string | null;
  dns_servers: string[];
  is_dhcp: boolean;
}

export type IpMode = "manual" | "dhcp";

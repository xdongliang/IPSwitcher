import type { NetworkInterface } from "../types";

interface InterfaceSelectorProps {
  interfaces: NetworkInterface[];
  value: string;
  onChange: (value: string) => void;
}

export default function InterfaceSelector({
  interfaces,
  value,
  onChange,
}: InterfaceSelectorProps) {
  return (
    <div className="form-group">
      <label className="form-label">网络接口</label>
      <select
        className="form-select"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        <option value="">请选择网络接口...</option>
        {interfaces.map((iface) => (
          <option key={iface.name} value={iface.name}>
            {iface.display_name}
            {iface.is_active ? " (活跃)" : ""}
            {iface.name !== iface.display_name ? ` [${iface.name}]` : ""}
          </option>
        ))}
      </select>
    </div>
  );
}

import { useState } from "react";

interface DnsEditorProps {
  servers: string[];
  onChange: (servers: string[]) => void;
}

export default function DnsEditor({ servers, onChange }: DnsEditorProps) {
  const [newDns, setNewDns] = useState("");

  const addServer = () => {
    const trimmed = newDns.trim();
    if (trimmed && !servers.includes(trimmed)) {
      onChange([...servers, trimmed]);
      setNewDns("");
    }
  };

  const removeServer = (index: number) => {
    onChange(servers.filter((_, i) => i !== index));
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      addServer();
    }
  };

  return (
    <div className="form-group">
      <label className="form-label">DNS 服务器</label>
      <div className="dns-list">
        {servers.map((server, index) => (
          <div key={index} className="dns-row">
            <input
              type="text"
              className="form-input dns-input"
              value={server}
              onChange={(e) => {
                const newServers = [...servers];
                newServers[index] = e.target.value;
                onChange(newServers);
              }}
              placeholder="例如: 114.114.114.114"
            />
            <button
              type="button"
              className="btn btn-icon btn-danger"
              onClick={() => removeServer(index)}
              title="删除此DNS"
            >
              ✕
            </button>
          </div>
        ))}
      </div>
      <div className="dns-add-row">
        <input
          type="text"
          className="form-input dns-input"
          value={newDns}
          onChange={(e) => setNewDns(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="输入DNS地址后回车添加"
        />
        <button
          type="button"
          className="btn btn-sm"
          onClick={addServer}
          disabled={!newDns.trim()}
        >
          添加
        </button>
      </div>
    </div>
  );
}

# IPSwitcher - AI Agent 项目指南

## 项目概述

IPSwitcher 是一个跨平台 IP 配置切换桌面工具，允许用户创建和快速切换网络配置方案（支持手动 IP 和 DHCP 两种模式）。采用 Tauri 2.0 框架，前端 React + TypeScript，后端 Rust。

- **应用标识**：com.ipswitcher.desktop
- **支持平台**：macOS、Windows

## 技术栈

### 前端
- React 18.3 + TypeScript 5.5
- Vite 5.4（构建工具）
- @tauri-apps/api 2（Tauri IPC 桥接）

### 后端
- Rust (edition 2021, MSRV 1.77)
- Tauri 2.0
- SQLite（rusqlite 0.31, bundled 模式）
- serde / serde_json（序列化）
- uuid（ID 生成）、chrono（时间）、regex（正则）
- thiserror（错误处理）、log + env_logger（日志）
- tauri-plugin-shell（Shell 命令执行）

### 平台特定依赖
- macOS：libc 0.2, objc2 0.6
- Windows：系统命令 netsh

## 开发和构建命令

```bash
# 开发模式（前端热更新 + Rust 实时编译）
npm run tauri dev

# 生产构建（前端构建 + Rust release 编译 + 打包）
npm run tauri build

# 仅前端开发服务器
npm run dev

# 仅前端构建（TypeScript 检查 + Vite 打包）
npm run build

# 预览前端构建结果
npm run preview
```

## 项目架构

### 目录结构

```
src/                          # 前端源码
├── components/               # React 组件
│   ├── ProfileList.tsx       # 配置方案列表（侧边栏）
│   ├── ProfileForm.tsx       # 方案编辑表单（IP/DHCP、DNS、接口）
│   ├── ProfileCard.tsx       # 方案卡片展示
│   ├── InterfaceSelector.tsx # 网络接口选择器
│   ├── DnsEditor.tsx         # DNS 服务器编辑器（多条）
│   ├── SwitchConfirmDialog.tsx # 应用确认对话框
│   └── StatusBar.tsx         # 底部状态栏（当前网络配置）
├── hooks/
│   ├── useProfiles.ts        # 方案 CRUD + 活跃方案管理
│   └── useNetwork.ts         # 网络接口/配置/应用/权限检查
├── types/
│   └── index.ts              # TypeScript 类型定义
├── App.tsx                   # 主应用（侧边栏 + 主区域 + 状态栏）
└── main.tsx                  # Vite 入口

src-tauri/src/                # 后端源码
├── commands/                 # Tauri IPC 命令
│   ├── profiles.rs           # 方案 CRUD（5 个命令）
│   ├── network.rs            # 网络操作（6 个命令）
│   └── interfaces.rs         # 接口枚举（1 个命令）
├── models/
│   └── profile.rs            # Profile 数据模型 + 验证规则
├── platform/                 # 跨平台网络管理
│   ├── mod.rs                # NetworkManager trait 定义
│   ├── macos.rs              # macOS 实现（networksetup）
│   └── windows.rs            # Windows 实现（netsh）
├── storage/
│   └── sqlite.rs             # SQLite 存储（ProfileRepository）
├── lib.rs                    # 应用初始化、插件注册、命令注册
├── main.rs                   # 二进制入口
├── admin.rs                  # 权限检查与提升
├── error.rs                  # AppError 错误枚举
└── tray.rs                   # 系统托盘菜单和事件
```

### 前后端 IPC 命令清单

#### 方案管理（commands/profiles.rs）

| 命令 | 功能 | 关键参数 |
|------|------|---------|
| `list_profiles` | 获取所有方案 | 无 |
| `get_profile` | 获取单个方案 | id |
| `create_profile` | 创建方案 | name, ip_mode, ip_address, subnet_mask, gateway, dns_servers, interface_name |
| `update_profile` | 更新方案 | 同上 + id |
| `delete_profile` | 删除方案 | id |

#### 网络操作（commands/network.rs）

| 命令 | 功能 | 关键参数 |
|------|------|---------|
| `get_current_network_config` | 获取当前网络配置 | interface? |
| `apply_profile` | 应用配置方案 | profile_id, interface? |
| `get_active_profile_id` | 获取活跃方案 ID | 无 |
| `set_active_profile_id` | 设置活跃方案 ID | profile_id? |
| `check_admin_status` | 检查管理员权限 | 无 |

#### 接口管理（commands/interfaces.rs）

| 命令 | 功能 |
|------|------|
| `list_network_interfaces` | 获取可用网络接口列表 |

### NetworkManager Trait（平台抽象）

```rust
pub trait NetworkManager {
    fn list_interfaces(&self) -> Result<Vec<NetworkInterface>, AppError>;
    fn get_current_config(&self, interface: &str) -> Result<CurrentNetworkConfig, AppError>;
    fn apply_static_config(&self, interface: &str, ip: &str, mask: &str, gateway: &str, dns: &[String]) -> Result<(), AppError>;
    fn set_dhcp(&self, interface: &str) -> Result<(), AppError>;
}
```

- macOS 通过 `networksetup` 命令实现
- Windows 通过 `netsh` 命令实现

## 核心数据模型

### Profile（配置方案）

```typescript
interface Profile {
  id: string;
  name: string;                    // 唯一，≤64 字符
  ip_mode: "manual" | "dhcp";
  ip_address: string | null;       // manual 模式必填
  subnet_mask: string | null;      // manual 模式必填
  gateway: string | null;          // manual 模式必填
  dns_servers: string[];           // manual 模式至少 1 个
  interface_name: string | null;
  created_at: string;
  updated_at: string;
}
```

### 验证规则
- Manual 模式：ip_address、subnet_mask、gateway、dns_servers（≥1）均为必填
- DHCP 模式：上述字段必须为空
- 所有 IP 地址需通过 IPv4 格式验证

### IpMode 序列化注意
- 前端使用小写：`"manual"` / `"dhcp"`
- 后端使用首字母大写：`Manual` / `Dhcp`
- 前端 `useProfiles.ts` 中手动进行大小写转换

## 数据库

### 存储位置
- macOS：`~/Library/Application Support/com.ipswitcher.app/profiles.db`
- Windows：`%APPDATA%\IPSwitcher\profiles.db`

### 表结构

```sql
CREATE TABLE profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    ip_mode TEXT NOT NULL CHECK (ip_mode IN ('Manual', 'Dhcp')),
    ip_address TEXT,
    subnet_mask TEXT,
    gateway TEXT,
    dns_servers TEXT NOT NULL DEFAULT '[]',  -- JSON 数组
    interface_name TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

## 应用行为

### 启动流程
1. 初始化日志（env_logger，受 RUST_LOG 环境变量控制）
2. 创建/打开 SQLite 数据库
3. 注册 Tauri 插件和命令
4. 设置系统托盘（Accessory 模式，不显示 Dock 图标）
5. 自动应用上次活跃的配置方案
6. 窗口关闭时隐藏而非退出（托盘应用风格）

### 系统托盘
- 动态菜单：列出所有方案，活跃方案带 ✓ 标记
- 菜单项：方案列表、新建方案、显示窗口、退出
- 事件：`tray-switch-profile`、`tray-new-profile`
- 双击托盘图标显示主窗口

### 权限管理
- macOS：通过 `libc::geteuid()` 检查，用 `osascript` (AppleScript) 请求权限提升
- Windows：通过 `net session` 检查，用 PowerShell 提升执行

## 错误处理

```rust
pub enum AppError {
    Database(rusqlite::Error),
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Validation(String),
    NotFound(String),
    DuplicateName(String),
    Network(String),
    PermissionDenied(String),
}
```

后端 AppError 自动序列化为字符串返回前端，前端在 invoke 调用中捕获异常并显示友好错误信息。

## 状态管理

- **前端**：React useState + Tauri IPC 调用（无外部状态库）
- **后端**：Tauri `State<ProfileRepository>`（单例数据库连接）
- **持久化**：SQLite 自动持久化

## macOS 特殊配置

- `Info.plist` 中 `LSUIElement = true`：无 Dock 图标的辅助应用
- 运行时设置 `ActivationPolicy::Accessory`
- 通过 objc2 调用 macOS 系统框架进行窗口激活

## Vite 开发配置

- 开发服务器端口：1420
- HMR WebSocket：ws://localhost:1421
- 前端分发目录：`dist/`（相对项目根目录）

## TypeScript 配置

- 编译目标：ES2020
- 严格模式：启用
- 启用规则：noUnusedLocals、noUnusedParameters、noFallthroughCasesInSwitch、forceConsistentCasingInFileNames

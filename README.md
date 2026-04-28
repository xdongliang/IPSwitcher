# 🌐 IPSwitcher

> 跨平台 IP 配置切换桌面工具 — 一键切换，网络自由

IPSwitcher 是一款基于 Tauri 2.0 构建的轻量级桌面应用，允许用户创建多套网络配置方案并快速切换，支持手动 IP 和 DHCP 两种模式。适用于需要频繁在不同网络环境间切换的开发者、网络工程师和 IT 运维人员。

## ✨ 功能特性

- 📋 **方案管理** — 创建、编辑、删除多套网络配置方案，支持命名管理
- ⚡ **一键切换** — 从系统托盘或主界面快速应用任意配置方案
- 🔄 **双模式支持** — 支持手动 IP（静态配置）和 DHCP（自动获取）两种模式
- 🖥️ **系统托盘** — 常驻系统托盘，动态菜单快速切换方案，活跃方案带 ✓ 标记
- 🚀 **启动自动应用** — 启动时自动恢复上次使用的配置方案
- 🌍 **跨平台** — 同时支持 macOS 和 Windows
- 🔒 **权限管理** — 智能检测并请求管理员权限，确保网络配置安全修改
- 🎯 **多接口支持** — 自动枚举可用网络接口，支持指定接口配置
- 🗄️ **本地存储** — 使用 SQLite 持久化存储，数据安全可靠

## 🛠️ 技术栈

| 层级 | 技术 |
|------|------|
| **前端** | React 18 + TypeScript 5 + Vite 5 |
| **后端** | Rust (Edition 2021) + Tauri 2.0 |
| **数据存储** | SQLite (rusqlite, bundled 模式) |
| **IPC 通信** | Tauri Commands (前后端桥接) |
| **平台集成** | macOS: `networksetup` / Windows: `netsh` |

## 📦 环境要求

在开始开发之前，请确保安装以下工具：

| 工具 | 最低版本 | 说明 |
|------|---------|------|
| **Node.js** | 18+ | 前端开发和构建 |
| **npm** | 9+ | 包管理器 |
| **Rust** | 1.77+ | 后端编译 |
| **Xcode Command Line Tools** | — | macOS 平台必需 |
| **Visual Studio Build Tools** | 2022+ | Windows 平台必需 |

> 💡 推荐使用 [rustup](https://rustup.rs/) 安装和管理 Rust 工具链。

## 🚀 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/xdongliang/IPSwitcher.git
cd IPSwitcher
```

### 2. 安装依赖

```bash
# 安装前端依赖
npm install
```

> Rust 依赖会在首次构建时由 Cargo 自动下载。

### 3. 开发模式运行

```bash
# 启动 Tauri 开发模式（前端热更新 + Rust 实时编译）
npm run tauri dev
```

开发服务器将在 `http://localhost:1420` 启动，支持 HMR 热更新。

## 🏗️ 构建

### 生产构建

```bash
# 前端构建 + Rust release 编译 + 平台打包
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录下：

| 平台 | 产物格式 |
|------|---------|
| macOS | `.app` / `.dmg` |
| Windows | `.exe` / `.msi` |

### 版本号配置

发布新版本时，需要同步修改以下 3 个文件中的版本号：

```bash
# 1. 主版本号来源（前端 getVersion() 读取此值）
src-tauri/tauri.conf.json  →  "version": "1.0.0"

# 2. Rust 包版本
src-tauri/Cargo.toml       →  version = "1.0.0"

# 3. npm 包版本
package.json               →  "version": "1.0.0"
```

版本号会展示在窗口标题栏、欢迎页和底部状态栏。

### 其他命令

```bash
# 仅前端开发服务器
npm run dev

# 仅前端构建（TypeScript 检查 + Vite 打包）
npm run build

# 预览前端构建结果
npm run preview
```

## 📁 项目结构

```
IPSwitcher/
├── src/                          # 前端源码
│   ├── components/               # React 组件
│   │   ├── ProfileList.tsx       # 配置方案列表（侧边栏）
│   │   ├── ProfileForm.tsx       # 方案编辑表单
│   │   ├── ProfileCard.tsx       # 方案卡片展示
│   │   ├── InterfaceSelector.tsx # 网络接口选择器
│   │   ├── DnsEditor.tsx         # DNS 服务器编辑器
│   │   ├── SwitchConfirmDialog.tsx # 切换确认对话框
│   │   └── StatusBar.tsx         # 底部状态栏
│   ├── hooks/                    # 自定义 Hooks
│   │   ├── useProfiles.ts        # 方案 CRUD + 活跃方案管理
│   │   └── useNetwork.ts         # 网络接口/配置/权限
│   ├── types/                    # TypeScript 类型定义
│   ├── App.tsx                   # 主应用入口
│   └── main.tsx                  # Vite 入口
│
├── src-tauri/                    # Rust 后端
│   └── src/
│       ├── commands/             # Tauri IPC 命令
│       │   ├── profiles.rs       # 方案 CRUD（5 个命令）
│       │   ├── network.rs        # 网络操作（6 个命令）
│       │   └── interfaces.rs     # 接口枚举
│       ├── models/               # 数据模型
│       │   └── profile.rs        # Profile 模型 + 验证规则
│       ├── platform/             # 跨平台网络管理
│       │   ├── mod.rs            # NetworkManager trait
│       │   ├── macos.rs          # macOS 实现
│       │   └── windows.rs        # Windows 实现
│       ├── storage/              # 数据存储
│       │   └── sqlite.rs         # SQLite 存储层
│       ├── lib.rs                # 应用初始化
│       ├── admin.rs              # 权限检查与提升
│       ├── error.rs              # 错误类型定义
│       └── tray.rs               # 系统托盘
│
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## 💻 平台支持

IPSwitcher 通过 `NetworkManager` trait 抽象了平台差异，为 macOS 和 Windows 提供统一的操作接口。

### macOS

- 通过 `networksetup` 命令管理网络配置
- 使用 `osascript` (AppleScript) 请求管理员权限提升
- 以 Accessory 模式运行（不显示 Dock 图标，仅系统托盘）
- 最低系统要求：macOS 10.15 (Catalina)
- 数据存储路径：`~/Library/Application Support/com.ipswitcher.app/`

### Windows

- 通过 `netsh` 命令管理网络配置
- 使用 PowerShell 进行权限提升
- 数据存储路径：`%APPDATA%\IPSwitcher\`

## 📄 许可证

本项目采用 [MIT License](LICENSE) 开源许可证。

# TustPortal Windows 移植 — 实施报告

## 概述

**项目：** TustPortal（天科大校园网自动登录）—— 基于 Tauri 2 的校园网自动登录工具  
**原始范围：** 仅支持 macOS  
**本次移植：** 完整 Windows 支持，含 Windows 独占的开机自启及首次启动体验  
**验证状态：** 12 项后端单元测试全部通过，完整应用构建并运行成功  
**日期：** 2026-06-05

macOS 独有的代码仅约 90 行，其余部分（HTTP 客户端、凭据存储、后台轮询循环、托盘菜单、日志系统、Vue 界面）本身已是跨平台的。本次新增 4 个模块和 3 个前端桥接层，共计约 350 行新 Rust 代码和约 120 行前端改动。

---

## 文件清单

### 新增文件（8 个后端，2 个前端，4 个文档）

| 文件 | 用途 |
|---|---|
| `src-tauri/src/platform/network_windows.rs` | Windows 平台的 WiFi SSID（`netsh wlan show interfaces`）和 IPv6（`ipconfig`）检测 |
| `src-tauri/src/auto_start.rs` | 基于注册表的开机自启控制器（通过 `reg.exe` 命令行实现） |
| `src-tauri/src/first_launch.rs` | 纯决策函数：仅在 Windows 且无已保存凭据时显示设置窗口 |
| `src/native/platform.ts` | 前端桥接：`getPlatform()` → 返回 `"windows"` 或 `"macos"` |
| `src/native/auto_start.ts` | 前端桥接：`getAutoStartEnabled()` / `setAutoStartEnabled()` |
| `.github/workflows/build-windows.yml` | Windows CI 工作流（`v*` 标签触发，生成 `.exe` + `.msi`） |
| `README_WD.md` | Windows 平台安装、使用与开发文档 |
| `CONTEXT.md` | 领域术语表（校园网、忽略 SSID、自动登录、Portal、开机自启、首次启动） |
| `docs/adr/0001-windows-auto-start.md` | 开机自启功能的架构决策记录（ADR） |
| `PRD-Windows-Port.md` | 完整产品需求文档（29 条用户故事、8 项技术决策、12 项测试） |

### 修改文件（5 个后端，1 个前端）

| 文件 | 改动内容 |
|---|---|
| `src-tauri/src/platform/mod.rs` | 添加 `#[cfg(target_os = "windows")]` 条件编译门控及 `network_windows` 重导出 |
| `src-tauri/src/tray.rs` | 新增 Windows 托盘左键点击处理（`handle_tray_click`）；`ActivationPolicy::Accessory` 用 `#[cfg(target_os = "macos")]` 门控 |
| `src-tauri/src/lib.rs` | 添加 `auto_start` + `first_launch` 模块；条件性显示/销毁初始窗口；`invoke_handler` 扩展 3 个新命令 |
| `src-tauri/src/js_bridge.rs` | 新增 `get_platform`、`get_auto_start_enabled`、`set_auto_start_enabled` 命令（cfg 门控移至辅助函数，避免宏展开问题） |
| `src-tauri/src/sign_in.rs` | User-Agent 从 `Macintosh; Intel Mac OS X 10_15_7` 改为通用 `Windows NT 10.0; Win64; x64` Chrome 字符串 |
| `src/views/SettingsPage.vue` | 新增平台检测、开机自启开关（仅 Windows）、首次启动模态弹窗 |

### 未修改文件

- `src-tauri/src/platform/network_macos.rs` — 未触碰
- `src-tauri/src/background.rs` — 未触碰
- `src-tauri/src/store/*` — 未触碰
- `.github/workflows/build-macos.yml` — 未触碰
- `README.md` — 未触碰
- 其余所有 Vue 组件、路由、类型定义 — 未触碰

---

## 架构

```
                          ┌────────────────────────┐
                          │    Vue 3 前端           │
                          │  SettingsPage.vue       │
                          │  ┌──────────────────┐   │
                          │  │ 首次启动模态弹窗  │   │
                          │  │ 开机自启开关      │   │
                          │  └──────────────────┘   │
                          └─────────┬──────────────┘
                                    │ invoke()
                          ┌─────────▼──────────────┐
                          │   js_bridge.rs          │
                          │   (Tauri 命令层)         │
                          │                         │
                          │   get_platform() → str   │
                          │   get_auto_start_enabled │
                          │   set_auto_start_enabled │
                          │   try_login / check_net  │
                          │   save/load_credentials  │
                          └──┬───────┬───────┬──────┘
                             │       │       │
            ┌────────────────▼┐  ┌───▼─────┐ ┌▼──────────────┐
            │ platform/        │  │ auto_    │ │ first_launch.rs │
            │  mod.rs          │  │ start.rs │ │                  │
            │                  │  │          │ │ should_show() →  │
            │ #[cfg(macos)]    │  │ is_enabled()  │ bool             │
            │  network_macos   │  │ set_enabled() │                  │
            │ #[cfg(windows)]  │  │          │ └──────────────────┘
            │  network_windows │  │ (reg.exe)│
            └────────┬─────────┘  └──┬───────┘
                     │               │
            ┌────────▼─────────┐     │
            │ netsh wlan show   │     │
            │ interfaces → SSID │     │
            │ ipconfig → IPv6   │     │
            │ local_ip_address  │     │
            │ → IPv4            │     │
            └──────────────────┘     │
                          ┌──────────▼──────────┐
                          │ HKCU\...\Run\        │
                          │   TustPortal = path  │
                          └─────────────────────┘
```

### 模块设计理念

各模块遵循**深模块**（deep module）模式：将复杂的实现细节隐藏在简洁、稳定的接口之后，使上层代码无需了解底层操作系统的具体机制。

**1. 平台网络提供者** (`network_windows.rs`) —— 深模块  
接口：`get_wifi_ssid()`、`get_local_ipv4()`、`get_local_ipv6()` —— 均返回 `Option<String>`。  
隐藏的复杂性：`netsh` 命令行调用、面向中文区域语言的块感知解析、`ipconfig` 解析（含 `fe80::` 本地链路过滤、`%zone` 后缀剥离）。

**2. 开机自启控制器** (`auto_start.rs`) —— 深模块  
接口：`is_enabled() -> bool`、`set_enabled(bool) -> Result<(), String>`。  
隐藏的复杂性：`reg.exe` 执行、错误转换、通过 `std::env::current_exe()` 获取可执行文件路径、互斥锁保护的测试隔离。

**3. 首次启动协调器** (`first_launch.rs`) —— 浅模块  
接口：`should_show_window_on_startup(has_credentials: bool, platform: &str) -> bool`。  
纯决策函数，无外部依赖。

**4. 托盘事件路由** (`tray.rs`) —— 浅适配器  
将托盘事件连接到已有的窗口管理函数。通过 `#[cfg(target_os = "windows")]` 门控实现平台差异化的左键点击行为。

---

## 关键技术决策

### 1. 中文区域语言兼容

目标用户运行的系统是简体中文 Windows 10/11。所有解析器均基于中文 Windows 实机输出进行了验证：

- `netsh wlan show interfaces` 在中文系统上输出的 SSID 字段键名仍然是 ASCII 字符串 `SSID`。解析器使用精确键名匹配（`key == "SSID"`，避免与 `AP BSSID` 键混淆），并采用块感知扫描：以 `"\n\n"` 分段，优先处理包含 `已连接`/`connected` 的适配器块。
- `ipconfig` 在中文系统上 IPv6 行仍包含 `IPv6` 标识。解析器跳过 `fe80::` 开头的本地链路地址，并剥离 `%zone` 后缀。

### 2. `reg.exe` 命令行方案 vs 原生 API

选择 `reg.exe` 而非原生 Win32 注册表 API 或 `tauri-plugin-autostart` 插件的原因：
- 避免引入在 macOS 上无用的依赖
- 与网络检测（`netsh`、`ipconfig`）已有的命令行调用风格保持一致
- `HKCU`（当前用户）无需提权，适合实验室公用电脑场景
- 通过互斥锁保护的三项测试，防止并行执行时的注册表冲突

### 3. Vue 模态弹窗 vs 原生对话框

开机自启的首次询问使用 Vue 模态覆盖层渲染，而非原生 `MessageBox` 的原因：
- 无需额外引入原生对话框依赖
- 与首次启动时已经打开的设置窗口体验保持一致
- 实现、测试及样式维护更简单，与应用的其余界面风格统一

### 4. 独立的 CI 工作流

Windows CI（`build-windows.yml`）与现有的 macOS 工作流分离，不采用统一矩阵的方式：
- 隔离 macOS 发布流程，避免受 Windows 构建不确定性影响
- 符合不修改现有 macOS 基础设施的项目约束
- `v*` 标签触发，运行在 `windows-latest`，产出 NSIS `.exe`（主）和 `.msi`（次）

### 5. 凭据存储

保持现状：明文 JSON 文件存储在 `%AppData%\com.tust.portal\credentials.json`。本次移植未引入加密层。

### 6. User-Agent

`sign_in.rs` 中的 User-Agent 从 `Macintosh; Intel Mac OS X 10_15_7` 改为通用 `Windows NT 10.0; Win64; x64` Chrome UA 字符串。Portal 目前未对操作系统做限制，但原字符串会错误地将 Windows 流量标记为 Mac 流量。

---

## 测试结果

12 项后端单元测试全部通过（验证环境：Windows 11，Rust 1.96.0 stable-x86_64-pc-windows-msvc）：

| 模块 | 测试用例 | 结果 |
|---|---|---|
| `first_launch` | `windows_no_credentials_shows_window`（Windows 无凭据时显示窗口） | ✅ |
| | `windows_with_credentials_hides_window`（Windows 有凭据时隐藏窗口） | ✅ |
| | `macos_no_credentials_hides_window`（macOS 无凭据时隐藏窗口） | ✅ |
| | `macos_with_credentials_hides_window`（macOS 有凭据时隐藏窗口） | ✅ |
| `network_windows` | `extracts_ssid_from_chinese_windows_netsh_output`（从中文 Windows netsh 输出中提取 SSID） | ✅ |
| | `returns_none_when_no_ssid_present`（无 SSID 行时返回 None） | ✅ |
| | `returns_ssid_of_connected_adapter_when_multiple_exist`（多适配器时返回已连接适配器的 SSID） | ✅ |
| | `extracts_global_ipv6_from_ipconfig_output`（从 ipconfig 输出中提取全球 IPv6） | ✅ |
| | `returns_none_when_only_link_local_ipv6_present`（仅有本地链路 IPv6 时返回 None） | ✅ |
| `auto_start` | `is_enabled_returns_false_when_no_key`（无注册表键时返回 false） | ✅ |
| | `set_enabled_true_creates_key`（启用后创建注册表键） | ✅ |
| | `set_enabled_false_removes_key`（禁用后删除注册表键） | ✅ |

**完整应用构建与运行：** `pnpm tauri dev` 成功编译全部 419 个 crate，应用正常启动。

**代码质量：** 1 个编译器警告（无害）—— `auto_start.rs` 中的 `static REGISTRY_MUTEX` 仅在 `#[cfg(test)]` 内部被引用，非测试构建中被标记为死代码。

---

## 构建与运行

### 前置条件
- **Rust**（stable-x86_64-pc-windows-msvc，通过 `rustup` 安装）
- **Visual Studio Build Tools**，含"使用 C++ 的桌面开发"工作负载（用于 MSVC 链接器）
- **Node.js** 22+ 及 **pnpm** 10+

### 命令

```cmd
:: 安装前端依赖
pnpm install

:: 运行后端测试
cd src-tauri
set CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
cargo test

:: 开发模式（前后端热重载）
pnpm tauri dev

:: 生产构建（产出 NSIS .exe + .msi，位于 src-tauri/target/release/bundle/）
pnpm tauri build
```

### PATH 说明

如果 Rust 未在 `PATH` 中，请显式设置工具链路径：
```cmd
set PATH=D:\SYS_tools\.cargo\bin;%PATH%
```

---

## 明确排除范围

1. **修改 macOS 行为** — macOS 的开机自启、托盘行为、窗口生命周期、CI 流水线均不修改。
2. **凭据加密** — 密码仍以明文 JSON 存储。
3. **修改 macOS README** — 现有 `README.md` 保持不变；Windows 文档位于 `README_WD.md`。
4. **Linux 移植** — 不含 Linux 相关模块或分发目标。
5. **Microsoft Store（MSIX）分发** — 仅提供 NSIS `.exe` 和 `.msi`。
6. **未启用"忽略 SSID"时的有线网络校园网检测** — 保持严格 WiFi 绑定规则。使用有线网络的用户需在设置中开启"忽略 SSID 检测"。
7. **系统服务 / 守护进程模式** — 开机自启仅通过当前用户注册表（`HKCU`）实现，非 Windows 服务。
8. **IPv6 校园网 Portal 支持** — IPv6 检测仅供信息展示；登录交互仍基于 IPv4。
9. **原生首次启动对话框** — 开机自启询问使用 Vue 模态实现，非 Win32 消息框。

---

## 已知不足

| 项目 | 状态 |
|---|---|
| macOS 开机自启 | 未实现（明确排除项） |
| `REGISTRY_MUTEX` 死代码警告 | 仅影响外观，如需消除可加 `#[allow(dead_code)]` |
| 前端 E2E 测试 | 未实现；UI 状态足够简单，可手动验证 |
| MSI 安装包在干净系统上的测试 | 当前为 CI 产出物 —— 发布前建议手动测试 |
| Windows 深色模式下的弹窗 | 使用 Vue 深色模式样式；未经 Windows 系统级深色主题切换测试 |
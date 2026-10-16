# rdl-shell

<div align="center">

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> · <a href="./ko.md">🇰🇷 한국어</a> · <strong>🇨🇳 中文</strong> · <a href="./es.md">🇪🇸 Español</a> · <a href="./hi.md">🇮🇳 हिन्दी</a><br />
  <a href="./ar.md">🇸🇦 العربية</a> · <a href="./pt-BR.md">🇧🇷 Português</a> · <a href="./ru.md">🇷🇺 Русский</a> · <a href="./fr.md">🇫🇷 Français</a> · <a href="./id.md">🇮🇩 Bahasa Indonesia</a>
</p>

> 适用于 [remote-dl](https://github.com/GHSFS/remote-dl) 的 Windows Explorer 资源管理器右键扩展 — 在 `.url`（Internet 快捷方式）文件的右键菜单中添加 "Send to remote-dl" 选项。

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#安装指南)

</div>

---

### 概述

`rdl-shell.dll` 是一个 Windows Shell 上下文菜单处理程序，将 [`rdl`](https://github.com/GHSFS/remote-dl) CLI 客户端无缝集成到 Explorer 资源管理器中。注册后，在 `.url` Internet 快捷方式文件上右键单击会显示新菜单项 — **Send to remote-dl**（多选时为 **Send N shortcuts to remote-dl**）— 它会从快捷方式中读取 URL 并将其快速推送到所配置的边缘工作器队列中。

#### ✨ 核心特性 (v0.2.0)
- **多文件批量排队 (Multi-File Selection)**: 支持同时选中多个 `.url` 快捷方式，一键批量发送。
- **防止资源管理器卡顿 (Non-Blocking Async)**: 后台异步工作线程搭配 `DllLockGuard` RAII 生命周期守护，保障 Explorer 界面 0 冻结。
- **全格式快捷方式编码兼容**: 自动解析 UTF-8、UTF-16LE（无论带无 BOM）、UTF-16BE 及不区分大小写的 `url=` 键。
- **严格 URL 协议校验**: 调度前严格限制并校验 `http://` 与 `https://` 协议，拦截非法请求。
- **连接池复用优化**: 基于 `OnceLock` 缓存 `reqwest` 客户端，充分复用 Keep-Alive 连接与 DNS 缓存。

这是个人用 `remote-dl` 生态的第三个工具:

| 工具 | 形式 | 仓库 |
|---|---|---|
| `rdl.exe` | 终端 CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | 系统托盘 + 剪贴板监听 | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **Explorer 右键菜单** | 本仓库 |

三个工具共享 CLI 写入的同一份磁盘配置文件 (`%APPDATA%\rdl\config.json`)。

### 架构

DLL 实现的 4 个标准 COM 入口点 + 2 个 Shell 接口:

| 符号 | 用途 |
|---|---|
| `DllGetClassObject` | 返回对应于本 CLSID 的 `IClassFactory` |
| `DllCanUnloadNow` | 报告是否仍有外部引用维持 DLL |
| `DllRegisterServer` | `regsvr32` 调用时安装注册表项 |
| `DllUnregisterServer` | `regsvr32 /u` 调用时移除注册表项 |
| `IShellExtInit` | 从 Explorer 接收选定的文件 |
| `IContextMenu` | 渲染菜单项并响应 `InvokeCommand` |

### 项目结构

```
rdl-shell/
├── Cargo.toml             包清单，crate-type=cdylib (输出 DLL)
├── rust-toolchain.toml    工具链锁定 (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     构建目标 + 静态 MSVC CRT
├── build.rs               编译资源 + 注入 /DEF: 链接器参数
├── src/
│   ├── lib.rs             CLSID、REF_COUNT、DLL_HMODULE、4 个 COM 入口
│   ├── factory.rs         IClassFactory 实现
│   ├── handler.rs         IShellExtInit + IContextMenu 实现
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          reqwest::blocking 客户端，POST /api/dl
│   ├── config.rs          读取 %APPDATA%\rdl\config.json + DPAPI 解密
│   └── error.rs           crate 范围错误类型
├── tests/smoke.rs         构建验证冒烟测试
├── resources/
│   ├── app.rc             Win32 资源脚本 (清单 + VERSIONINFO)
│   ├── app.manifest       OS 兼容性声明
│   └── rdl-shell.def      按名称公开 COM 导出的模块定义文件
└── .github/workflows/
    ├── build.yml          cargo build + dumpbin /EXPORTS 验证 CI
    └── test.yml           fmt / clippy / test 的 CI
```

### 兼容性

| 项目 | 支持规格 |
|---|---|
| 操作系统 | Windows 10 1809+ 及 Windows 11 |
| 处理器架构 | 仅限 x86_64 (与 Explorer 进程位数一致) |
| Rust 工具链 | 1.75+ (`rust-toolchain.toml` 锁定 stable) |
| 链接器 | MSVC (Visual Studio Build Tools 2022) |
| 注册权限 | 机器级别 (`HKCR`) — `regsvr32` 运行需要管理员权限 |

DLL 采用静态链接 MSVC 运行时，除 Windows 核心系统动态链接库 (`KERNEL32.DLL`, `USER32.DLL`, `SHELL32.DLL`, `ADVAPI32.DLL`, `OLE32.DLL`, `CRYPT32.DLL`) 外无任何额外第三方 DLL 依赖。

### 安全注意事项

- **静态存储令牌加密** — Bearer 令牌在落盘前使用 Windows DPAPI 硬件加密，仅生成该令牌的当前 Windows 账户方可解密。
- **传输链路加密** — `reqwest` 以 `https_only=true` 构建，仅允许 TLS 加密通信，且令牌仅在 `Authorization` 请求头中传递。
- **运行时最小权限** — 仅一次性 `regsvr32` 注册需要管理员提权，实际右键菜单运行于普通权限的 Explorer 进程中。
- **无遥测** — 除向配置的工作器 URL 发起请求外，绝不进行任何外部网络请求。
- **幂等卸载** — 重复执行 `regsvr32 /u` 完全安全。

### 故障排除

| 现象 | 可能原因 | 解决方法 |
|---|---|---|
| 右键菜单未出现 | Explorer 缓存了旧的菜单项 | 重启 Explorer (`taskkill /IM explorer.exe /F && start explorer`) |
| `regsvr32` 报错 DllRegisterServer failed | 对 64 位 DLL 运行了 32 位 regsvr32 或非管理员运行 | 在管理员终端显式调用 `C:\Windows\System32\regsvr32.exe` |
| 点击菜单后无反应 | CLI 尚未完成首次身份认证 | 运行一次 `rdl auth login --token <令牌>` |
| 出现 401 凭证被拒提示 | 令牌失效或被撤销 | 在 Telegram 机器人中重新获取令牌并登录 |
| 卸载后无法删除 DLL | Explorer 进程仍在映射该 DLL | 重启 Explorer 或注销重新登录后删除 |

### 安装指南

> 仅在注册阶段需要管理员权限。

1. 前往 [Releases](https://github.com/GHSFS/rdl-shell/releases) 页面下载 `rdl-shell.dll`。
2. 存放在固定路径（例如 `C:\Tools\rdl-shell.dll`），请勿随意移动（注册表项与路径绑定）。
3. 以管理员身份打开 PowerShell 并执行：

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. 重启 Explorer 资源管理器即可加载右键扩展。

### 卸载

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

卸载后即可安全删除 DLL 文件。

### 许可证

MIT 许可证。详见 [LICENSE](../../LICENSE)。

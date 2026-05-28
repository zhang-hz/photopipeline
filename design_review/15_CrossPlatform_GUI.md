# GUI 设计审查 — #15 跨平台 GUI（macOS / Linux）

---

## 一、设计文档声明

ARCHITECTURE.md 多处将项目描述为跨平台桌面应用：

**第 1 章（开发环境）：**
```
| macOS GUI (SwiftUI) | GitHub Actions | 本地无 macOS |
| Linux GUI (GTK4+Rust) | 本地 cargo build | |
```

**第 2 章（核心架构）：**
```
GUI Layer（平台原生，gRPC 客户端）
  Windows: WinUI 3 (.NET 8)
  macOS:   SwiftUI
  Linux:   GTK4 + Rust
```

**第 11 章（项目结构）：**
```
gui/
├── linux/                # GTK4 + Rust
├── windows/              # WinUI 3 (.NET 8)
└── macos/                # SwiftUI
```

**第 10 章（CI/CD）：**
```
build-gui-windows.yml (dotnet build)
build-gui-macos.yml (xcodebuild)
```

## 二、实际存在

| 平台 | 目录 | 代码 | CI 配置 |
|------|------|------|---------|
| Windows | `gui/windows/` ✅ 完整 WPF 项目 | ✅ ~15,000+ 行 C#/XAML | ❌ 无 `.github/workflows/build-gui-windows.yml` |
| macOS | ❌ `gui/macos/` **不存在** | ❌ **0 行 Swift 代码** | ❌ 无 `build-gui-macos.yml` |
| Linux | ❌ `gui/linux/` **不存在** | ❌ **0 行 GTK/Rust 代码** | ❌ 无专用配置 |

## 三、设计承诺 vs 现实

### macOS（SwiftUI）

| 设计承诺 | 现实 |
|---------|------|
| gui/macos/ 目录 | 目录不存在 |
| SwiftUI 项目文件 | 0 个 .swift 文件 |
| gRPC 客户端（Swift） | 不存在 |
| macOS 原生界面 | 不存在 |
| macOS CI | build-gui-macos.yml 不存在 |

### Linux（GTK4 + Rust）

| 设计承诺 | 现实 |
|---------|------|
| gui/linux/ 目录 | 目录不存在 |
| GTK4 Rust 项目 | 0 行 GTK 相关代码 |
| gRPC 客户端 | 不存在 |
| Linux 原生界面 | 不存在 |

## 四、完成度评分

| 平台 | 完成度 | 说明 |
|------|:-----:|------|
| Windows WPF | **100%** | 完整实现 |
| macOS SwiftUI | **0%** | 完全未开始 |
| Linux GTK4+Rust | **0%** | 完全未开始 |
| **跨平台总计** | **33%** | 3 个平台仅完成 1 个 |

## 五、影响评估

> **设计文档将本项目定位为跨平台应用，但 66% 的平台代码从未编写。用户如果期待在 macOS 或 Linux 上使用此应用，将无法运行。**

注意：虽然 `photopipeline-server`（Rust gRPC 后端）和 `photopipeline-cli`（命令行工具）可以在任何 Rust 支持平台上运行，但**桌面 GUI 目前是 Windows-only**。

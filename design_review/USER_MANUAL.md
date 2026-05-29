# Photopipeline 用户手册

> **版本**: 2.0
> **适用于**: Photopipeline GUI v1.0 (Tauri v2 + React 19 + Fluent UI 2)
> **目标读者**: 摄影师、后期处理工程师、批量处理工作流设计者
> **篇幅**: 书籍级完整教程 (3000+ 行)

---

## 目录

1. [安装与启动](#1-安装与启动)
2. [界面概览](#2-界面概览)
3. [快速入门: 处理第一张图片](#3-快速入门-处理第一张图片)
4. [图片管理](#4-图片管理)
5. [分组管理](#5-分组管理)
6. [DAG 管线构建](#6-dag-管线构建)
7. [编辑参数与覆盖系统](#7-编辑参数与覆盖系统)
8. [插件参考](#8-插件参考)
9. [执行管线](#9-执行管线)
10. [批量处理](#10-批量处理)
11. [预览与辅助视图](#11-预览与辅助视图)
12. [设置](#12-设置)
13. [键盘快捷键](#13-键盘快捷键)
14. [典型工作流程教程](#14-典型工作流程教程)
15. [PipelineConfig JSON 参考](#15-pipelineconfig-json-参考)
16. [常见问题 FAQ](#16-常见问题-faq)

---

## 1. 安装与启动

### 1.1 系统要求

Photopipeline 是一款专业级跨平台图像后处理管线应用，采用 Tauri v2 原生壳 + React 19 Web 前端 + Fluent UI 2 设计系统。在处理 24MP 以上 RAW 文件时，建议使用推荐配置以获得流畅体验。

| 要求 | 最低配置 | 推荐配置 |
|------|---------|---------|
| 操作系统 | Windows 10 21H2 / macOS 12 Monterey / Linux (Wayland) | Windows 11 24H2 / macOS 14 Sonoma+ |
| CPU | 双核 64 位 | 8 核以上 (批量并行) |
| 内存 | 4 GB | 16 GB+ (AI 降噪需要更大) |
| GPU | 集成显卡, OpenGL 3.3+ | NVIDIA CUDA 12+ / Apple Silicon M1+ (AI 降噪加速) |
| VRAM | 512 MB | 2 GB+ (AI 降噪 ONNX 推理) |
| 磁盘空间 | 500 MB (应用本体) | SSD, 10 GB+ 可用 (图像处理缓存) |
| 网络 | 初次启动可选 (检查更新) | 无需联网 (离线完全可用) |
| 显示分辨率 | 1280x720 | 1920x1080+ (三栏布局最佳体验) |

### 1.2 获取安装包

**发布渠道**:

1. **GitHub Releases** (推荐): 访问 `https://github.com/photopipeline/photopipeline/releases` 下载最新稳定版。
2. **官方网站**: 访问 `https://photopipeline.app/download` 选择对应平台。
3. **包管理器** (Linux): `flatpak install photopipeline` 或 `snap install photopipeline`。

### 1.3 安装步骤

#### Windows 安装

1. 下载 `Photopipeline_Setup_{版本号}_x64.msi` 文件。
2. 双击 MSI 文件启动安装向导。
3. 在欢迎页面点击"下一步"。
4. 阅读并接受 MIT 开源许可协议。
5. 选择安装目录 (默认 `%LOCALAPPDATA%/Programs/Photopipeline`)。
6. (可选) 勾选"创建桌面快捷方式"和"注册 .ppjson 文件关联"。
7. 点击"安装"，等待进度条完成。
8. 点击"完成"——安装程序会自动将 `photopipeline-server.exe` 注册到系统 PATH。

**注意**: Windows 安装程序会自动安装 Visual C++ 2022 运行时库 (如尚未安装)。如果手动部署，请确保已安装 VC++ Redist 14.40+。

#### macOS 安装

1. 下载 `Photopipeline_{版本号}_aarch64.dmg` (Apple Silicon) 或 `Photopipeline_{版本号}_x64.dmg` (Intel)。
2. 双击 DMG 文件挂载磁盘映像。
3. 将 `Photopipeline.app` 拖入 `Applications` 文件夹。
4. 首次启动时，系统可能提示"无法验证开发者"。请前往 **系统设置 > 隐私与安全性** 点击"仍要打开"。
5. (可选) 从 DMG 中将 `photopipeline-server` 二进制文件复制到 `/usr/local/bin/` 以便命令行使用。

**注意**: Apple Silicon 版本原生支持 CoreML ANE 引擎，AI 降噪性能显著优于 Intel 版本。

#### Linux 安装

**AppImage**:
1. 下载 `Photopipeline_{版本号}_x86_64.AppImage`。
2. 在终端中运行: `chmod +x Photopipeline_*.AppImage`。
3. 双击运行，或终端执行: `./Photopipeline_*.AppImage`。
4. (可选) 移动到 `~/.local/bin/` 并创建桌面入口文件。

**手动部署** (通用):
1. 下载 `photopipeline_{版本号}_linux.tar.gz`。
2. 解压: `tar xzf photopipeline_*.tar.gz -C ~/apps/photopipeline/`。
3. 将 `~/apps/photopipeline/bin/` 加入 `PATH`。
4. 运行 `photopipeline` 启动 GUI。

**依赖检查**: Linux 需要以下系统库: `libwebkit2gtk-4.1`, `libgtk-3`, `libglib2.0`, `libssl3`, `libayatana-appindicator3`。使用包管理器安装: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev` (Debian/Ubuntu) 或 `sudo dnf install webkit2gtk4.1-devel gtk3-devel` (Fedora)。

### 1.4 启动应用

1. 双击桌面快捷方式或从开始菜单启动 Photopipeline。
2. 应用启动后首先显示启动画面 (Splash Screen, ~2 秒)。
3. 启动画面消失后，应用自动执行以下操作:
   - 初始化 Tauri 运行时
   - 在后台启动处理引擎 (`photopipeline serve --port 50051`)
   - 通过 gRPC 健康检查 (`HealthCheck` RPC) 确认后端就绪
   - 调用 `ListPlugins` RPC 加载所有可用插件
4. 状态栏右下角显示绿色圆点 "Connected"，表示后端引擎完全就绪。
5. 如果连接失败，状态栏显示红色圆点 "Disconnected"，此时:
   - 所有编辑操作被禁用 (UI 控件灰色不可点击)
   - 底部显示错误信息 "Backend disconnected — Retrying in 5s..."
   - 系统每 5 秒自动重试连接，最多重试 10 次
6. 如果 50 秒内仍未连接成功，弹出错误对话框提示用户检查后端进程。

### 1.5 首次启动界面

首次启动 Photopipeline 时，应用处于**空态 (Empty State)**:

- **左栏 (Sidebar)**: 显示空胶片条，中央有大图标 + "No images loaded — Click Import or drag files here" 占位文字。
- **中栏 (Content)**: 显示空白 DAG 画布，中央有 "Add images to begin" 引导提示。
- **右栏 (Panel)**: 插件浏览器正常显示，可以浏览 14 个内置插件卡片，但参数面板显示为空。

### 1.6 关闭应用

1. 点击窗口右上角关闭按钮 (X)，或使用快捷键 `Alt+F4` (Windows/Linux) / `Cmd+Q` (macOS)。
2. 如果管线有未保存的修改，弹出确认对话框: "Pipeline has unsaved changes. Save before closing?" — 提供 [Save & Close] / [Discard & Close] / [Cancel] 三个选项。
3. 如果批量处理正在运行中，弹出警告: "Batch processing is in progress (12/50). Closing will cancel all pending items." — 提供 [Stop & Close] / [Cancel] 两个选项。
4. 关闭后，应用自动执行清理:
   - 向 `photopipeline-server` 发送 `Shutdown` RPC
   - 等待后端进程正常退出 (超时 5 秒)
   - 如果后端未响应，强制终止进程 (SIGTERM / TerminateProcess)
   - 清除 `%TEMP%/photopipeline/` 下的临时文件
   - 将最后状态写入 `appsettings.json`

---

## 2. 界面概览

### 2.1 窗口布局全景

Photopipeline 的主窗口采用经典的三栏布局，从上到下分为标题栏、主三栏区域、状态栏三个水平带:

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ◆ Photopipeline — HDR Pipeline v1          [Editor] [Batch 0]    ◐   ⚙  │ ← TitleBar (44px)
├───────────────┬────────────────────────────┬─────────────────────────────┤
│               │                            │                             │
│  胶片条        │    DAG 画布                 │   插件控制面板               │
│  (图片列表)    │    (管线编辑)               │   (参数编辑)                 │
│               │                            │                             │
│  [Import]     │  ┌─────┐    ┌───────┐     │  [Template] [Group] [Image] │ ← ContextBar
│  [Clear]      │  │ raw │───→│ ai    │     │  ▼ Resize        inherited  │
│  [To Batch]   │  │input│    │denoise│     │    Width: [1920 ▲▼] px ⬜   │
│               │  └─────┘    └───┬───┘     │    Height:[1080 ▲▼] px ⬜   │
│  图片列表      │                │          │                             │
│  分组管理      │  ┌─────────────┘          │  辅助视图                    │
│               │  │                         │  HISTOGRAM ▂▃▅▆▇▆▅▃▂▁    │
│               │  └─→ ┌──────────┐          │                             │
│               │      │ heif     │          │  前后对比预览                 │
│               │      │ encoder  │          │  Before │ After             │
│               │      └──────────┘          │                             │
│               │                            │                             │
│               │  [Plugin Browser — 底部]    │                             │
├───────────────┴────────────────────────────┴─────────────────────────────┤
│ ▶ Batch: 8/12  ██████████░░  65%  │  Mem:512MB · GPU:Ready  │ ● Connected │ ← StatusBar (36px)
└──────────────────────────────────────────────────────────────────────────┘
```

### 2.2 界面各区域详解

| 区域 | 位置 | 尺寸 | 核心功能 |
|------|------|------|---------|
| **TitleBar** | 顶部 | 100% x 44px | 应用标题、模式切换标签、主题切换、设置入口 |
| **Sidebar** | 左侧 | w=272px, 可拖拽 | 图片胶片条、分组树、导入/清除/发送批量按钮 |
| **Content** | 中间 | flex:1, min=400px | DAG 管线画布、插件浏览器横条、画布工具栏 |
| **Panel** | 右侧 | w=440px, 可拖拽 | 插件参数面板、ContextBar 层级切换、辅助视图、预览 |
| **StatusBar** | 底部 | 100% x 36px | 批量进度条、后端连接状态、内存/GPU 信息 |

### 2.3 两种工作模式

Photopipeline 有**两种工作模式**，通过 TitleBar 上方的模式标签切换。这两种模式共享同一图片列表和管线定义，但界面布局和侧重点不同。

| 模式 | 标签外观 | 用途 | 界面布局 |
|------|---------|------|---------|
| **Pipeline Editor** | `[Pipeline Editor]` (蓝色激活态) | 构建管线、调整参数、实时预览 | 三栏: 胶片条 / DAG画布 / 参数面板 |
| **Batch Processing N** | `[Batch Processing N]` (N=队列中图片数) | 批量处理队列、监控进度、逐图覆盖 | 三栏: 管线摘要 / 队列+进度 / 输出设置 |

**模式切换行为**:

1. 在编辑模式中点击 `[Batch Processing N]` 标签 → 切换到批量模式，界面变为队列+进度视图。
2. 在批量模式中点击 `[Pipeline Editor]` 标签 → 切换回编辑模式，可继续编辑管线。
3. 在编辑模式中点击侧边栏 **To Batch** 按钮 → 选中图片加入批量队列，Badge 数字 +N，保持在编辑模式 (不自动切换)。
4. 两个模式**同时保持状态** — 切换到批量模式不会清除编辑模式的画布，反之亦然。

### 2.4 TitleBar 详解

```
┌──────────────────────────────────────────────────────────────────┐
│ ◆ Photopipeline — HDR Pipeline v1     [Editor] [Batch 0]  ◐  ⚙  │
└──────────────────────────────────────────────────────────────────┘
```

| 元素 | 位置 | 功能 |
|------|------|------|
| **Logo** ◆ | 最左 | 22x22px 品牌图标, 品牌蓝色背景 |
| **标题** | Logo 右侧 | "Photopipeline — {管线名称}" |
| **模式标签** | 中间偏右 | [Pipeline Editor] 和 [Batch Processing N] |
| **主题切换** ◐ | 右侧 | 一键切换 Dark / Light / System 主题 |
| **设置齿轮** ⚙ | 最右 | 打开设置对话框 (详见第 12 章) |

### 2.5 Sidebar 详解

Sidebar 由以下子组件垂直排列:

1. **SidebarHeader**: 标题栏 `CANDIDATE FILES` + 文件计数 `12 images`。
2. **SidebarToolbar**: 三个操作按钮 — [Import (蓝色主按钮)] [Clear] [To Batch]。
3. **SortBar**: 排序下拉菜单 (Name/Size/Format/ISO) + 缩略图大小切换 (S/M/L)。
4. **MultiSelectBar**: 条件显示 (选中图片 >= 2 时出现)，提供 [+Group] [To Batch] [Clear] 操作。
5. **FilmstripList**: 图片卡片列表，可滚动的垂直列表。
6. **GroupTree**: 分组管理区域，列出所有分组并提供创建/编辑/删除操作。

### 2.6 Content 详解

Content 区域包含:

1. **ContentHeader**: 左侧绿色状态圆点 + "Pipeline Editor" 标题 + 右侧节点数/缩放比例。
2. **DAGToolbar**: 工具栏 — New / Save / Load | Validate | Run / Cancel | Zoom+ / Zoom- / Fit。
3. **DAGCanvas**: 可缩放/平移的 DAG 图形编辑画布，32px 网格背景。
4. **PluginBrowser**: 底部插件卡片横条，3 列网格布局，支持搜索和分类筛选。

### 2.7 Panel 详解

Panel 从顶部到底部排列:

1. **ContextBar**: 覆盖层级切换标签栏 — [All] [Template] [Group Name...] [Image Name]。
2. **PluginHeader**: 选中插件的名称、版本、标签、硬件需求信息。
3. **ParamSection** (xN): 可折叠参数分区，每区含标题、覆盖徽章、参数行列表。
4. **ExpressionEditor**: 表达式编辑器 (仅支持表达式的插件显示)。
5. **AuxView** (xN): 辅助视图区域 — Histogram / Waveform / Vectorscope / GamutDiagram / Map / StatusText / ProgressBar。
6. **Preview**: 前后对比分屏预览 (BeforeAfter 模式)。
7. **RemoveButton**: 底部全宽的红色 [Remove from Pipeline] 按钮。

### 2.8 StatusBar 详解

```
┌──────────────────────────────────────────────────────────────────┐
│ ▶ Batch: 8/12  ██████████░░░  65%  12 done  3 failed               │
│ ⏱ 00:03:15  ~00:01:20  12 img/min  │  Mem:512MB · GPU:Ready  │ ● Connected │
└──────────────────────────────────────────────────────────────────┘
```

| 元素 | 说明 |
|------|------|
| **批量指示** ▶ | 播放图标, 仅在批量处理运行时显示 |
| **进度条** | 品牌蓝色填充, 实时更新批量/执行进度 |
| **进度数字** | 完成计数 + 失败计数 |
| **计时器** ⏱ | 已用时间、预计剩余时间 (ETA) |
| **速度信息** | 处理速率 (img/min) |
| **内存** | 当前内存使用量 |
| **GPU** | GPU 就绪状态和温度 (如适用) |
| **连接状态** ● | 绿色=已连接, 红色=断开 |

---

## 3. 快速入门: 处理第一张图片

本节面向首次使用的用户，以最快的速度走完从导入到输出的完整流程。每个步骤都有详细编号和预期结果说明。

### 3.1 步骤 1: 导入图片

1. 确认 Photopipeline 已启动，状态栏右下角显示绿色圆点 "Connected"。
2. 在左侧 Sidebar 中找到 **Import** 按钮 (品牌蓝色，带 📂 图标)。
3. 点击 Import 按钮，系统弹出原生文件选择对话框。
4. 在对话框中导航到你的图片目录。
5. 选择一张或多张 RAW 文件 (支持 ARW/CR2/CR3/NEF/DNG/RAF/ORF/RW2/PEF/TIFF/PNG/JPEG 等格式)。
6. 也可以使用 Ctrl+点击 或 Shift+点击 进行多选 (最多可同时选择 500 张)。
7. 点击"打开"确认。

**预期结果**:
- 选中图片出现在左侧胶片条中，每张显示缩略图、文件名、分辨率、ISO 等元数据。
- DAG 画布中央自动创建一个 **raw_input** 节点，节点顶部显示 "auto · N files" 徽章 (N 等于导入的图片数)。
- ContentHeader 的节点数变为 "1 node"。

**备用导入方式**:
- **拖放导入**: 从 Windows 资源管理器 / macOS Finder 直接拖放图片文件到胶片条区域。
- **快捷键导入**: 按 `Ctrl+O` 打开文件选择对话框。

### 3.2 步骤 2: 添加处理节点

现在管线只有一个 raw_input 节点。接下来添加处理节点。

1. 查看底部 **PluginBrowser** (插件浏览器) — 显示 14 个内置插件的卡片网格。
2. 使用搜索框快速定位: 在 PluginBrowser 搜索框中输入 "denoise" 筛选出 AI Denoise 插件。
3. 双击 **AI Denoise** 插件卡片 → 节点自动添加到 DAG 画布上 raw_input 右侧。
4. 点击搜索框的清除按钮 (X)，恢复显示全部插件。
5. 在卡片网格中找到 **HEIF Encoder** (或使用搜索框)。
6. 拖放 **HEIF Encoder** 卡片到 DAG 画布右侧空白区域。

**预期结果**:
- DAG 画布现在有三个节点: raw_input、ai_denoise、heif_encoder。
- 各节点以其分类颜色标识: raw_input=红色, ai_denoise=紫色, heif_encoder=青色。
- 每个节点左侧有绿色输入端口 ⊡, 右侧有蓝色输出端口 ⊡。
- ContentHeader 显示 "3 nodes"。

**三种添加节点方式回顾**:

| 方式 | 操作 | 适用场景 |
|------|------|---------|
| **双击** | 双击 PluginCard | 快速添加, 节点自动排在末尾 |
| **拖放** | 从 PluginBrowser 拖动到画布 | 精确控制节点位置 |
| **右键菜单** | 右键画布空白 → Add Node → 选插件 | 了解可用插件, 精确添加 |

### 3.3 步骤 3: 连接节点

节点之间还没有连线，管线无法执行。需要按处理顺序连接节点。

1. 将鼠标悬停在 **raw_input** 节点的**蓝色输出端口** ⊡ (位于节点右侧) 上。
2. 端口放大 1.3 倍并高亮，指示可以拖拽。
3. 按住鼠标左键，从 raw_input 的输出端口拖出 → 一根蓝色虚线跟随光标。
4. 将虚线拖到 **ai_denoise** 节点的**绿色输入端口** ⊡ (位于节点左侧) 上。
5. 输入端口高亮表示可以接受连接。
6. 释放鼠标左键 → 创建一条蓝色贝塞尔曲线连线。
7. 重复以上步骤，将 ai_denoise 的输出端口连接到 heif_encoder 的输入端口。

**预期结果**:
- 节点链: `raw_input ───→ ai_denoise ───→ heif_encoder`。
- 连线呈现品牌蓝色 (#4793ff)，2px 宽，不透明度 0.55。
- 悬停连线时加粗至 3px 并变为完全不透明，表示当前关注的连线。

### 3.4 步骤 4: 调整参数

1. **单击** DAG 画布中的 **ai_denoise** 节点将其选中 → 节点边框变为品牌蓝色并带发光阴影。
2. 右侧 Panel 切换到 ai_denoise 插件的参数面板。
3. 在 **Model** 分区中，从 `Denoise Model` 下拉框选择 "Standard v2 (balanced)" (推荐选项带 ★ 标记)。
4. 在 **Strength** 分区中:
   - 将 `Denoise Strength` 滑块从默认 50 拖动到 65。
   - 将 `Detail Preservation` 滑块拖动到 70。
5. 观察参数值右侧的覆盖标记: 每个被你修改的参数右侧出现 **🟡 (覆盖)** 标记。
6. 单击 **heif_encoder** 节点，检查输出质量设置: `Quality` 默认 95 保持不变。

### 3.5 步骤 5: 验证管线

在执行前，验证管线确保配置正确。

1. 点击 DAGToolbar 的 **Validate** 按钮 (带 ✓ 图标)。
2. 系统调用后端 `PipelineService.Validate` RPC 进行全面检查:
   - 所有节点引用的插件是否存在
   - 节点间数据类型是否兼容 (例如: pixel output → pixel input)
   - 参数值是否在合法范围内 (min/max/pattern)
   - 是否有孤立的节点 (未连接且非输入/输出类型)
   - 是否存在循环依赖 (A→B→C→A)
3. 验证结果在底部弹出信息面板:
   - 绿色 "Validation passed — 3 nodes, 2 edges, 0 issues" 表示通过。
   - 黄色警告不阻止执行，红色错误阻止执行。

### 3.6 步骤 6: 执行管线

1. 确认管线连接完整: raw_input → ai_denoise → heif_encoder。
2. 确认状态栏连接状态为绿色 "Connected"。
3. 点击 DAGToolbar 的 **▶ Run** 按钮 (品牌蓝色主按钮)。
4. **Run** 按钮变为灰色不可点击，**⏹ Cancel** 按钮出现。
5. 观察执行过程:
   - 当前执行节点显示**绿色闪烁**状态灯。
   - 状态栏进度条实时更新 (Loading → Decoding → Processing → Encoding → Done)。
   - StatusBar 显示当前阶段: "Processing: ai_denoise · Tile 3/8 · 42%"。
6. 如果想取消，点击 ⏹ Cancel → 当前图片完成处理后停止。
7. 完成后所有节点状态灯变为绿色常亮。
8. StatusBar 显示: "Done — 24.3 MB in 00:03.42"。
9. 输出文件默认保存在原始图片所在目录，文件名为 `{原文件名}_processed.heic`。

**恭喜!** 你已经完成了第一张图片的处理。这只是 Photopipeline 的最基本用法。接下来将深入讲解每个功能。

---

## 4. 图片管理

### 4.1 图片导入完整指南

#### 支持的格式

Photopipeline 支持以下图像格式的导入:

| 格式类别 | 扩展名 | 说明 |
|---------|--------|------|
| **Sony RAW** | .ARW | Sony Alpha 系列相机 |
| **Canon RAW** | .CR2, .CR3 | Canon EOS/R 系列相机 |
| **Nikon RAW** | .NEF | Nikon Z/D 系列相机 |
| **通用 RAW** | .DNG | Adobe Digital Negative |
| **Fujifilm RAW** | .RAF | Fujifilm X/GFX 系列 |
| **Olympus RAW** | .ORF | Olympus/OM System |
| **Panasonic RAW** | .RW2 | Panasonic Lumix |
| **Pentax RAW** | .PEF | Pentax 数码相机 |
| **TIFF** | .TIF, .TIFF | 支持 8/16/32-bit, LZW/ZIP/无压缩 |
| **PNG** | .PNG | 支持 8/16-bit, RGBA |
| **JPEG** | .JPG, .JPEG | 标准 JPEG |
| **HEIF/HEIC** | .HEIC, .HEIF | 支持 8/10-bit |

**不支持直接导入**: PSD, PDF, SVG, GIF, BMP — 请先用其他工具转换为 TIFF/PNG。

#### 导入方式详解

**方式一: Import 按钮**

1. 点击 Sidebar 左侧工具栏中的 **Import** 按钮 (品牌蓝色)。
2. 打开的文件选择对话框内默认过滤显示支持的图像格式。
3. 导航至目录，选择文件 (支持多选):
   - `Ctrl+点击` 追加/取消单个文件。
   - `Shift+点击` 选择连续范围。
   - `Ctrl+A` 全选当前目录下所有支持的文件。
4. 点击"打开" → 系统读取每张图片的元数据并生成缩略图。
5. 导入进度: 如果选择大量文件 (比如 100 张以上)，Import 按钮变为禁用状态并显示转圈动画，StatusBar 显示 "Loading 3/45..."。

**方式二: 拖放导入**

1. 在操作系统的文件管理器 (Windows 资源管理器 / macOS Finder) 中选中图片文件。
2. 按住鼠标左键拖动文件到 Photopipeline 窗口的**胶片条区域**。
3. 当鼠标悬停在胶片条上方时，该区域显示蓝色虚线边框 + "Drop images here to import" 提示文字。
4. 释放鼠标 → 图片开始导入。

**方式三: 快捷键导入 `Ctrl+O`**

1. 按下 `Ctrl+O` (macOS: `Cmd+O`)。
2. 文件选择对话框弹出，操作与 Import 按钮相同。

**批量导入建议**:
- 单次导入 **50 张以内**: 性能最佳，界面立即响应。
- 单次导入 **50-200 张**: 可接受，缩略图分批加载 (每批 10 张)。
- 单次导入 **200 张以上**: 建议分批导入或使用命令行批量导入 (`photopipeline import --recursive /path/to/photos/`)。

#### 导入后的自动处理

导入完成后，Photopipeline 自动执行以下操作:

1. **生成缩略图**: 调用 `ImageService.GetThumbnail(max_size=256)` gRPC, 生成 JPEG 格式的 256px 缩略图用于胶片条显示。缩略图缓存在 `%APPDATA%/Photopipeline/cache/thumbnails/`。
2. **读取元数据**: 调用 `ImageService.Load()` 提取 EXIF 信息:
   - 基本: make, model, lens_model
   - 曝光: iso, aperture (f-number), shutter_speed, exposure_bias
   - 时间: date_time_original, offset_time
   - GPS: latitude, longitude, altitude (如果有)
   - 图像: width, height, pixel_format, color_space
3. **自动创建 raw_input 节点**: 如果 DAG 画布没有 raw_input 节点，系统自动创建一个并关联所有导入的图片。
4. **自动分组** (如果启用): 如果 `auto_group` 设置开启，系统根据 EXIF 数据自动创建分组:
   - 按相机型号分组
   - 按 ISO 范围分组 (Low ISO < 400, Medium 400-1600, High > 1600)
   - 按拍摄日期分组

### 4.2 图片卡片 (ImageCard)

#### 卡片布局

```
┌──────────────────────────────────┐
│ [✓] 🖼  DSC_0034.ARW             │  ← 勾选框 (多选时显示) + 缩略图 + 文件名
│         6000x4000 ARW · ISO 6400  │  ← 元数据行
│         🟡 High ISO          24MB │  ← 分组标签 + 文件大小
└──────────────────────────────────┘
```

| 元素 | 规格 |
|------|------|
| 勾选框 | 18x18px, 仅多选时显示, 选中时品牌蓝色背景 + 白色 ✓ |
| 缩略图 | 50x34px, borderRadius 4px, JPEG 格式 |
| 文件名 | 12px fontSize, fontWeight 500, 超出宽度显示省略号 |
| 元数据行 | 10px fontSize, neutralFg4 灰色, "分辨率 · 格式 · ISO" |
| 分组标签 | 8px fontSize, 黄色背景 + 黄色文字, 仅已分组时显示 |
| 文件大小 | 10px fontSize, neutralFg4 灰色, 右对齐 |

#### 卡片状态

| 状态 | 边框样式 | 背景色 | 勾选框 | 触发条件 |
|------|---------|--------|:-----:|---------|
| default | transparent (1.5px) | neutralBg2 (#1f1f1f) | 隐藏 | 未选中 |
| hover | transparent | neutralBg3 (#292929) | 隐藏 | 鼠标悬停 |
| single-selected | brandFg1 蓝色实线 | rgba(brand,0.06) | 隐藏 | 单选 (直接单击) |
| multi-selected | brandFg1 蓝色虚线 | rgba(brand,0.03) | 显示 ✓ | 多选 (Ctrl+Click/Shift+Click) |

### 4.3 图片选择操作

#### 选择操作详解

| 操作 | 步骤 | 视觉效果 |
|------|------|---------|
| **单选** | 直接**左键单击**图片卡片 | 蓝色实线边框, 之前选中的图片取消选中 |
| **追加选择** | 按住 **Ctrl** 键, 左键单击图片卡片 | 蓝色虚线边框 + 勾选框 ✓, 不影响已选图片 |
| **取消选择** | 按住 **Ctrl** 键, 再次左键单击已选中图片 | 蓝色虚线消失, 恢复默认状态 |
| **范围选择** | 单击第一张图片, 按住 **Shift** 键, 单击最后一张图片 | 第一张到末张之间的所有图片全被选中 (虚线边框) |
| **全选** | 按 `Ctrl+A` | 所有图片进入多选状态 (虚线边框 + 勾选框) |
| **清除选择** | 按 `Escape` 键 | 所有图片恢复默认状态 |
| **反选** | 右键任意图片 → "Invert Selection" | 已选的取消, 未选的选中 |

#### 多选状态下的 MultiSelectBar

当选中 **2 张及以上** 图片时，Sidebar 中 SortBar 下方出现**黄色多选栏**:

```
┌─ 📋 3 images selected │ +Group │ To Batch │ Clear ─┐
```

| 按钮 | 功能 | 快捷键 |
|------|------|:-----:|
| **+Group** | 将选中图片加入现有分组或创建新分组 | — |
| **To Batch** | 将选中图片发送到批量处理队列 | — |
| **Clear** | 清除选择 (等同于按 Escape) | Esc |

### 4.4 右键上下文菜单

在任意图片卡片上**右键单击**弹出上下文菜单:

```
┌────────────────────────┐
│ Open in Explorer       │  → 在操作系统文件管理器中定位该文件
│ Copy Path              │  → 复制文件完整路径到剪贴板
├────────────────────────┤
│ Select All     Ctrl+A  │  → 全选所有图片
│ Clear Selection Esc    │  → 清除所有选择
│ Invert Selection       │  → 反选 (选中的取消, 未选的选中)
├────────────────────────┤
│ Add to Group →         │  → 子菜单: 选择已有分组 或 "+ New Group"
│ Send to Batch          │  → 发送该图片到批量处理队列
├────────────────────────┤
│ Remove          Del    │  → 从胶片条列表移除 (不删除磁盘上的文件)
└────────────────────────┘
```

### 4.5 排序与缩略图控制

#### SortBar

```
Sort: [Name ▾]    Size: [S] [M] [L]
```

**排序方式** (Sort 下拉):

| 排序方式 | 规则 |
|---------|------|
| **Name** | 文件名字母顺序排列 (A→Z) |
| **Size** | 文件大小排序 (大→小) |
| **Format** | 按文件扩展名分组 (ARW → CR2 → DNG → ...) |
| **ISO** | 按 ISO 感光度排序 (低→高, 无 ISO 信息的排在最后) |

**缩略图大小** (S / M / L 按钮):

| 选项 | 加载分辨率 | 说明 |
|------|:---------:|------|
| **S** | 80px | 加载最快, 缩略图最小, 适合批量浏览大量图片 |
| **M** | 120px | 默认大小, 平衡质量和速度 |
| **L** | 180px | 最清晰, 加载较慢, 适合仔细筛选 |

注意: 缩略图大小仅影响 `GetThumbnail` 的 `max_size` 参数, 不影响卡片布局尺寸。切换大小后需要重新加载缩略图。

### 4.6 从列表移除图片

**移除操作** (不删除磁盘文件):

1. 选中一张或多张图片。
2. 按 `Delete` 键，或右键 → "Remove"。
3. 如果图片在分组中，弹出提示 "This image belongs to group 'High ISO'. Remove from group only or from list?"。
4. 选择 "Remove from List" → 图片从胶片条移除。
5. 文件**不受影响**，仅从 Photopipeline 列表中移除。

---

## 5. 分组管理

### 5.1 什么是分组

分组 (Group) 是覆盖系统 (Override System) 的核心机制之一。一个分组定义了两个关键要素:

1. **匹配条件**: 哪些图片属于该分组。可以是手动指定或自动条件匹配 (如 `ISO >= 1600`)。
2. **默认覆盖参数**: 该分组内所有图片的默认参数值 — 优于 Template (管线模板) 但低于 Image (单图覆盖)。

### 5.2 GroupTree 组件

```
┌─ Groups ───────────────────────┐
│ ● High ISO (≥1600)      4  ✎ 🗑│  ← hover 出现操作按钮
│ ● Night (21-05)         3  ✎ 🗑│
│ ● GPS: Chengdu          5  ✎ 🗑│
│                                │
│ [+ Create Group…]              │  ← 虚线边框按钮
│ [Auto-group ▾]                 │  ← 虚线边框按钮
└────────────────────────────────┘
```

各元素说明:

| 元素 | 说明 |
|------|------|
| **分组名称** | 自定义名称, 如 "High ISO"、"Night Shots" |
| **条件摘要** | 灰色小字描述自动匹配条件, 如 "(>=1600)" |
| **计数徽章** | 当前满足条件或被手动加入的图片数量 |
| **✎ 编辑** | 悬停显示, 打开分组编辑对话框 |
| **🗑 删除** | 悬停显示, 删除分组 (保留图片的覆盖参数到 Template 层) |

### 5.3 创建分组

#### 方式一: 手动创建分组

1. 点击 GroupTree 底部的 **[+ Create Group...]** 虚线按钮。
2. 在弹出的 "Create Group" 对话框中:
   a. 在 Name 字段输入分组名称 (必填), 如 "Landscape HDR"。
   b. (可选) 设置自动匹配条件:
      - Field 下拉选择字段: ISO / Aperture / Shutter / Focal Length / Camera Model / Lens / GPS Region / Time Range
      - Op 下拉选择比较操作符: >=, <=, =, !=, between
      - Value 输入目标值
   c. (可选) 添加默认覆盖参数:
      - 点击 "[Add parameter override]" 下拉
      - 选择插件和参数, 如 "ai_denoise.strength"
      - 设置该组的默认值, 如 0.8
   d. 点击 **[Create]** 确认创建。
3. 如果设置了自动匹配条件，符合条件的图片自动加入该分组。
4. 如果没有设置条件，需要手动将图片加入分组。

#### 方式二: 从选中的图片创建分组

1. 选中一组图片 (Ctrl+Click 多选或 Shift+Click 范围选择)。
2. MultiSelectBar 出现 → 点击 **[+Group]** 按钮。
3. 选择 "Create New Group..."。
4. 输入分组名称 → 选中的图片自动加入该分组。
5. (可选) 勾选 "Auto-match similar" 让系统分析选中图片的共同特征 (如同款相机、相近 ISO) 并设置自动匹配条件。

#### 方式三: 自动分组 (Auto-group)

1. 点击 GroupTree 底部的 **[Auto-group ▾]** 下拉按钮。
2. 从下拉菜单选择一个分组维度:
   - **By ISO Range**: 自动按 ISO 区间分组 (Low<400, Mid 400-1600, High>1600, Ultra>6400)
   - **By GPS Cluster**: 按 GPS 坐标的地理聚类分组 (适用于旅行摄影)
   - **By Time Interval**: 按拍摄时间间隔分组 (如间隔 >2 小时的分为不同组)
   - **By Camera Model**: 按相机型号分组 (多机身拍摄时非常有用)
3. 选择维度后弹出配置对话框:
   - ISO Range: 可自定义各区间阈值
   - GPS Cluster: 设置聚类半径 (默认 500 米)
   - Time Interval: 设置最小分段时间间隔 (默认 2 小时)
   - Camera Model: 无额外参数，直接分组
4. 点击 **[Create Groups]** → 系统遍历所有图片并创建对应的分组。

### 5.4 编辑分组

1. 在 GroupTree 中将鼠标悬停在分组名称上。
2. 点击出现的 **✎ (编辑)** 按钮。
3. 编辑对话框允许修改:
   - **名称**: 重命名分组。
   - **匹配条件**: 修改 Field / Op / Value。
   - **默认覆盖参数**: 添加/删除/修改默认参数值。
4. 修改条件后，系统提示 "Condition changed. Re-evaluate membership?" → 选择 [Re-evaluate] 重新匹配图片。

### 5.5 删除分组

1. 悬停分组 → 点击 **🗑 (删除)**。
2. 弹出确认对话框: "Delete group 'High ISO'? The override parameters for 4 images will fall back to Template level."
3. 点击 [Delete] 确认删除。
4. 分组被删除，图片不再受该组的参数覆盖影响 — 参数回退到 Template 层或内置默认值。

### 5.6 分组与参数覆盖的关系

这是分组最重要的用途。在典型的处理工作流中:

1. **Template 层**定义了管线的通用默认参数 (如 AI 降噪强度=0.5)。
2. **Group 层**为不同图片集合提供了差异化参数:
   - "High ISO 组": AI 降噪强度=0.8 (ISO>=1600 的图片需要更强的降噪)
   - "Landscape 组": 锐化强度=0.3 (风光片不需要过高锐度)
   - "Portrait 组": 色彩饱和度=0.9 (人像片需要更饱和的色彩)
3. **Image 层**可以对单张图片进行微调 (仅对个别异常图片使用)。

参数的最终有效值 = Image层覆盖 OR Group层覆盖 OR Template层定义 OR 插件内置默认值 (取第一个存在的)。

---

## 6. DAG 管线构建

### 6.1 DAG 基础概念

**DAG** (Directed Acyclic Graph, 有向无环图) 是 Photopipeline 的核心管线模型。与传统的线性流水线 (A→B→C→D) 不同，DAG 支持:

- **分支处理**: 一个节点的输出可以连接到多个下游节点。
- **合并处理**: 一个节点可以接收多个上游节点的输入 (如果类型兼容)。
- **选择性路径**: 根据不同条件选择不同的处理路径。
- **不可循环**: 禁止 A→B→C→A 这样的循环连接, 系统自动检测并拒绝。

### 6.2 DAGCanvas 画布操作

#### 视图操作

| 操作 | 方式 | 效果 |
|------|------|------|
| **平移画布** | 按住 `Space` + 鼠标左键拖动, 或**鼠标中键拖动** | 移动视口, 查看画布的不同区域 |
| **缩放画布** | `Ctrl + 滚轮` (向前放大, 向后缩小) | 以鼠标位置为中心缩放, 范围 10% - 400% |
| **适应窗口** | 点击工具栏 [⊞ Fit] 按钮 | 自动调整缩放和位置, 使所有节点可见 |
| **缩放按钮** | 点击工具栏 [+] [−] 按钮 | 每次缩放 10% |
| **MiniMap 导航** | 右下角 MiniMap: 点击任意位置→跳转视口, 拖动蓝色矩形→平移视口 | 快速定位到画布特定区域 |

#### 画布视觉元素

| 元素 | 外观 | 说明 |
|------|------|------|
| **网格背景** | 32px 间距, rgba(255,255,255,0.015) | 辅助对齐, 提供空间感 |
| **节点** | 深色卡片, 圆角 8px, 投影 shadow4 | 每个节点代表一个处理插件 |
| **连线** | 品牌蓝色贝塞尔曲线, 2px, opacity 0.55 | 代表数据流方向 |
| **端口** | 14x14px 方块 | 绿色=输入端口, 蓝色=输出端口 |

### 6.3 节点 (DAGNode)

#### 节点视觉结构

```
┌──────────────────┐
│ auto · 12 files  │  ← 徽章 (可选, 仅 Auto 创建的节点)
│                  │
│ ai_denoise       │  ← 插件名 (fontWeight 600, 14px)
│ Enhance          │  ← 分类标签 (10px, neutralFg4 灰色)
│            ⊡     │  ← 输出端口 (蓝色方块, 右侧居中)
│     ⊡            │  ← 输入端口 (绿色方块, 左侧居中)
└──────────────────┘
```

#### 节点状态

| 状态 | 边框 | 阴影 | 触发 |
|------|------|------|------|
| **default** | neutralStroke2 (#383838), 1.5px | shadow4 | 未选中, 未悬停 |
| **hover** | neutralFg4 (#999999), 1.5px | shadow4 | 鼠标悬停节点区域 |
| **selected** | brandFg1 (#4793ff), 1.5px | 0 0 0 3px rgba(71,147,255,0.15) + shadow8 | 单击选中 |
| **disabled** | 灰色 + 半透明覆盖层 | 无 | 右键 → Disable |
| **executing** | 绿色闪烁边框 | shadow8 | 当前正在执行的节点 |
| **done** | 绿色常亮边框 | shadow4 | 执行完成 |

#### 端口

| 类型 | 形状 | 颜色 | 位置 | hover 行为 |
|------|------|------|------|-----------|
| **Input** | 14x14px 方块 | successFg 绿色边框 | 节点左侧垂直居中 | 放大 1.3x, 高亮 |
| **Output** | 14x14px 方块 | brandFg1 蓝色边框 | 节点右侧垂直居中 | 放大 1.3x, 高亮 |

### 6.4 添加节点的三种方式

#### 方式一: 拖放 (Drag & Drop)

**操作步骤**:

1. 在底部 PluginBrowser 中找到目标插件卡片。
2. 按住鼠标左键拖动卡片到 DAG 画布上。
3. 拖动时显示半透明的节点预览跟随光标。
4. 将预览放到目标位置, 释放鼠标。
5. 节点创建在该位置。

**优点**: 可精确控制节点位置。

#### 方式二: 双击 (Double Click)

**操作步骤**:

1. 在底部 PluginBrowser 中双击目标插件卡片。
2. 节点自动添加到 DAG 画布的**管线末尾** (最右侧节点的右侧 200px 处)。
3. 如果画布上还没有任何节点, 节点添加到画布中心。

**优点**: 最快, 不需要拖拽。

**限制**: 不适用于需要插入到管线中间的场景 (插入中间需要先添加再手动连线)。

#### 方式三: 右键菜单

**操作步骤**:

1. 在 DAG 画布的**空白区域**上右键单击。
2. 弹出上下文菜单: `Add Node →` 子菜单列出所有 14 个插件, 按分类分组。
3. 点击目标插件名称。
4. 节点创建在右键点击的位置。

**优点**: 不需要视线离开画布去底部找插件卡片, 适合构建复杂管线。

### 6.5 连线 (DAGEdge) 操作详解

#### 创建连线

1. 将鼠标悬停在源节点的**蓝色输出端口** ⊡ 上。
2. 端口会放大至 1.3 倍并高亮。
3. 按住鼠标左键开始拖动。
4. 拖动过程中一根**蓝色虚线**跟随鼠标光标移动。
5. 将光标移动到目标节点的**绿色输入端口** ⊡ 上方。
6. 当端口高亮 (接受连接) 时释放鼠标左键。
7. 创建一条蓝色的贝塞尔曲线连线。

#### 连线规则

| 规则 | 说明 |
|------|------|
| **端口匹配** | Output 端口 → Input 端口, 不能 Output→Output 或 Input→Input |
| **类型兼容** | 上游像素输出 (pixel_output) 必须连接到下游像素输入 (pixel_input) |
| **无循环** | A→B→C→A 这样的循环连接会被系统自动拒绝, 拖放时输入端口不亮 |
| **单输入多输出** | 一个节点可以有多个输入 (如果支持合并), 可以有多个输出 (分支) |
| **不能重连已占端口** | 一个输入端口如果已经有连线, 需要先删除旧连线再创建新连线 |

#### 常见连线错误

| 错误场景 | 现象 | 解决方案 |
|---------|------|---------|
| 尝试创建循环 | 目标端口不亮, 无法放下 | 重新规划管线结构 |
| 类型不兼容 | 目标端口显示红色, 提示 "Incompatible types" | 检查上下游插件的像素格式要求 |
| 输入端口已连接 | 端口已有蓝色连线 | 右键旧连线 → Delete Edge, 再创建新连线 |
| 连接 Metadata 插件到 Pixel 插件 | 端口不亮 (Metadata 插件不输出像素) | 确保数据流类型匹配 |

#### 连线交互

| 操作 | 方式 |
|------|------|
| **查看连线** | 悬停连线 → 加粗至 3px, opacity 变为 0.9 |
| **选中连线** | 单击连线 → 高亮 |
| **删除连线** | 右键选中连线 → "Delete Edge", 或选中后按 Delete |

### 6.6 节点操作

#### 节点右键菜单

```
┌─────────────────┐
│ Copy            │ → 复制节点参数到剪贴板
│ Duplicate  Ctrl+D│ → 创建同参数副本 (不含连线)
│ Disable         │ → 禁用节点 (变灰, 执行时跳过)
│ Enable          │ → 重新启用 (仅禁用时显示)
├─────────────────┤
│ Disconnect All  │ → 断开所有连线
│ Delete     Del  │ → 删除节点及相关连线
└─────────────────┘
```

#### 节点操作快捷键

| 操作 | 快捷键 | 说明 |
|------|:-----:|------|
| 选中节点 | 单击 | 右侧面板切换到该节点参数 |
| 移动节点 | 拖动 | 改变节点在画布上的位置 |
| 复制节点 | `Ctrl+D` | 创建副本 (参数一致, 无连线) |
| 删除节点 | `Delete` | 删除选中节点及所有关联连线 |
| 撤销 | `Ctrl+Z` | 撤销上一步操作 |
| 重做 | `Ctrl+Y` | 重做被撤销的操作 |
| 禁用/启用 | 右键菜单 | 切换节点启用状态 |

### 6.7 管线管理

#### 工具栏操作

```
┌──────────────────────────────────────────────────────────────┐
│ 📄New  💾Save  📂Load  │  ✓Validate  │  ▶Run  ⏹Cancel  │  🔍+  🔍−  ⊞Fit  │
└──────────────────────────────────────────────────────────────┘
```

| 按钮 | 功能 | 快捷键 | 说明 |
|------|------|:-----:|------|
| **New** | 新建管线 | — | 清空当前画布 (如果未保存, 弹出确认) |
| **Save** | 保存管线 | `Ctrl+S` | 保存为 .ppjson 文件 |
| **Load** | 加载管线 | — | 从 .ppjson 文件加载管线定义 |
| **Validate** | 验证管线 | — | 检查节点/连线/参数合法性 |
| **Run** | 执行管线 | `Ctrl+E` | 对当前图片执行管线处理 |
| **Cancel** | 取消执行 | — | 仅执行中可见 |
| **Zoom+** | 放大画布 | — | 每次 10% |
| **Zoom-** | 缩小画布 | — | 每次 10% |
| **Fit** | 适应窗口 | — | 自动缩放使所有节点可见 |

### 6.8 管线验证详情

点击 Validate 后, 系统通过 gRPC 调用 `PipelineService.Validate` 进行全面检查:

#### 检查项目

1. **插件存在性**: 所有节点引用的 plugin_id 是否在系统中注册。
2. **数据流兼容性**: 逐对检查上下游节点的像素格式是否兼容:
   - 原始 (raw) → 像素 (pixel) → 像素 (pixel) → ... → 编码器 (encoder)
   - Metadata 节点可以插入在兼容位置处理元数据
3. **参数合法性**: 每个参数的值是否在其定义的 min/max/pattern 范围内。
4. **管道完整性**: 确保有输入节点 (raw_input) 和输出节点 (encoder)。
5. **循环检测**: 检查图中是否存在环路。
6. **孤立节点检测**: 检测是否有未连接的节点 (警告, 不阻止执行)。

#### 验证结果消息

| 级别 | 图标 | 说明 | 阻止执行? |
|------|:---:|------|:---:|
| **INFO** | 蓝色 ⓘ | 提示信息, 如 "1 orphan node detected" | 否 |
| **WARNING** | 黄色 ⚠ | 潜在问题, 如 "Parameter exceeds typical range" | 否 |
| **ERROR** | 红色 ✕ | 严重错误, 如 "Output encoder missing" | **是** |

验证结果显示在弹出面板中，消息可展开查看详细信息。

### 6.9 管线文件格式

管线文件使用 **.ppjson 扩展名** (Photopipeline JSON)。这是标准 JSON 格式，可以:

- 用任何文本编辑器查看和编辑
- 纳入 Git 版本控制
- 在团队之间分享
- 在命令行中使用: `photopipeline run config.ppjson`

详细的 PipelineConfig JSON 格式参见[第 15 章](#15-pipelineconfig-json-参考)。

---

## 7. 编辑参数与覆盖系统

### 7.1 什么是覆盖系统

覆盖系统 (Override System) 是 Photopipeline 最强大的功能之一。它允许你在不修改管线模板的前提下，为不同分组或单张图片设置差异化的参数。这使得同一个管线可以同时处理多种不同场景的图片。

#### 使用场景举例

假设你拍摄了一次旅行，包含以下场景:
- 白天风光片: ISO 100-400, 几乎不需要降噪
- 傍晚黄金时刻: ISO 800-1600, 需要中等降噪
- 夜景长曝光: ISO 3200-6400, 需要强力降噪
- 航拍片: DJI 无人机拍摄, 需要不同的镜头校正

传统方法需要创建 4 个不同的管线。Photopipeline 的覆盖系统允许你用**一个管线 + 4 个分组**解决这个问题。

### 7.2 四级参数覆盖

参数的实际生效值通过四级优先级系统确定。优先级从高到低:

| 层级 | 优先级 | 说明 | 典型用途 |
|------|:---:|------|---------|
| **Image** (图片) | 3 (最高) | 单张图片的独立覆盖值 | 修复个别异常图片, 如某张严重欠曝的图片 |
| **Group** (分组) | 2 | 满足分组条件的图片的覆盖值 | 不同场景的差异化处理, 如 High ISO 组 |
| **Template** (模板) | 1 | 管线模板中定义的默认参数值 | 管线的通用默认设置 |
| **Plugin Builtin** (插件内置) | 0 (最低) | 插件源代码中的硬编码默认值 | 软件出厂默认值, 如 denoise_strength=50 |

#### 覆盖优先级示例

假设你有一个管线，其中 `ai_denoise.strength` 参数:

- **插件内置默认值**: 50
- **Template 层设置**: 60 (你打开管线后调整的默认值)
- **Group "High ISO" 设置**: 80 (ISO≥1600 的图片)
- **Image "DSC_0036" 设置**: 90 (这张图片特别需要强力降噪)

那么最终生效值为:
- DSC_0036: **90** (Image 层覆盖了 Group 层的 80)
- DSC_0042 (属于 High ISO 组): **80** (Group 层覆盖了 Template 层的 60)
- DSC_0050 (不属于任何组): **60** (Template 层覆盖了内置默认的 50)

### 7.3 ContextBar — 切换编辑层级

右侧面板顶部的标签栏是控制覆盖编辑层级的关键控件:

```
│ [All] [Template] [High ISO] [DSC_0034] │
```

| 标签 | 含义 | 编辑行为 |
|------|------|---------|
| **All** | 查看所有图片的合并结果 | **只读** — 参数以灰色显示，不可编辑。显示的是最终生效值 |
| **Template** | 编辑管线模板的默认参数值 | **可编辑** — 修改影响所有使用该模板且没有被 Group/Image 覆盖的图片 |
| **Group: xxx** | 编辑特定分组的覆盖值 | **可编辑** — 修改仅影响该分组内的图片 |
| **Image: xxx** | 编辑单张图片的覆盖值 | **可编辑** — 修改仅影响该单张图片 |

#### 层级切换操作

1. **单击** ContextBar 中的标签切换编辑层级。
2. 当前激活的标签显示**品牌蓝色下划线** (2px 粗)。
3. 切换后右侧参数面板重新计算每个参数的覆盖状态标记。
4. 在 Template 层修改的值成为该管线的新默认值。
5. 在 Group 层修改的值仅在该图片满足该分组条件时生效。
6. 在 Image 层修改的值仅对该图片生效 — 这是最精确的覆盖。

#### All 标签的用途

All 标签是一个**只读合并视图**。它显示当前选中图片的最终生效参数值 — 即经过四级优先级计算后的结果。这让你能够:

- 快速了解不同图片实际会使用什么参数值
- 检查覆盖是否按预期工作
- 发现意外的参数值差异

### 7.4 覆盖标记 (OverrideDot)

每个参数行的右侧有一个圆形标记，指示该参数的覆盖状态:

| 标记 | 名称 | 含义 | 显示条件 |
|:---:|------|------|---------|
| **⬜** | 继承 (Inherited) | 当前层级未覆盖该参数, 值来自上级 | 在 Group/Image 层, 该参数使用 Template 或上级的值 |
| **🟡** | 覆盖 (Overridden) | 当前层级显式修改了该参数 | 在当前编辑层级, 该参数被修改过 |
| **🔵** | 表达式 (Expression) | 参数值由动态公式计算 | 参数使用了表达式 (如 `clamp(iso/12800, 0, 1)`) |

#### 覆盖标记的交互

| 当前标记 | 用户操作 | 结果 |
|:---:|------|------|
| ⬜ (继承) | 点击标记 | 激活编辑 → 标记变为 🟡, 你可以在当前层级修改该参数值 |
| ⬜ (继承) | 直接修改控件值 | 自动激活覆盖 → 标记变为 🟡 |
| 🟡 (覆盖) | 悬停标记 | 显示 **×** 恢复按钮 + 灰色提示 "Click to revert to inherited value" |
| 🟡 (覆盖) | 点击 × | 恢复继承 → 标记变为 ⬜, 参数值回退到上级定义的值 |
| 🔵 (表达式) | 双击标记 | 打开表达式编辑器 (仅支持表达式的参数) |

### 7.5 分区覆盖徽章

每个参数分区 (如 "Resize", "Strength", "Quality") 的标题右侧显示一个徽章:

| 徽章 | 颜色 | 含义 |
|------|------|------|
| **inherited** | neutralFg3 灰色 | 该分区全部参数均继承, 无本地覆盖 |
| **N overrides** | warningFg 黄色 | 有 N 个参数在当前层级被覆盖 |
| **values vary** | warningFg 黄色 | 多选图片时, 不同图片该参数的值不一致 |

### 7.6 表达式编辑器

表达式编辑器允许你使用动态公式计算参数值, 而不是使用固定的常量。这使得参数可以随图片的 EXIF 信息自动调整。

#### 开启表达式编辑器

1. 确保当前选中的节点支持表达式 (`ParameterField.supports_expression == true`)。
2. 在参数行上双击，或点击参数旁的 🔵 标记。
3. 表达式编辑器在参数面板底部展开:

```
┌─ Expression ──────────────────────────────┐
│ clamp(iso / 12800, 0, 1)                   │
│                                            │
│ DSC_0034: ISO 6400  → 0.50                 │
│ night_012: ISO 3200 → 0.25                 │
│ city_001:  ISO 100  → 0.01                 │
└────────────────────────────────────────────┘
```

#### 可用变量

| 变量名 | 类型 | 说明 | 示例值 |
|--------|------|------|--------|
| `iso` | number | ISO 感光度 | 6400, 100, 3200 |
| `aperture` | number | 光圈 f 值 | 2.8, 8.0, 16.0 |
| `shutter` | number | 快门速度 (秒) | 0.001 (1/1000), 30 (30s) |
| `focal_length` | number | 焦距 (mm) | 24, 50, 200 |
| `ev` | number | 曝光补偿 (EV) | -2.0, 0, +1.3 |
| `filename` | string | 文件名 | "DSC_0034.ARW" |

#### 可用函数

| 函数 | 说明 | 示例 |
|------|------|------|
| `clamp(val, min, max)` | 将值限制在 [min, max] 范围内 | `clamp(iso/12800, 0, 1)` |
| `min(a, b)` | 取较小值 | `min(iso, 6400)` |
| `max(a, b)` | 取较大值 | `max(iso, 100)` |
| `abs(x)` | 绝对值 | `abs(ev)` |
| `lerp(a, b, t)` | 线性插值 | `lerp(0.2, 1.0, iso/12800)` |
| `round(x)` | 四舍五入 | `round(iso/100)*100` |

#### 表达式实时预览

当你输入表达式后，编辑器底部显示**实时预览**部分:
- 列出当前选中的每张图片
- 显示各变量的实际值
- 显示表达式的计算结果
- 结果与参数的合法范围比较 (超出范围标红)

### 7.7 多选编辑

当你在胶片条中选中多张图片时 (ContextBar 显示 Image 标签):

#### 场景一: 值一致的参数

- 显示 ⬜ (继承) 标记。
- 参数值显示为这些图片的共同值。
- 如果该参数来自上级 (Template/Group)，所有图片保持一致。
- 编辑该参数 → 为所有选中图片统一设置覆盖值。

#### 场景二: 值不一致的参数

- 显示 🟡 标记 + "values vary" 黄色提示。
- 控件显示为空或占位符 "Multiple values"。
- 提供两个批量操作:

| 操作 | 按钮 | 行为 |
|------|------|------|
| **Unify to value...** | 统一设置 | 将所有选中图片的该参数统一设置为新值 |
| **Clear overrides** | 清除覆盖 | 移除所有选中图片的该参数覆盖, 回退到上级值 |

### 7.8 覆盖系统完整交互流程

以下是覆盖编辑的完整流程示例:

**场景**: 你有一个 "HDR Pipeline" 管线，导入了 20 张旅行照。

**步骤 1 — 设置管线默认值 (Template 层)**:

1. 在 ContextBar 点击 **[Template]** 标签。
2. 选中 ai_denoise 节点。
3. 将 `denoise_strength` 设为 50 → 标记变为 🟡 (Template 层覆盖了插件内置默认)。
4. 选中 heif_encoder 节点。
5. 将 `quality` 设为 90 → 这是整个管线的默认输出质量。

**步骤 2 — 为高 ISO 图片设置分组覆盖**:

1. 通过 Auto-group → By ISO Range 创建 "High ISO" 分组 (ISO≥1600)。
2. 在 ContextBar 点击 **[High ISO]** 标签。
3. 选中 ai_denoise 节点。
4. `denoise_strength` 显示 ⬜ (继承自 Template=50)。
5. 点击 ⬜ → 激活编辑 → 设为 85 → 标记变为 🟡 (Group 层覆盖)。
6. 现在 "High ISO" 组的 6 张图片使用降噪强度=85，其他图片使用 Template=50。

**步骤 3 — 为一张异常图片单独调整**:

1. 在胶片条中单击 "night_bridge_012.NEF" → 单张选中。
2. 在 ContextBar 点击 **[night_bridge_012]** 标签。
3. 选中 ai_denoise 节点。
4. `denoise_strength` 显示为 85, 但标记是 ⬜, 旁边灰色小字 "Inherited from Group: High ISO"。
5. 点击 ⬜ → 设为 95 → 标记变为 🟡 (Image 层覆盖)。
6. 现在这张图片使用 95, 其他 High ISO 图片仍使用 85, 普通图片使用 50。

---

## 8. 插件参考

### 8.1 插件分类总览

Photopipeline 内置 14 个插件，分为 6 个功能类别:

| 分类 | 标识色 | 插件列表 | 功能领域 |
|------|:---:|------|------|
| **Input** | #ef4444 红 | raw_input | 解码 RAW 相机文件 |
| **Transform** | #06b6d4 青 | transform | 缩放/旋转/裁剪/滤镜 |
| **Color** | #8b5cf6 紫 / #ec4899 粉 | colorspace, lut3d | 色彩空间转换, LUT 色彩分级 |
| **Correct** | #6366f1 靛蓝 | lens_correct | 镜头畸变/色差/暗角校正 |
| **Enhance** | #a855f7 紫罗兰 | ai_denoise | AI 降噪 (ONNX Runtime) |
| **Metadata** | #3b82f6 蓝 / #10b981 绿 / #f59e0b 琥珀 | exif_rw, gps_set, time_shift | 元数据编辑, GPS, 时间调整 |
| **Format** | 各编码器独有色 | heif_encoder, jxl_encoder, avif_encoder, tiff_encoder, png_encoder | 图像编码输出 |

### 8.2 插件 #01: raw_input — RAW 输入

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | RAW Input |
| 插件 ID | `photopipeline.plugins.raw_input` |
| 版本 | 1.0.0 |
| 分类 | Input |
| 图标 | 📷 camera |
| 标识色 | #ef4444 (红) |
| 能力 | FormatProcessor (无像素输入, 有像素输出) |
| 最低内存 | 512 MB |
| 描述 | 解码 RAW 相机文件, 支持 ARW/CR2/CR3/NEF/DNG/RAF/ORF/RW2/PEF 格式 |

#### 参数表

##### 分区: RAW Format (Card, 展开)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `raw_mode` | Enum (4选项) | Dropdown | `"auto"` | RAW 解码引擎选择 | 一般保持 auto; 特殊需求用 dcraw; 需要高级色彩科学用 LibRaw; HDR 工作流用 RawTherapee |

**枚举选项详情**:

| 值 | 标签 | 说明 | 推荐场景 |
|------|------|------|---------|
| `"auto"` ★ | Auto | 自动从文件中检测最佳解码方式 | 日常使用, 默认推荐 |
| `"dcraw"` | dcraw | 使用 dcraw 进行 RAW 转换 | 需要特定 dcraw 参数 |
| `"libraw"` | LibRaw | 使用 LibRaw FFI (如果可用) | 需要高级 RAW 处理选项 |
| `"rawtherapee"` | RawTherapee | 使用 RawTherapee CLI | 批量 HDR 管线 |

##### 分区: Output (Card, 展开)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `output_format` | Enum (2选项) | Dropdown | `"u16"` | 解码输出像素格式 | 标准处理用 16-bit; HDR 工作流用 32-bit float |
| `half_size` | Boolean | Switch | `false` | 半分辨率解码 (加速预览) | 快速预览用 Half; 最终输出用 Full |
| `apply_white_balance` | Boolean | Switch | `true` | 解码时应用相机白平衡 | 需要保留原始白平衡信息时选 As-Shot; 一般保持 Apply |

**枚举选项详情**:

| 值 | 标签 | 说明 |
|------|------|------|
| `"u16"` ★ | 16-bit Integer | 标准 16 位无符号整数, 0-65535 范围 |
| `"f32"` | 32-bit Float | 32 位浮点, 适合 HDR 处理和极端动态范围 |

##### 分区: dcraw Options (CollapsibleCard, 默认折叠, 高级)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `dcraw_path` | String | Input | `"dcraw"` | dcraw 二进制文件路径 | 仅在使用自定义 dcraw 构建时需要修改 |
| `dcraw_extra_args` | String | Input | `""` | 额外 dcraw 命令行参数 | 高级用户: 如 `-H 2` 使用特定高光模式 |

#### 推荐用法

- **日常快速处理**: 全部保持默认 (auto + u16 + Full + Apply WB)。
- **预览工作流**: half_size=true, 大幅加速加载。
- **HDR 专业工作流**: output_format=f32 + apply_white_balance=false (保留所有原始数据)。
- **自定义 RAW 处理**: raw_mode=dcraw + dcraw_extra_args="-H 2 -q 3" (dcraw 高光模式+高质量插值)。

### 8.3 插件 #02: transform — 变换处理

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | Transform |
| 插件 ID | `photopipeline.plugins.transform` |
| 版本 | 1.0.0 |
| 分类 | Transform |
| 图标 | ↔ maximize |
| 标识色 | #06b6d4 (青) |
| 能力 | PixelProcessor (像素输入+输出) |
| 最低内存 | 256 MB |
| 预览 | BeforeAfter 分屏对比 |
| 辅助视图 | Histogram (直方图) |
| 描述 | 缩放、旋转、裁剪图像, 支持多种重采样滤镜 |

#### 参数表

##### 分区: Resize (Card, 展开, 5 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `resize_mode` | Enum (6选项) | Dropdown | `"none"` | — | 缩放模式选择 | 输出到固定长边用 Long Edge; 精确控制用 Absolute |
| `target_width` | Integer | SpinButton | 1920 | 1-65535 px | 目标宽度 (px) | 仅 Absolute 模式生效 |
| `target_height` | Integer | SpinButton | 1080 | 1-65535 px | 目标高度 (px) | 仅 Absolute 模式生效 |
| `scale_percent` | Float | Slider + % | 100.0 | 1-1000% | 缩放百分比 | 仅 Percentage 模式生效 |
| `long_edge_px` | Integer | SpinButton | 2048 | 1-65535 px | 长边目标像素 | 仅 Long Edge/Short Edge 模式生效 |

**resize_mode 枚举选项详情**:

| 值 | 标签 | 说明 | 示例 |
|------|------|------|------|
| `"none"` | None | 不缩放 | 保持原始分辨率 |
| `"long_edge"` ★ | Long Edge | 限制长边像素, 保持宽高比 | 6000x4000 → 2048x1365 (长边 2048) |
| `"short_edge"` | Short Edge | 限制短边像素, 保持宽高比 | 6000x4000 → 3072x2048 (短边 2048) |
| `"absolute"` | Absolute | 精确指定宽高 | 6000x4000 → 1920x1080 |
| `"percentage"` | Percentage | 按百分比缩放 | 50%: 6000x4000 → 3000x2000 |
| `"megapixels"` | Megapixels | 限制总像素数 | 24MP → 缩放至 ~24MP |

##### 分区: Rotation (Card, 展开, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `angle` | Float | Slider + ° | 0.0 | -360° ~ +360°, step=0.1° | 顺时针旋转角度 | 水平校正用 ±0.1° - ±3°; 艺术旋转用 90°/180° |
| `flip_horizontal` | Boolean | Switch | false | Normal/Flipped | 水平翻转 | 镜像效果 |
| `flip_vertical` | Boolean | Switch | false | Normal/Flipped | 垂直翻转 | 倒影效果 |

##### 分区: Crop (Card, 展开, 5 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `crop_enabled` | Boolean | Switch | false | Disabled/Enabled | 启用裁剪 | 先启用再设置裁剪区域 |
| `crop_x` | Integer | SpinButton | 0 | 0-65535 px | 裁剪左边界 X 坐标 | 从图像左边缘算起的像素数 |
| `crop_y` | Integer | SpinButton | 0 | 0-65535 px | 裁剪上边界 Y 坐标 | 从图像上边缘算起的像素数 |
| `crop_width` | Integer | SpinButton | 1920 | 1-65535 px | 裁剪宽度 | 裁剪区域的像素宽度 |
| `crop_height` | Integer | SpinButton | 1080 | 1-65535 px | 裁剪高度 | 裁剪区域的像素高度 |

##### 分区: Filter (CollapsibleCard, 默认折叠, 高级, 1 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `filter_type` | Enum (3选项) | Dropdown | `"bilinear"` | 重采样滤镜 | 缩小用 Lanczos3 (最高质量); 放大用 Bilinear (平滑过渡); 像素艺术用 Nearest |

**filter_type 枚举选项详情**:

| 值 | 标签 | 说明 | 推荐场景 |
|------|------|------|---------|
| `"bilinear"` | Bilinear | 双线性插值, 计算成本低 | 快速预览 |
| `"lanczos3"` ★ | Lanczos3 (Halide) | Lanczos 3-lobed, Halide 优化, 最高质量 | 最终输出缩小, 推荐默认 |
| `"nearest"` | Nearest Neighbor | 最近邻插值, 无模糊 | 像素艺术放大, 保留硬边 |

#### 推荐用法

- **输出到社交媒体 (Instagram)**: resize_mode=Long Edge + long_edge_px=1080 + filter_type=Lanczos3。
- **打印准备 (A3 300dpi)**: resize_mode=Long Edge + long_edge_px=4961 (A3 长边) + filter_type=Lanczos3。
- **预览缩略图**: resize_mode=Percentage + scale_percent=25 + filter_type=Bilinear + half_size=true。
- **水平校正**: angle=1.5° (如果地平线微倾)。
- **二次构图**: crop_enabled=true + crop_x/y/w/h (去除边缘干扰元素)。

### 8.4 插件 #03: colorspace — 色彩空间转换

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | Color Space |
| 插件 ID | `photopipeline.plugins.colorspace` |
| 版本 | 1.0.0 |
| 分类 | Color |
| 图标 | 🎨 palette |
| 标识色 | #8b5cf6 (紫) |
| 能力 | PixelProcessor |
| 最低内存 | 256 MB |
| 预览 | BeforeAfter (lock_zoom: true) |
| 辅助视图 | Histogram + GamutDiagram (色域图) |
| 描述 | 色彩空间转换, 支持 ICC Profile 和渲染意图 |

#### 参数表

##### 分区: Color Space Conversion (Card, 展开, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `source_color_space` | Enum (8选项) | Dropdown | `"auto"` | 源色彩空间 | 一般保持 auto (自动检测嵌入的 ICC Profile) |
| `target_color_space` | Enum (6选项) | Dropdown | `"srgb"` | 目标色彩空间 | Web 输出=sRGB; Apple 设备=Display P3; HDR=BT.2020 PQ |

**source_color_space 枚举选项详情**:

| 值 | 标签 | 描述 | 特殊标记 |
|------|------|------|:---:|
| `"auto"` ★ | Auto-detect | 从嵌入的 ICC Profile 自动检测 | — |
| `"srgb"` | sRGB | 标准 sRGB IEC61966-2.1 | — |
| `"display_p3"` | Display P3 | 广色域 P3 D65 | — |
| `"adobe_rgb"` | Adobe RGB | Adobe RGB (1998) | — |
| `"pro_photo"` | ProPhoto RGB | Kodak ProPhoto RGB | — |
| `"bt2020"` | BT.2020 | Rec. 2020 UHDTV | [hdr] |
| `"aces_cg"` | ACEScg | ACES CG linear | [cinema] |
| `"linear_srgb"` | Linear sRGB | Linear-light sRGB | — |

**target_color_space 枚举选项详情**:

| 值 | 标签 | 描述 | 特殊标记 |
|------|------|------|:---:|
| `"srgb"` ★ | sRGB | 标准 sRGB — 最佳兼容性 | — |
| `"display_p3"` | Display P3 | 广色域 P3 D65 — Apple 设备原生 | — |
| `"adobe_rgb"` | Adobe RGB | Adobe RGB (1998) — 打印工作流 | — |
| `"pro_photo"` | ProPhoto RGB | Kodak ProPhoto — 最大色域存档 | — |
| `"bt2020_pq"` | BT.2020 PQ (HDR) | Rec. 2020 + PQ 传输函数, HDR 1000 nits | [hdr] |
| `"linear_srgb"` | Linear sRGB | 线性光工作空间 — 用于合成 | — |

##### 分区: Rendering (Card, 展开, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `rendering_intent` | Enum (4选项) | Dropdown | `"relative_colorimetric"` | 渲染意图 | 照片用 Relative; 设计用 Perceptual; 标志用 Saturation |
| `black_point_compensation` | Boolean | Switch | true (On/Off) | 黑点补偿 | 从大色域到小色域建议开启 |
| `gamut_mapping` | Enum (3选项, 高级) | Dropdown | `"compress"` | 色域映射算法 | 保持默认 compress |

**rendering_intent 枚举选项详情**:

| 值 | 标签 | 说明 | 推荐场景 |
|------|------|------|---------|
| `"relative_colorimetric"` ★ | Relative Colorimetric | 裁剪超出色域的颜色, 保持白点 | 大多数照片处理, 推荐默认 |
| `"perceptual"` | Perceptual | 压缩色域, 保持颜色关系 | 从 ProPhoto 到 sRGB, 保留视觉层次 |
| `"saturation"` | Saturation | 优先保持饱和度 | 图表/标志/UI 元素 |
| `"absolute_colorimetric"` | Absolute Colorimetric | 保留精确颜色值, 裁剪 | 打样/校色/精确色彩复制 |

**gamut_mapping 枚举选项详情**:

| 值 | 标签 | 说明 |
|------|------|------|
| `"compress"` ★ | Compress | 平滑压缩进目标色域 (默认推荐) |
| `"clip"` | Clip | 硬裁剪超出目标色域的颜色 |
| `"luminance_preserve"` | Luminance Preserve | 保持亮度, 牺牲色度 |

##### 分区: ICC Profile (CollapsibleCard, 默认折叠, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `embed_icc` | Boolean | Switch | true (Embed/Skip) | 在输出中嵌入 ICC Profile | 建议始终 Embed |
| `icc_profile_path` | FilePath (*.icc,*.icm) | Input+📂 | `""` | 自定义 ICC Profile 路径 | 打印机专用 Profile, 显示器校正 Profile |

#### 推荐用法

- **通用 Web 输出**: source=auto + target=sRGB + intent=Relative Colorimetric + embed_icc=true。
- **Apple 生态输出**: source=auto + target=Display P3 + intent=Perceptual + embed_icc=true。
- **HDR 输出**: source=auto + target=BT.2020 PQ + intent=Relative Colorimetric + embed_icc=true。
- **打印工作流**: source=auto + target=Adobe RGB + intent=Absolute Colorimetric + icc_profile_path=/path/to/printer_profile.icc。
- **视频后期素材**: source=auto + target=ACEScg 或 BT.2020 PQ。

### 8.5 插件 #04: lut3d — 3D LUT 色彩分级

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | 3D LUT |
| 插件 ID | `photopipeline.plugins.lut3d` |
| 版本 | 1.0.0 |
| 分类 | Color |
| 图标 | 3x3 grid |
| 标识色 | #ec4899 (粉) |
| 能力 | PixelProcessor |
| 最低内存 | 256 MB |
| GPU | 推荐 |
| 预览 | BeforeAfter (lock_zoom: true) |
| 辅助视图 | Histogram + Vectorscope (矢量示波器) |
| 描述 | 应用 3D LUT 进行色彩分级和胶片模拟 |

#### 参数表

##### 分区: LUT File (Card, 展开, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `lut_path` | FilePath (*.cube,*.3dl,*.look) | Input+📂 | `""` | LUT 文件路径 | 常用 LUT 放在统一目录方便切换 |
| `lut_format` | Enum (4选项) | Dropdown | `"cube"` | LUT 文件格式 | 一般自动检测; 如加载失败手动指定 |

**lut_format 枚举选项详情**:

| 值 | 标签 | 说明 |
|------|------|------|
| `"cube"` | .cube | DaVinci Resolve / Adobe 标准格式 |
| `"3dl"` | .3dl | Autodesk / Nuke 格式 |
| `"look"` | .look | 自定义 LUT 格式 |
| `"auto"` | Auto-detect | 从文件头自动检测格式 |

##### 分区: Transform (Card, 展开, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `intensity` | Slider (刻度) | Slider + 刻度 | 100.0 | 0-100%, step=1, ticks[0,25,50,75,100] | LUT 应用强度 | 100%=完全, 50%=半强度混合, 微调用 90-95% |
| `input_color_space` | Enum (4选项, 高级) | Dropdown | `"srgb"` | — | LUT 解释的色彩空间 | 与 LUT 设计者提供的信息一致 |
| `clamp_output` | Boolean | Switch | true (Clamp/Pass Through) | — | 是否将输出裁剪到 [0,1] 范围 | 一般保持 Clamp; HDR 特殊需求选 Pass Through |

##### 分区: Interpolation (CollapsibleCard, 默认折叠, 1 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 |
|--------|------|:---:|--------|------|
| `interpolation_method` | Enum (2选项, 高级) | Dropdown | `"tetrahedral"` | LUT 插值方法 |

| 值 | 标签 | 说明 |
|------|------|------|
| `"tetrahedral"` ★ | Tetrahedral | 四面体插值, 精度更高 |
| `"trilinear"` | Trilinear | 三线性插值, 速度更快 |

#### 推荐用法

- **经典胶片模拟**: 加载 Kodak Portra 400 / Fujifilm Velvia .cube LUT + intensity=80-95%。
- **色彩风格统一**: 同一组照片加载相同 LUT + intensity=100%。
- **微调色彩倾向**: intensity=20-40%, 仅做轻微调整。
- **电影级色彩分级**: 加载 ACES 到 Rec.709 LUT + input_color_space=aces_cg + intensity=100%。

### 8.6 插件 #05: lens_correct — 镜头校正

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | Lens Correction |
| 插件 ID | `photopipeline.plugins.lens_correct` |
| 版本 | 1.0.0 |
| 分类 | Correct |
| 图标 | aperture |
| 标识色 | #6366f1 (靛蓝) |
| 能力 | PixelProcessor |
| 最低内存 | 256 MB |
| 预览 | BeforeAfter (lock_zoom: true) |
| 辅助视图 | Histogram + StatusText (镜头检测信息) |
| 描述 | 使用 LensFun 库校正镜头畸变、色差和暗角 |

#### 参数表

##### 分区: Lens Detection (Card, 展开, 1 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `correction_mode` | Enum (3选项) | Dropdown | `"auto"` | 镜头检测/校正模式 | 一般保持 auto; 特定需求用 manual; 测试对比用 off |

| 值 | 标签 | 说明 |
|------|------|------|
| `"auto"` ★ | Auto-detect from EXIF | 从图片 EXIF 自动检测相机和镜头型号 |
| `"manual"` | Manual | 手动指定相机和镜头型号 |
| `"off"` | Off | 关闭镜头校正 |

##### 分区: Corrections (Card, 展开, 4 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `correct_distortion` | Boolean | Switch | true (Correct/Skip) | 校正桶形/枕形畸变 | 广角镜头和长焦镜头建议开启 |
| `correct_tca` | Boolean | Switch | true (Correct/Skip) | 校正横向色差 (紫边/绿边) | 高对比度边缘非常明显, 建议开启 |
| `correct_vignetting` | Boolean | Switch | true (Correct/Skip) | 校正暗角 | 大光圈拍摄建议开启; 有时保留暗角可作为艺术效果 |
| `correct_geometry` | Boolean (高级) | Switch | false (Correct/Skip) | 校正透视几何变形 | 仅 Ultra-Wide (超广角) 需要, 如 12-16mm |

##### 分区: LensFun (CollapsibleCard, 默认折叠, 高级, 4 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 |
|--------|------|:---:|--------|------|
| `lensfun_db_path` | FilePath (目录) | Input+📂 | `/usr/share/lensfun` | LensFun XML 数据库目录 |
| `camera_make` | String (max=128) | Input | `""` | 相机制造商 (仅 manual 模式) |
| `camera_model` | String (max=128) | Input | `""` | 相机型号 (仅 manual 模式) |
| `lens_model` | String (max=256) | Input | `""` | 镜头型号 (仅 manual 模式) |

#### 推荐用法

- **标准 RAW 处理**: correction_mode=auto + 前三个校正全开 (distortion/TCA/vignetting)。
- **大光圈人像 (f/1.4)**: 保留暗角作为艺术效果: correct_vignetting=false。
- **广角风光 (16-35mm)**: distortion + TCA + vignetting 全开, 考虑 correct_geometry。
- **LensFun 数据库更新**: 下载最新的 lensfun XML 文件 → 设置 lensfun_db_path 指向该目录。

### 8.7 插件 #06: ai_denoise — AI 降噪

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | AI Denoise |
| 插件 ID | `photopipeline.plugins.ai_denoise` |
| 版本 | 1.0.0 |
| 分类 | Enhance |
| 图标 | sparkles |
| 标识色 | #a855f7 (紫罗兰) |
| 能力 | PixelProcessor + AiProcessor |
| 最低内存 | 2048 MB |
| GPU | 推荐 (CUDA / CoreML / TensorRT 加速) |
| 预览 | BeforeAfter (lock_zoom: true) |
| 辅助视图 | Histogram + ProgressBar + StatusText |
| 描述 | 基于 ONNX Runtime 的 AI 图像降噪 |

#### 参数表

##### 分区: Model (Card, 展开, 1 个参数 + 动态 ModelInfo 卡片)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `denoise_model` | Enum (4选项) | Dropdown | `"standard_v2"` | AI 降噪模型选择 | 日常用 Standard v2; 速度优先用 Lightweight; 质量优先用 High Quality |

| 值 | 标签 | 说明 | 适用场景 |
|------|------|------|---------|
| `"lightweight_v1"` ★ | Lightweight v1 (fast) | 轻量级模型, 速度最快 | 批量快速处理, ISO<1600 |
| `"standard_v2"` | Standard v2 (balanced) | 标准模型, 平衡速度和质量 | 日常使用, ISO 100-12800 |
| `"high_quality_v2"` | High Quality v2 (quality, slow) | 高质量模型, 最佳降噪效果 | 高 ISO (>6400) 单张精细处理 |
| `"raw_denoise_v1"` | RAW Denoise v1 (raw) | RAW 域降噪专用模型 | 在 demosaic 之前进行降噪 |

**模型选中后显示的 ModelInfo 卡片**包含模型名称、版本、输入尺寸要求、VRAM 需求等信息，帮助用户评估硬件需求。

##### 分区: Strength (Card, 展开, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `denoise_strength` | Slider (刻度) | Slider + 刻度 | 50 | 0-100, ticks[0,25,50,75,100] | 降噪强度 | ISO<400→20-30; ISO400-1600→40-60; ISO>1600→70-90 |
| `detail_preservation` | Slider (刻度) | Slider + 刻度 | 50 | 0-100, ticks[0,25,50,75,100] | 细节保留程度 | 风光片→60-80 (保留纹理); 人像→30-50 (皮肤平滑优先) |
| `color_noise_reduction` | Slider (刻度, 高级) | Slider + 刻度 | 75 | 0-100, ticks[0,50,100] | 色彩噪点专门抑制 | 一般保持 75; 极端高 ISO 调至 90+ |

##### 分区: Hardware (CollapsibleCard, 默认折叠, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `ai_backend` | Enum (5选项) | Dropdown | `"onnx_cpu"` | AI 推理后端 | 有 NVIDIA GPU→CUDA; Apple Silicon→CoreML; Intel→OpenVINO |
| `tile_size` | Integer | SpinButton | 0 | 0-4096, step=64, 0=自动 | 分块处理大小 (px) | 显存不足时减小 (如 512); 默认 0=自动 |
| `use_fp16` | Boolean | Switch | true (FP16/FP32) | 使用半精度推理 | 大多数情况选 FP16 (更快); 精度敏感选 FP32 |

**ai_backend 枚举选项详情**:

| 值 | 标签 | 适用硬件 |
|------|------|---------|
| `"onnx_cpu"` ★ | ONNX CPU | 通用 CPU 推理 (无 GPU 时使用) |
| `"onnx_cuda"` | ONNX CUDA (gpu, cuda) | NVIDIA GPU, CUDA 12+ |
| `"tensorrt"` | TensorRT (gpu, nvidia) | NVIDIA GPU, TensorRT 优化 (最快) |
| `"coreml_ane"` | CoreML ANE (apple) | Apple Silicon M1/M2/M3 Neural Engine |
| `"openvino"` | OpenVINO (intel) | Intel CPU/GPU, OpenVINO 加速 |

#### 推荐用法

- **日常批量 (混合 ISO)**: Model=Standard v2 + Strength=50 + Detail=50 + 表达式 `clamp(iso/12800, 0.2, 0.9)` 实现自适应降噪。
- **高 ISO 夜景 (>3200)**: Model=High Quality v2 + Strength=80 + Detail=60 + Color=90 (重点抑制彩色噪点)。
- **低 ISO 风光 (<400)**: Model=Lightweight v1 + Strength=20 + Detail=80 (保留纹理, 仅轻微降噪)。
- **GPU 加速**: 有 NVIDIA GPU→ai_backend=CUDA; Apple Silicon→CoreML ANE; tile_size=1024 (匹配 GPU tile)。
- **CPU 处理**: ai_backend=CPU; tile_size 保持 0 (自动); 预计处理时间为 GPU 的 5-10 倍。

### 8.8 插件 #07: exif_rw — EXIF 元数据读写

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | EXIF Reader/Writer |
| 插件 ID | `photopipeline.plugins.exif_rw` |
| 版本 | 1.0.0 |
| 分类 | Metadata |
| 图标 | tag |
| 标识色 | #3b82f6 (蓝) |
| 能力 | MetadataProcessor (无像素访问) |
| 最低内存 | 128 MB |
| 预览 | 无 |
| 辅助视图 | 无 |
| 描述 | 通过 exiftool 读取和写入 EXIF/XMP/IPTC/GPS 元数据 |

#### EXIF 字段总览 (42+ 可编辑字段, 8 个分区)

##### 分区: Camera (5 个字段)

| 字段名 | 可编辑? | 说明 | 控件 |
|--------|:---:|------|:---:|
| Make | 只读 | 相机制造商 (如 SONY) | 灰色斜体文本 |
| Model | 只读 | 相机型号 (如 ILCE-7RM5) | 灰色斜体文本 |
| Lens | 只读 | 镜头型号 | 灰色斜体文本 |
| Serial Number | ✅ | 相机序列号 | Input |
| Firmware | ✅ | 相机固件版本 | Input |

##### 分区: Author & Copyright (5 个字段)

| 字段名 | 可编辑? | 说明 | 控件 |
|--------|:---:|------|:---:|
| Artist | ✅ | 作者 (EXIF:Artist, XMP:dc:creator) | Input |
| Copyright | ✅ | 版权信息 (EXIF:Copyright) | Input |
| Image Description | ✅ | 图片描述 (EXIF:ImageDescription) | Textarea |
| Rating | ✅ | 评分 0-5 星 | StarRating 组件 |
| Instructions | ✅ | 使用说明 | Input |

##### 分区: Date & Time (5 个字段)

| 字段名 | 可编辑? | 说明 | 控件 |
|--------|:---:|------|:---:|
| Date Taken | ✅ | 拍摄日期时间 | datetime-local 控件 |
| Digitized | 只读 | 数字化日期 | 灰色斜体文本 |
| Offset Time | ✅ | 时区偏移 (+08:00) | Input |
| SubSec | ✅ | 亚秒时间 | Input |
| Modified | 只读 | 最后修改时间 | 灰色斜体文本 |

##### 分区: Exposure (8 个字段)

| 字段名 | 可编辑? | 说明 | 控件 |
|--------|:---:|------|:---:|
| Shutter Speed | ✅ | 快门速度 (如 1/250) | Input |
| Aperture | ✅ | 光圈 (如 f/8.0) | Input |
| ISO | ✅ | ISO 感光度 | Input |
| Exposure Bias | ✅ | 曝光补偿 (EV) | Input |
| Metering Mode | ✅ | 测光模式 | Dropdown: Multi/Center/Spot |
| Flash | ✅ | 闪光灯 | Dropdown: Off/On/Auto |
| Exposure Program | ✅ | 曝光程序 | Dropdown: Aperture/Manual/Shutter/Auto |
| Scene Type | 只读 | 场景类型 | 灰色斜体文本 |

##### 分区: Lens (4 个字段)

| 字段名 | 可编辑? | 说明 | 控件 |
|--------|:---:|------|:---:|
| Focal Length | ✅ | 焦距 (mm) | Input |
| 35mm Equivalent | ✅ | 35mm 等效焦距 (mm) | Input |
| Max Aperture | ✅ | 最大光圈 (f/) | Input |
| Lens ID | 只读 | 镜头识别码 | 灰色斜体文本 |

##### 分区: GPS (5 个字段)

| 字段名 | 可编辑? | 说明 | 控件 |
|--------|:---:|------|:---:|
| Latitude | ✅ | 纬度 (十进制度) | Input |
| Longitude | ✅ | 经度 (十进制度) | Input |
| Altitude | ✅ | 海拔 (米) | Input |
| Reference | ✅ | 参考方向 | Dropdown: N/E, S/W |
| Direction | ✅ | 拍摄方向 (度) | Input |

##### 分区: Image (6 个字段)

| 字段名 | 可编辑? | 说明 | 控件 |
|--------|:---:|------|:---:|
| Orientation | ✅ | 旋转方向 | Dropdown: 1/3/6/8 |
| X Resolution | ✅ | 水平 DPI | Input |
| Y Resolution | ✅ | 垂直 DPI | Input |
| Resolution Unit | ✅ | 分辨率单位 | Dropdown: inches/cm |
| Color Space | ✅ | 色彩空间标签 | Dropdown: sRGB/AdobeRGB/Uncalibrated |
| Software | ✅ | 处理软件名称 | Input |

##### 分区: Keywords & Location (5 个字段)

| 字段名 | 可编辑? | 说明 | 控件 |
|--------|:---:|------|:---:|
| Keywords | ✅ | 关键词 (逗号分割) | Input |
| City | ✅ | 城市 (XMP:photoshop:City) | Input |
| Province/State | ✅ | 省份/州 | Input |
| Country | ✅ | 国家 (XMP:photoshop:Country) | Input |
| Country Code | ✅ | 国家代码 (ISO 3166) | Input |

#### 字段状态说明

| 状态 | 样式 | 含义 |
|:---:|------|------|
| 可编辑 | 正常 Input/Select | 用户可以修改 |
| 只读 (RO) | 灰色斜体 | 从源文件提取, 不可覆盖 |
| 已覆盖 🟡 | 黄色圆点标记 | 在当前层级被覆盖的值 |

#### 推荐用法

- **统一版权信息**: 在 Template 层设置 Artist/Copyright → 所有输出图片自动嵌入。
- **批量添加关键词**: 选中多张图片 → 在 Image 层批量设置 Keywords (逗号分割, 如 "旅行,四川,成都,风光")。
- **恢复拍摄时间**: 填写 Date Taken (如果 EXIF 时间因某些原因丢失)。
- **添加位置信息**: 先使用 gps_set 插件设置 GPS, 再在此处填写 City/Province/Country 文本信息。

### 8.9 插件 #08: gps_set — GPS 坐标设置

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | GPS Coordinate Manager |
| 插件 ID | `photopipeline.plugins.gps_set` |
| 版本 | 1.0.0 |
| 分类 | Metadata |
| 图标 | map-pin |
| 标识色 | #10b981 (绿) |
| 能力 | MetadataProcessor |
| 最低内存 | 64 MB |
| 预览 | 无 |
| 辅助视图 | Map (交互式地图 + 搜索) |
| 描述 | 设置 GPS 坐标 — 手动输入、地图选点或 GPX 轨迹插值 |

#### 参数表

##### 分区: GPS Source (Card, 展开, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `gps_mode` | Enum (3选项) | Dropdown | `"manual"` | GPS 数据来源模式 | 手动写死坐标用 Manual; 有 GPX 轨迹用 GPX Track; 清除 GPS 用 Clear |
| `gpx_file` | FilePath (*.gpx) | Input+📂 | `""` | GPX 轨迹文件 | 仅在 mode="gpx_track" 时显示 |

| 值 | 标签 | 说明 |
|------|------|------|
| `"manual"` ★ | Manual | 手动输入或地图选点设置坐标 |
| `"gpx_track"` | GPX Track | 从 GPX 轨迹文件根据时间戳自动插值 GPS |
| `"clear"` | Clear GPS | 清除图片的所有 GPS 数据 |

##### 地图选点器 (Map Picker 辅助视图)

地图选点器位于 Manual Coordinates 分区上方:

```
┌─ Map Picker ────────────────────────────┐
│ [Search location...       ] [Search] [🇨🇳Amap▾]│
│ Results for "Chengdu":                    │
│ 📍 Chengdu, Sichuan, CN — 30.5728, 104.0668 │
│ 📍 Chengdu Panda Base — 30.7386, 104.1390  │
│ 📍 Tianfu Square — 30.6570, 104.0660        │
│                                           │
│ ┌───────────────────────────────────────┐ │
│ │           🗺 Map Area                │ │
│ │              📍                      │ │
│ │        30.5728, 104.0668             │ │
│ │                          [+] [-]    │ │
│ └───────────────────────────────────────┘ │
│ Provider: Amap (China) | Click to place   │
└───────────────────────────────────────────┘
```

**地图供应商选择规则**:

| 区域 | 使用供应商 | 说明 |
|------|:---------:|------|
| 中国境内 | 高德地图 (Amap) / 百度地图 (Baidu) | 默认高德, 国内定位更精确 |
| 中国境外 | Google 地图 | 全球覆盖 |

系统根据搜索结果的坐标自动判断使用哪个供应商。

**地图交互流程**:
1. 在搜索框输入地名 (如 "成都宽窄巷子")。
2. 点击 Search 或按 Enter。
3. 搜索结果列表显示匹配的地点。
4. 点击搜索结果 → 地图定位到该地点 → lat/lon 自动填入 Manual Coordinates。
5. 也可以直接在地图上点击选点。
6. 使用地图右下角的 [+] [−] 缩放按钮调整视图。

##### 分区: Manual Coordinates (Card, 由地图选点器填充, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 精度 | 说明 |
|--------|------|:---:|--------|------|------|------|
| `latitude` | Float | SpinButton | 0.0 | -90° ~ +90° | 6 位小数 | 纬度 (蓝色边框=来自地图自动填充) |
| `longitude` | Float | SpinButton | 0.0 | -180° ~ +180° | 6 位小数 | 经度 (蓝色边框=来自地图自动填充) |
| `altitude` | Float | SpinButton | 0.0 | -500 ~ +9000 m | 1 位小数 | 海拔 (需手动输入, 地图 API 不返回) |

##### 分区: GPX Options (CollapsibleCard, 默认折叠, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 |
|--------|------|:---:|--------|------|------|
| `time_offset_seconds` | Integer | SpinButton | 0 | -86400 ~ +86400 s | GPX 时间偏移 (相机与 GPS 设备时间差) |
| `max_interpolation_gap` | Integer (高级) | SpinButton | 300 | 1-3600 s | 最大插值间隔 (超过此间隔不插值) |

#### 推荐用法

- **手动标定位置 (少量图片)**: gps_mode=Manual → 在地图搜索框中搜索地点 → 点击结果 → lat/lon 自动填入。
- **旅行摄影批量 GPS (有 GPX)**: gps_mode=GPX Track → 加载 .gpx 文件 → 设置 time_offset_seconds (如果相机时间与 GPS 设备不同步) → 系统自动按时间戳插值为每张图片分配 GPS。
- **清除隐私 GPS**: gps_mode=Clear → 移除所有 GPS 数据 (发布到社交媒体前使用)。
- **海拔补充**: 使用地图选点获取 2D 坐标后, 手动从其他来源 (如 Strava/GPX 的海拔数据) 填入 altitude。

### 8.10 插件 #09: time_shift — 时间偏移

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | Time Shift |
| 插件 ID | `photopipeline.plugins.time_shift` |
| 版本 | 1.0.0 |
| 分类 | Metadata |
| 图标 | clock |
| 标识色 | #f59e0b (琥珀) |
| 能力 | MetadataProcessor |
| 最低内存 | 64 MB |
| 预览 | 无 |
| 辅助视图 | 无 |
| 描述 | 调整 EXIF DateTimeOriginal 时间偏移和时区转换 |

#### 参数表

##### 分区: Time Adjustment (Card, 展开, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `shift_hours` | Integer | SpinButton | 0 | -23 ~ +23 h | 小时偏移 | 时区转换整小时部分 |
| `shift_minutes` | Integer | SpinButton | 0 | -59 ~ +59 min | 分钟偏移 | 半小时时区 (如印度 +5:30) |
| `shift_seconds` | Integer | SpinButton | 0 | -59 ~ +59 s | 秒偏移 | 微调相机时钟漂移 |

##### 实时预览 (动态显示)

参数变化时，预览区即时更新:

```
┌─ Preview ─────────────────────────────────────┐
│ 2025-03-15 18:42:30 → 2025-03-15 20:12:30     │
│ (+01:30:00)                                    │
└────────────────────────────────────────────────┘
```

如果有多张选中图片，每张都显示对应的预览行，对比原时间、新时间和偏移量。

##### 分区: Timezone (Card, 展开, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `source_timezone` | Enum (9选项) | Dropdown | `"UTC"` | 源时区 | 如果相机设为 UTC 时间就选 UTC; 否则选相机时区 |
| `target_timezone` | Enum (9选项) | Dropdown | `"local"` | 目标时区 | 照片拍摄地的实际时区 |

**时区换算预览**:

```
Result: 18:42 UTC → 02:42 CST (+08:00 TZ + 01:30 shift = 20:12 CST)
```

##### 分区: Batch Options (CollapsibleCard, 默认折叠, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `offset_mode` | Enum (3选项) | Dropdown | `"fixed"` | 批量偏移模式 | 多数情况用 fixed |
| `increment_seconds` | Integer | SpinButton | 60 | 增量秒数 (仅 incremental 模式) | 延时摄影排序用 |

| 值 | 标签 | 说明 |
|------|------|------|
| `"fixed"` | Fixed | 所有图片统一偏移 |
| `"incremental"` | Incremental | 每张图片递增偏移 (用于延时摄影排序) |
| `"filename"` | From Filename | 从文件名提取时间戳 |

#### 推荐用法

- **跨洲旅行时区修正**: 相机设为 UTC → source=UTC + target=Asia/Shanghai (+8:00) → shift=0。
- **相机时钟漂移修正**: 发现相机快了 3 分 15 秒 → shift_hours=0, shift_minutes=-3, shift_seconds=-15。
- **回退到当地时区**: 照片 EXIF 已经是 UTC, 想改为拍摄地时区 → source=UTC + target=local。
- **批量 AM/PM 修正**: 下午照片被标记为 AM → shift_hours=12, shift_minutes=0。
- **延时摄影顺序调整**: offset_mode=incremental + increment_seconds=60 → 第1张=0偏移, 第2张=+60s, 第3张=+120s...

### 8.11 插件 #10: heif_encoder — HEIF 编码器

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | HEIF Encoder |
| 插件 ID | `photopipeline.plugins.heif_encoder` |
| 版本 | 1.0.0 |
| 分类 | Format |
| 图标 | image |
| 标识色 | #14b8a6 (青绿) |
| 能力 | FormatProcessor |
| 最低内存 | 512 MB |
| 预览 | 无 |
| 辅助视图 | 无 |
| 描述 | 使用 libheif 原生 FFI 编码 HEIF/HEIC 10-bit 图像 |

#### 参数表

##### 分区: Quality (Card, 展开, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `quality` | Slider (刻度) | Slider + 刻度 | 95.0 | 0-100, ticks[0,25,50,75,100] | 编码质量 | 存档 95-100; 共享 85-90; Web 75-85 |
| `lossless` | Boolean | Switch | false (Lossless/Lossy) | 无损模式 | 存档/中间文件使用; 注意文件会很大 |
| `bit_depth` | Enum (2选项) | Dropdown | `"10"` | 位深度 | HDR 用 10-bit; 兼容性优先用 8-bit |

| 值 | 标签 | 说明 |
|------|------|------|
| `"10"` ★ | 10-bit (HDR) | 10 位深, 支持 HDR, 文件略大 |
| `"8"` | 8-bit | 8 位深, 更广泛的兼容性 |

##### 分区: Advanced (Card, 展开, 3 个参数, 均高级)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `chroma_subsampling` | Enum (3选项, 高级) | Dropdown | `"444"` | 色度子采样 | 照片=444 (最高质量); 视频素材=422; Web=420 (最小) |
| `encoder_effort` | Integer (高级) | SpinButton | 4 | 0-10, 编码器努力程度 | 快速预览=0-2; 日常=4; 最优压缩=8-10 |
| `tune` | Enum (4选项, 高级) | Dropdown | `"ssim"` | 编码器优化目标 | 照片=SSIM; 截图=PSNR; 胶片质感=Grain; 快速解码=FastDecode |

**chroma_subsampling 枚举**:

| 值 | 标签 | 说明 | 文件大小 |
|------|------|------|:---:|
| `"444"` ★ | 4:4:4 | 无色度子采样, 最高质量 | 最大 |
| `"422"` | 4:2:2 | 轻度子采样, 专业视频标准 | 中等 |
| `"420"` | 4:2:0 | 重度子采样, 最小文件 | 最小 |

**tune 枚举**:

| 值 | 标签 | 说明 |
|------|------|------|
| `"ssim"` ★ | SSIM | 优化结构相似度 (照片标准) |
| `"psnr"` | PSNR | 优化峰值信噪比 |
| `"grain"` | Grain | 保留胶片颗粒感 |
| `"fastdecode"` | Fast Decode | 优先解码速度 |

#### 推荐用法

- **最佳质量存档**: quality=100 + bit_depth=10 + chroma=444 + effort=8 + tune=ssim。
- **日常输出 (平衡)**: quality=90 + bit_depth=10 + chroma=444 + effort=4 + tune=ssim。
- **Web 发布**: quality=80 + bit_depth=8 + chroma=420 + effort=2 + tune=ssim。
- **HDR 照片**: quality=95 + bit_depth=10 + chroma=444 + tune=ssim。
- **中间文件 (作为后续处理输入)**: lossless=true + bit_depth=10 + chroma=444。

### 8.12 插件 #11: jxl_encoder — JPEG XL 编码器

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | JPEG XL Encoder |
| 插件 ID | `photopipeline.plugins.jxl_encoder` |
| 版本 | 1.0.0 |
| 分类 | Format |
| 图标 | file-image |
| 标识色 | #f97316 (橙) |
| 能力 | FormatProcessor |
| 最低内存 | 512 MB |
| 描述 | 使用 libjxl 原生 FFI 编码 JPEG XL 16-bit 图像 |

#### 参数表

##### 分区: Quality (Card, 展开, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `quality` | Slider (刻度) | Slider + 特殊刻度 | 90.0 | -1~100, ticks[-1,25,50,75,100] | 编码质量 | quality=-1=无损; 存档 90-100; Web 75-85 |
| `lossless` | Boolean | Switch | false (Lossless/Lossy) | 无损模式 | 如需无损, 可直接设 quality=-1 或 lossless=true |
| `bit_depth` | Enum (4选项) | Dropdown | `"16"` | 位深度 | JXL 支持到 16-bit |

| 值 | 标签 | 说明 |
|------|------|------|
| `"16"` ★ | 16-bit (HDR) | 最高位深, 适合 HDR 和专业归档 |
| `"12"` | 12-bit | 中高位深, 质量与大小平衡 |
| `"10"` | 10-bit | 标准 HDR 位深 |
| `"8"` | 8-bit | 兼容性最高 |

**quality=-1 特殊说明**: 当 quality 滑块拉到最左边 (-1) 时, 等同于 lossless=true。滑块首个刻度显示 "lossless"。

##### 分区: Advanced (Card, 展开, 2 个参数, 均高级)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `effort` | Integer (高级) | SpinButton | 7 | 1-9 | 编码器努力程度 | 最终输出=7-9; 预览=3-4; 越高压缩率越好但越慢 |
| `modular` | Boolean (高级) | Switch | false (VarDCT/Modular) | 编码模式 | 照片=VarDCT; 截图/图形=Modular |

#### jxl vs heif 对比

| 特性 | HEIF | JPEG XL |
|------|------|---------|
| Quality 范围 | 0-100 | -1 ~ 100 |
| 最大位深 | 10-bit | **16-bit** |
| 默认位深 | 10 | 16 |
| 有无损模式 | 有 (lossless flag) | 有 (quality=-1 或 lossless) |
| 独有参数 | Chroma 444/422/420, Tune (SSIM/PSNR) | Effort 1-9, Modular/VarDCT |
| 浏览器支持 | iOS/macOS 原生; Chrome 有限 | Chrome/Edge/Firefox 逐步支持 |
| 适用场景 | Apple 生态, 照片分享 | 归档, 高动态范围, 渐进加载 |

#### 推荐用法

- **高质量归档**: quality=95 + bit_depth=16 + effort=9 + modular=false。
- **无损归档**: quality=-1 (或 lossless=true) + bit_depth=16 + effort=9。
- **Web 渐进加载**: quality=85 + bit_depth=8 + effort=5。
- **HDR 专业工作流**: quality=90 + bit_depth=16 + effort=7。

### 8.13 插件 #12: avif_encoder — AVIF 编码器

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | AVIF Encoder |
| 插件 ID | `photopipeline.plugins.avif_encoder` |
| 版本 | 1.0.0 |
| 分类 | Format |
| 图标 | image |
| 标识色 | #22c55e (绿) |
| 能力 | FormatProcessor |
| 最低内存 | 512 MB |
| 描述 | 使用 ravif (纯 Rust AV1) 编码 AVIF 图像 |

#### 参数表

##### 分区: Quality (Card, 展开, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `quality` | Slider (刻度) | Slider + 刻度 | 85.0 | 0-100, ticks[0,25,50,75,100] | 编码质量 | AVIF 压缩率高, 85 已等于 HEIF 95 的视觉效果 |
| `speed` | Integer | SpinButton | 6 | 0-10 | 编码速度 (0=最慢/最佳压缩, 10=最快) | 批量=6-8; 单张精细=2-4; 快速预览=10 |

##### 分区: Format Options (Card, 展开, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `bit_depth` | Enum (3选项) | Dropdown | `"10"` | 位深度 | 标准 HDR=10; 极致质量=12 |
| `chroma_subsampling` | Enum (3选项, 高级) | Dropdown | `"444"` | 色度子采样 | 照片=444; Web=420 |

| bit_depth 值 | 标签 |
|------|------|
| `"10"` ★ | 10-bit |
| `"12"` | 12-bit [hdr] |
| `"8"` | 8-bit |

##### 分区: Advanced (CollapsibleCard, 默认折叠, 1 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 |
|--------|------|:---:|--------|------|
| `lossless` | Boolean (高级) | Switch | false (Lossless/Lossy) | 无损模式 |

#### 编码器对比总结

| 特性 | HEIF | JPEG XL | AVIF |
|------|------|------|------|
| Quality 范围 | 0-100 | -1~100 | 0-100 |
| 最大位深 | 10-bit | **16-bit** | 12-bit |
| 默认 quality | 95 | 90 | **85** |
| 独有参数 | Chroma, Effort, Tune | Effort, Modular | **Speed** |
| Lossless 位置 | Quality 分区 | Quality 分区 | **Advanced (折叠)** |
| 颜色 | #14b8a6 | #f97316 | #22c55e |
| Web 浏览器支持 | iOS/macOS 原生 | Chrome/Edge/Firefox | Chrome/Edge/Firefox/Widevine |
| 最佳适用场景 | Apple 生态 | 归档/HDR 工作流 | **Web 发布 (最佳压缩率)** |

#### 推荐用法

- **最佳压缩率 Web 发布**: quality=80 + speed=6 + bit_depth=10 + chroma=420 (极小文件)。
- **高质量 Web 发布**: quality=85 + speed=4 + bit_depth=10 + chroma=444。
- **平衡批量输出**: quality=85 + speed=8 + bit_depth=10 + chroma=444。
- **高质量归档**: quality=95 + speed=2 + bit_depth=12 + chroma=444。

### 8.14 插件 #13: tiff_encoder — TIFF 编码器

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | TIFF Encoder |
| 插件 ID | `photopipeline.plugins.tiff_encoder` |
| 版本 | 1.0.0 |
| 分类 | Format |
| 图标 | file |
| 标识色 | #64748b (灰蓝) |
| 能力 | FormatProcessor |
| 最低内存 | 256 MB |
| 描述 | 编码 TIFF 图像 (LZW/ZIP 压缩) |

#### 参数表

##### 分区: Encoding (Card, 展开, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `compression` | Enum (4选项) | Dropdown | `"none"` | 压缩算法 | 存档/交换文件=无压缩或LZW; 磁盘空间紧张=Deflate |
| `bigtiff` | Boolean | Switch | true (BigTIFF/Classic) | BigTIFF 格式 (支持>4GB) | 高分辨率照片选 BigTIFF; 兼容旧软件选 Classic |

| compression 值 | 标签 | 说明 |
|------|------|------|
| `"none"` ★ | None | 无压缩 (最快, 文件最大) |
| `"lzw"` | LZW | LZW 无损压缩 (广泛兼容) |
| `"deflate"` | Deflate | ZIP 无损压缩 (最佳压缩率) |
| `"packbits"` | PackBits | Apple 传统压缩格式 |

##### 分区: Metadata (Card, 展开, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `embed_icc` | Boolean | Switch | true (Embed/Skip) | 嵌入 ICC Profile | 始终 Embed |
| `pixel_format` | Enum (3选项) | Dropdown | `"u16"` | 像素格式 | 标准=u16; HDR=f32; 兼容性=u8 |

| pixel_format 值 | 标签 |
|------|------|
| `"u16"` ★ | 16-bit Integer |
| `"f32"` | 32-bit Float (HDR) |
| `"u8"` | 8-bit Integer |

#### 推荐用法

- **最广泛兼容的归档格式**: compression=LZW + bigtiff=true + pixel_format=u16 + embed_icc=true。
- **最大兼容性 (旧软件)**: compression=None + bigtiff=false (Classic) + pixel_format=u8。
- **最小文件归档**: compression=Deflate + bigtiff=true + pixel_format=u16。
- **HDR 中间文件**: compression=None + pixel_format=f32 + embed_icc=true (后续处理使用)。

TIFF 是最简洁的编码器: 仅 4 个参数, 2 个 Card, 无 Advanced 折叠区, 无 Slider — 因为 TIFF 是无损容器, 不需要质量滑块。

### 8.15 插件 #14: png_encoder — PNG 编码器

#### 插件信息

| 属性 | 值 |
|------|-----|
| 名称 | PNG Encoder |
| 插件 ID | `photopipeline.plugins.png_encoder` |
| 版本 | 1.0.0 |
| 分类 | Format |
| 图标 | image |
| 标识色 | #0ea5e9 (天蓝) |
| 能力 | FormatProcessor |
| 最低内存 | 128 MB (最轻量) |
| 描述 | 编码 PNG 图像, 支持多种色彩类型和压缩级别 |

#### 参数表

##### 分区: Encoding (Card, 展开, 2 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 范围 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|------|---------|
| `compression_level` | Integer | SpinButton | 6 | 0-9 | ZLIB 压缩级别 (0=store, 9=best) | 日常=6; 最大压缩=9; 速度优先=1-3 |
| `bit_depth` | Enum (2选项) | Dropdown | `"16"` | — | 位深度 | 高动态范围=16; 一般=8 |

| bit_depth 值 | 标签 |
|------|------|
| `"16"` ★ | 16-bit (HDR) |
| `"8"` | 8-bit |

##### 分区: Metadata (Card, 展开, 3 个参数)

| 参数名 | 类型 | 控件 | 默认值 | 说明 | 推荐用法 |
|--------|------|:---:|--------|------|---------|
| `embed_icc` | Boolean | Switch | true (Embed/Skip) | 嵌入 ICC Profile | 始终 Embed |
| `include_exif` | Boolean | Switch | false (Include/Skip) | 包含 EXIF 元数据 | 需要元数据时开启 (注意隐私: GPS 等) |
| `color_type` | Enum (4选项) | Dropdown | `"rgb"` | PNG 色彩类型 | 照片=RGB; 透明=RGBA; 黑白=Grayscale |

| color_type 值 | 标签 | 说明 |
|------|------|------|
| `"rgb"` ★ | RGB | 标准 RGB 彩色 |
| `"rgba"` | RGBA | RGB + Alpha 透明通道 |
| `"grayscale"` | Grayscale | 灰度图像 |
| `"gray_alpha"` | Gray + Alpha | 灰度 + Alpha 透明 |

#### 推荐用法

- **Web 兼容最佳**: compression_level=6 + bit_depth=8 + color_type=RGB。
- **透明背景输出**: compression_level=6 + bit_depth=8 + color_type=RGBA。
- **高质量归档 (HDR)**: compression_level=9 + bit_depth=16 + color_type=RGB + embed_icc=true。
- **黑白艺术输出**: compression_level=9 + color_type=Grayscale + bit_depth=16。
- **快速导出 (中间预览)**: compression_level=0 + bit_depth=8 (无压缩, 秒级完成)。

### 8.16 插件选择策略速查表

| 需求场景 | 推荐插件组合 |
|---------|------------|
| RAW → 高质量 Apple 照片 | raw_input → ai_denoise → colorspace (Display P3) → heif_encoder (10-bit, quality=95) |
| RAW → 长期归档 | raw_input → lens_correct → tiff_encoder (LZW, 16-bit) |
| RAW → Web 发布 (最小体积) | raw_input → transform(长边=2048) → colorspace(sRGB) → avif_encoder (quality=80, chroma=420) |
| RAW → 最佳质量存档 | raw_input → ai_denoise → lens_correct → jxl_encoder (16-bit, quality=95) |
| 已有照片 → 元数据编辑 | 仅需 exif_rw + gps_set + time_shift (不需要 encoder) |
| RAW → 电影级色彩分级 | raw_input → colorspace (ACEScg) → lut3d (ACES→Rec.709 LUT) → heif_encoder (10-bit) |
| 影片扫描 → 最高质量输出 | raw_input → ai_denoise → lens_correct → png_encoder (16-bit, compression=9)|
| 批量 → 自适应降噪 | raw_input → ai_denoise (表达式: clamp(iso/12800,0,1)) → colorspace (sRGB) → jxl_encoder |
| 旅行照 → 元数据+GPS | raw_input → time_shift → gps_set (GPX) → exif_rw → heif_encoder |

---

## 9. 执行管线

### 9.1 执行前检查清单

在点击 ▶ Run 之前, 务必确认以下项目:

1. **[ ] 输入节点**: DAG 中必须包含 raw_input 节点 (作为数据输入源)。
2. **[ ] 输出节点**: DAG 中必须包含 encoder 节点 (作为输出格式, 如 heif_encoder/jxl_encoder 等)。
3. **[ ] 连线完整性**: 所有节点必须通过连线形成从输入到输出的完整路径。检查是否有悬空端口。
4. **[ ] 参数合法性**: 检查每个节点的参数值是否在合法范围内。不确定时点击 Validate。
5. **[ ] 后端连接**: 状态栏右下角必须是绿色圆点 "Connected"。
6. **[ ] 输出目录**: 确保输出目录存在且有写入权限。

### 9.2 执行流程详解

1. 点击 DAGToolbar 的 **▶ Run** 按钮 (品牌蓝色主按钮)。
2. 系统调用前端验证 → 通过后调用 `PipelineService.Execute` gRPC 流式 RPC。
3. **Run 按钮变为灰色不可点击**, **⏹ Cancel 按钮出现**。
4. 后端开始分阶段处理:

| 阶段 | 说明 | StatusBar 显示 | 耗时占比 |
|------|------|---------------|:---:|
| **LOADING** | 从磁盘加载输入图像文件 | "Loading image DSC_0034.ARW..." | ~5% |
| **DECODING** | 根据 raw_input 参数解码像素数据 | "Decoding RAW — 6000x4000 u16" | ~15% |
| **PROCESSING** | 逐节点顺序执行处理管线 | "Processing: ai_denoise · Tile 3/8 · 42%" | ~70% |
| **ENCODING** | 根据 encoder 参数编码输出 | "Encoding HEIF — q=95, 10-bit 444" | ~8% |
| **SAVING** | 写入输出文件到磁盘 | "Saving output to /output/DSC_0034.heic" | ~2% |
| **DONE** | 处理完成 | "Done — 24.3 MB in 00:03.42" | — |

5. **阶段进度反馈**:
   - 当前执行节点的状态灯以**绿色脉冲**闪烁。
   - StatusBar 进度条以品牌蓝色填充实时更新。
   - AI 降噪节点额外显示 Tile 处理进度 (如 "Tile 6/8, 78%")。

6. **取消执行**:
   - 点击 ⏹ Cancel → 发送 `Cancel` gRPC → 后端停止处理。
   - 当前正在处理的阶段完成后停止 (不是硬中断)。
   - Run 按钮恢复可用。
   - 已处理完成的输出文件已写入磁盘 (不会被删除)。

### 9.3 执行结果

完成后:
- 所有节点状态灯变为**绿色常亮**。
- StatusBar 显示: "Done — 24.3 MB in 00:03.42"。
- 输出文件保存在配置的输出目录。

### 9.4 执行错误处理

如果执行过程中出现错误:

1. 出错的节点状态灯变为**红色**。
2. StatusBar 背景变为红色, 显示错误信息: "ERROR: ai_denoise — ONNX inference failed: CUDA out of memory"。
3. 所有下游节点标记为 "Skipped" (灰色, 表示因上游失败未执行)。
4. 点击错误信息弹出详细错误对话框, 显示:
   - 错误代码 (Error Code)
   - 错误消息 (Error Message)
   - 发生阶段 (Stage)
   - 发生节点 (Node Name)
   - 堆栈跟踪 (Stack Trace, 可选展开)
   - [Copy Error Details] 按钮 (用于提交 Bug 报告)

### 9.5 多次执行

- 修改参数后可以再次点击 ▶ Run 重新执行。
- 每次执行都会覆盖之前的输出文件 (除非更改了输出路径或文件名模板)。
- 节点状态在每次新执行开始时复位。

---

## 10. 批量处理

### 10.1 批量处理概述

批量处理是 Photopipeline 的核心生产力功能。它允许你定义一个管线，然后对大批量图片应用同一管线处理，并实时监控进度。

#### 完整工作流程

```
编辑模式 (Pipeline Editor)
  ① Import images → Filmstrip
  ② Build pipeline → DAG
  ③ Select images → Send to Batch (Badge +N)
  ④ Click [Batch Processing N] → 切换到批量模式

批量模式 (Batch Processing)
  ⑤ Review pipeline summary (左栏, 只读)
  ⑥ Configure output settings (右栏)
  ⑦ (Optional) Per-image override (右栏底部)
  ⑧ ▶ Start Batch → 实时进度监控
  ⑨ All done → 导出完成
  ⑩ Click [Pipeline Editor] → 回到编辑模式
```

### 10.2 步骤详解

#### 步骤 1-3: 在编辑模式中准备

与单张处理的前三步相同 — 导入图片、构建管线、调整参数。

#### 步骤 4: 发送图片到批量队列

有以下几种发送方式:

1. **选中图片 → 点击 Sidebar 的 To Batch 按钮**: 将选中的图片加入批量队列。
2. **选中图片 → 点击 MultiSelectBar 的 To Batch 按钮** (多选时): 批量加入。
3. **在图片卡片上右键 → Send to Batch**: 单张加入。
4. **拖放**: 将图片卡片拖放到 TitleBar 的 [Batch Processing N] 标签上。
5. **全选加入**: Ctrl+A 全选 → To Batch。

每次发送后:
- TitleBar 的 [Batch Processing N] 标签上的 Badge 数字增加。
- StatusBar 短暂显示 "Added N images to batch queue"。
- 图片仍在胶片条中 (不会被移除)。

#### 步骤 5: 切换到批量模式

1. 点击 TitleBar 的 **[Batch Processing N]** 标签。
2. 主界面切换为批量模式三栏布局。
3. 左栏显示当前管线的只读摘要。
4. 中栏显示批量队列和进度。
5. 右栏显示输出设置和逐图覆盖。

### 10.3 批量模式界面详解

#### 左栏: 管线摘要 (260px, 只读)

```
┌─ Pipeline Summary ───────────────────────┐
│ Name: HDR Pipeline v1                     │
│                                           │
│ Nodes:                                    │
│ ● raw_input → ● ai_denoise → ● heif_enc  │
│                                           │
│ Output:                                   │
│ Format: HEIF · Quality: 95%               │
│ Directory: D:/Photos/Output/              │
│                                           │
│ Groups:                                   │
│ ● High ISO (4)                            │
│ ● Night (3)                               │
│ ● GPS: Chengdu (5)                        │
└───────────────────────────────────────────┘
```

此区域完全只读，用于确认当前管线配置是否正确。如果发现问题，需要切换回编辑模式修改。

#### 中栏: 队列与进度 (flex)

##### 控制栏

```
[▶ Start Batch] [⏸ Pause] [▶ Resume] [⏹ Stop] [Clear Done]
```

| 按钮 | 适用状态 | 功能 |
|------|---------|------|
| **▶ Start Batch** | 空闲/暂停 | 开始或继续处理队列中的所有待处理项目 |
| **⏸ Pause** | 运行中 | 暂停处理 — 当前项目完成后暂停, 保留进度 |
| **▶ Resume** | 暂停 | 从暂停位置继续处理剩余项目 |
| **⏹ Stop** | 运行中/暂停 | 取消所有剩余任务, 已完成的项目保留 |
| **Clear Done** | 任意 | 从列表中清除状态为 Done 或 Failed 的项目 |

##### 进度头部

```
████████████████░░░░░░  68%
11 done · 2 failed · 16 total
⏱ 00:03:15 · ~00:01:30 · 14 img/min
```

进度指标:

| 指标 | 计算方式 | 说明 |
|------|---------|------|
| 百分比 | (done+failed) / total x 100% | 蓝色进度条 + 数字 |
| 完成/失败/总计 | 实时计数 | 分类统计 |
| 已用时间 | 从 Start 开始计时 | 动态更新 |
| 预计剩余 (ETA) | elapsed / percent x (100-percent) | 动态估算 |
| 处理速度 | (done+failed) / elapsed_minutes | img/min |

##### 队列列表

每行显示单张图片的处理状态:

```
● DSC_0034.ARW    6000x4000 · ARW    24 MB    Done
● PANO_001.DNG    8256x5504 · DNG    45 MB    Done
● DSC_0036.ARW    6000x4000 · ARW    23 MB    Failed — GPU out of memory
● night_012.NEF   6048x4024 · NEF    28 MB    Processing · Tile 6/8
● city_001.ARW    6000x4000 · ARW    25 MB    Queued
```

| 队列项列 | 内容 |
|---------|------|
| 状态圆点 | 8px: 灰色=Queued, 蓝色脉冲=Processing, 绿色=Done, 红色=Failed |
| 文件名 | fontWeight 500, 主文件名 |
| 分辨率+格式 | "6000x4000 · ARW" |
| 文件大小 | 右对齐 |
| 状态描述 | "Processing · Tile 6/8" 或错误原因 |

**Processing 行特殊样式**: 蓝色边框 + 背景高亮 + 脉冲动画, 方便快速定位当前处理位置。

#### 右栏: 输出设置 (340px)

##### 输出设置

| 设置 | 控件 | 默认值 | 说明 |
|------|:---:|:---:|------|
| **Directory** | Input + 📂 按钮 | 上次使用 | 输出文件存放目录 |
| **Template** | Input | `{date}/{filename}` | 文件名模板, 支持占位符 |
| **Format** | Dropdown (HEIF/JXL/AVIF/TIFF/PNG) | HEIF | 输出格式 (覆盖管线中的 encoder 设置) |
| **Quality** | Slider (0-100) | 95 | 编码质量 (覆盖管线中的 encoder 设置) |
| **Parallel** | SpinButton (1-32) | 4 | 并行处理数 (CPU 核心数的 50-75% 为最佳) |
| **Conflict** | Dropdown (Skip/Overwrite/Rename) | Skip | 同名文件处理策略 |

**文件名模板占位符**:

| 占位符 | 含义 | 示例结果 |
|--------|------|---------|
| `{filename}` | 原始文件名 (不含扩展名) | DSC_0034 |
| `{ext}` | 输出文件扩展名 | heic, jxl |
| `{date}` | 拍摄日期 (YYYY-MM-DD) | 2025-03-15 |
| `{time}` | 拍摄时间 (HH-MM-SS) | 18-42-30 |
| `{camera}` | 相机型号 | ILCE-7RM5 |
| `{iso}` | ISO 值 | 6400 |
| `{index}` | 序号 (递增, 从 1 开始) | 001, 002, 003... |

示例: `{date}/{camera}_{filename}_{iso}ISO.{ext}` → `2025-03-15/ILCE-7RM5_DSC_0034_6400ISO.heic`。

##### 逐图覆盖 (Per-Image Override)

```
┌─ Override ───────────────────────────────────┐
│ [Select queued image ▾]                      │
│                                               │
│ Overrides for night_012.NEF                   │
│ ┌───────────────────────────────────────────┐ │
│ │ Denoise Strength: [0.8___]             🟡 │ │
│ │ Transform Angle:  [1.5°__]             🟡 │ │
│ │ [+ Add parameter override]                │ │
│ └───────────────────────────────────────────┘ │
└───────────────────────────────────────────────┘
```

逐图覆盖允许为队列中的**特定图片**设置独立参数:

1. 从 "Select queued image" 下拉菜单选择图片。
2. 点击 "[+ Add parameter override]"。
3. 在弹出的参数浏览器中选择插件和参数。
4. 修改值 → 参数行右侧显示 🟡 覆盖标记。
5. 该图片执行时将使用覆盖值而非管线默认值。

**典型使用场景**:
- 某张图片特别需要降噪 (ISO 过高): 覆盖 ai_denoise.strength=90
- 某张图片需要水平校正 (微倾): 覆盖 transform.angle=1.5°
- 某张图片需要不同的色温: 覆盖 raw_input.apply_white_balance=false

### 10.4 批量处理执行

#### 开始处理

1. 确认输出设置无误 (目录、格式、质量)。
2. (可选) 为特定图片设置逐图覆盖。
3. 点击 **▶ Start Batch**。
4. 系统将管线定义 + 输出设置 + 逐图覆盖组合为 `BatchSpec` 发送给后端。
5. 后端按 `Parallel` 设置的并行数启动多个处理线程。
6. 每完成一张图片，更新进度:
   - 队列列表中的对应行更新状态 (Queued → Processing → Done/Failed)
   - 进度条、计数、计时器全部实时更新
   - 处理完成的图片其状态圆点变为绿色

#### 暂停与恢复

1. 处理中点击 **⏸ Pause**。
2. 当前正在处理的图片完成后，后端暂停。
3. 进度保留，按钮变为 [▶ Resume] 和 [⏹ Stop]。
4. 点击 **▶ Resume** → 后端从断点继续处理剩余队列项。

#### 停止

1. 点击 **⏹ Stop**。
2. 当前正在处理的图片完成后，后端停止处理。
3. 已完成的图片保留输出结果，剩余的标记为 Queued (不会被处理)。

#### 完成

- 所有项目 Done 或 Failed 时，进度条达到 100%。
- StatusBar 显示总耗时和总处理量。
- 处理结果保存在输出目录。

### 10.5 批量模式下的状态栏

```
● Backend: Connected | Mem: 1.2GB / 2GB | GPU: CUDA · 70°C | Pipeline: HDR_v1 | 16 images
```

比编辑模式多显示: 内存使用、GPU 温度、管线名称、图片计数。

---

## 11. 预览与辅助视图

### 11.1 BeforeAfter 分屏预览

BeforeAfter 是 Photopipeline 的核心预览功能，让你直观对比处理前后的效果。

#### 适用插件

BeforeAfter 预览适用于以下插件 (这些插件的 GuiSchema 中定义了 `PreviewMode::BeforeAfter`):

| 插件 | BeforeAfter 特性 |
|------|-----------------|
| transform | 默认水平分屏, lock_zoom: false |
| colorspace | 水平分屏, lock_zoom: **true** (锁定缩放) |
| lut3d | 水平分屏, lock_zoom: true |
| lens_correct | 水平分屏, lock_zoom: true |
| ai_denoise | 水平分屏, lock_zoom: true |

#### 预览操作

1. 在胶片条中**选中一张图片** (单选)。
2. 在 DAG 画布中**选中一个支持预览的节点**。
3. 右侧面板底部出现 BeforeAfter 分屏预览:
   - 左侧 = 处理前 (Before) — 进入该节点之前的图像。
   - 右侧 = 处理后 (After) — 经过该节点处理后的图像。
4. **拖动中间分割线**调整左右对比区域大小。
5. 默认分割线在中央 (50%)。

#### 预览工具栏

```
🔍+  🔍-  [Fit]  [1:1]  │  125%  │  [Split]  [Export]
```

| 控件 | 功能 |
|------|------|
| 🔍+ / 🔍- | 缩放预览图像 |
| Fit | 自动适应窗口大小 |
| 1:1 | 100% 原始像素视图 (检查细节) |
| 125% | 缩放百分比显示，点击可重置 |
| Split | 切换分屏方向 (水平 ↔ 垂直) |
| Export | 导出当前预览为 PNG 图像 |

#### 预览空态

当没有图片选中或节点不支持预览时，预览区域显示为空:
- 虚线占位框 + "No preview available for this plugin" (插件不支持) 或 "Select an image to preview" (未选中图片)。

### 11.2 辅助视图系统

根据所选中节点的 GuiSchema.aux_views 定义，右侧面板底部动态显示对应的辅助视图。

#### Histogram (直方图)

```
┌─ HISTOGRAM ────────────────────────────┐
│ ▂▃▅▆▇▆▅▃▂▁                            │
│ RGB overlay: R=红 G=绿 B=蓝 L=白(亮度) │
└────────────────────────────────────────┘
```

**适用插件**: transform, colorspace, lut3d, lens_correct, ai_denoise

**显示内容**: 亮度 (Luminance) + RGB 三通道叠加直方图。

**用途**:
- 检查曝光: 纯白溢出 (右端截止) / 纯黑缺失 (左端堆积)。
- 检查色彩平衡: RGB 三个通道的分布是否均衡。
- 评估降噪效果: 直方图形状不应剧烈改变。

#### Waveform (波形监视器)

**适用插件**: transform, colorspace, lut3d, lens_correct, ai_denoise (可选, 取决于 GuiSchema)

**显示内容**: 逐列亮度波形 — X 轴 = 图像水平位置, Y 轴 = 亮度值。

**用途**: 精确评估场景亮度分布，常用于视频调色。

#### Vectorscope (矢量示波器)

```
┌─ VECTORSCOPE ──────────────────────────┐
│            (极坐标图)                    │
│   R· · · Mg                              │
│    ·   ·                                 │
│  Yl · G · Cy· B                          │
│ 皮肤色调线 (Skin Tone Line)              │
└─────────────────────────────────────────┘
```

**适用插件**: lut3d

**显示内容**: 色相/饱和度极坐标图。

**用途**: 色彩分级监视 — 查看 LUT 对色调分布的影响。检查肤色是否在 Skin Tone Line 上。

#### GamutDiagram (色域图)

```
┌─ GAMUT DIAGRAM ─────────────────────────┐
│ CIE 1931 xy 色度图                       │
│ 1. 源色彩空间色域 (白色三角形)            │
│ 2. 目标色彩空间色域 (蓝色三角形)          │
│ 3. 图像实际色彩范围 (点云覆盖)            │
└──────────────────────────────────────────┘
```

**适用插件**: colorspace

**显示内容**: CIE 1931 xy 色度图, 叠加源/目标色域边界和图像实际色彩分布。

**用途**:
- 判断色域是否匹配: 图像颜色是否超出目标色域。
- 选择目标色彩空间: P3 还是 sRGB 足够？
- 检查色域压缩效果: gamut_mapping 是否正确工作。

#### Map (地图)

**适用插件**: gps_set

**显示内容**: 交互式地图 (详见 8.9 节 gps_set 插件的 Map Picker 描述)。

#### StatusText (状态文本)

**适用插件**: lens_correct, ai_denoise

**显示内容**: 文本信息, 如:
- lens_correct: "Detected: Sony ILCE-7RM5 + FE 24-70mm F2.8 GM II · LensFun XML 2025-01-01"
- ai_denoise: "Model: Standard v2 · Backend: CUDA · VRAM: 1.8GB / 2.0GB"

#### ProgressBar (进度条)

**适用插件**: ai_denoise

**显示内容**: AI 推理进度, 如 "Tile 6/8 · 78% · ~00:12 remaining"。

#### 像素信息悬停

在 BeforeAfter 预览中，将鼠标悬停在图像上，StatusBar 或辅助视图区显示像素信息:
```
X: 2456  Y: 1320  R: 128  G: 64  B: 32
```
帮助精确定位色偏和曝光问题。

---

## 12. 设置

点击 TitleBar 右侧 ⚙ 齿轮图标打开设置对话框。

### 12.1 设置对话框布局

```
┌─ Settings ──────────────────────────────┐
│ General │ Backend │ Output │ Advanced   │ ← TabList (标签页切换)
├─────────────────────────────────────────┤
│                                          │
│  (tab content — 可滚动, max-height 400px) │
│                                          │
├─────────────────────────────────────────┤
│              [Cancel] [Reset] [Save]     │ ← Footer 按钮栏
└──────────────────────────────────────────┘
```

- 尺寸: 520x440px, 不可调整大小, 居中显示。
- 切换标签页时内容平滑过渡。

### 12.2 General (通用设置)

| 设置项 | 控件 | 默认值 | 说明 |
|--------|:---:|:---:|------|
| **Theme** | Dropdown (Dark/Light/System) | Dark | 切换后**立即生效**, 无需重启 |
| **Language** | Dropdown (English/中文/日本語) | English | 切换后需**重启应用**生效 |
| **Max Recent Files** | SpinButton (5-50, step=5) | 10 | 最近文件列表最大长度 |
| **Check Updates** | Switch (On/Off) | On | 启动时自动检查更新 (调用 GitHub API) |
| **Telemetry** | Switch (On/Off) | Off | 匿名使用数据上报 (帮助改进产品) |

### 12.3 Backend (后端设置)

| 设置项 | 控件 | 默认值 | 说明 |
|--------|:---:|:---:|------|
| **Server Path** | Input + 📂 | `photopipeline-server` | 后端二进制文件路径 (系统 PATH 或绝对路径) |
| **Port** | SpinButton (1024-65535) | 50051 | gRPC 监听端口 — 修改后需重启 |
| **Auto-start** | Switch (On/Off) | On | 启动应用时自动启动后端进程 |
| **GPU Backend** | Dropdown (Auto/CUDA/CPU/CoreML/OpenVINO) | Auto | GPU 推理后端选择 |
| **Log Level** | Dropdown (Info/Debug/Warn/Error) | Info | 后端日志级别 — Debug 会输出大量日志 |

**GPU Backend 选项详情**:

| 选项 | 说明 |
|------|------|
| **Auto** ★ | 自动检测最佳可用 GPU 后端 |
| **CUDA** | NVIDIA GPU (需 CUDA 12+) |
| **CPU** | 仅使用 CPU (无 GPU 加速) |
| **CoreML** | Apple Silicon Neural Engine |
| **OpenVINO** | Intel GPU/CPU OpenVINO 加速 |

### 12.4 Output (输出设置)

| 设置项 | 控件 | 默认值 | 说明 |
|--------|:---:|:---:|------|
| **Default Format** | Dropdown (HEIF/JXL/AVIF/TIFF/PNG) | HEIF | 默认输出格式 |
| **Default Directory** | Input + 📂 | — (上次使用) | 默认输出目录 |
| **JPEG Quality** | Slider (0-100) | 95 | 默认 JPEG 质量 (如使用 JPEG 相关功能) |
| **Embed Metadata** | Switch (On/Off) | On | 输出时嵌入 EXIF 元数据 |
| **Thumbnail Size** | SpinButton (64-512, step=64) | 120 | 胶片条缩略图加载分辨率 |

### 12.5 Advanced (高级设置)

| 设置项 | 控件 | 默认值 | 说明 |
|--------|:---:|:---:|------|
| **Tile Size** | SpinButton (256-4096, step=64) | 1024 | AI 分块处理大小 (px) — 显存不足时减小 |
| **Cache Directory** | Input + 📂 | `%APPDATA%/Photopipeline/cache` | 缓存目录 |
| **Max Cache Size** | SpinButton (128-8192, step=128) | 1024 MB | 缓存上限 — 超出后自动清理旧缓存 |
| **ExifTool Path** | Input + 📂 | `exiftool` | exiftool 二进制路径 (用于 EXIF 读写) |
| **Reset All** | Button (danger 样式) | — | 恢复所有设置为出厂默认值 (弹出确认对话框) |

### 12.6 设置持久化

- **存储位置**: `%APPDATA%/Photopipeline/appsettings.json` (Windows) / `~/Library/Application Support/Photopipeline/appsettings.json` (macOS) / `~/.config/photopipeline/appsettings.json` (Linux)。
- **保存行为**: 点击 Save 后写入文件 + 立即应用可即时生效的变更 (如 Theme, GPU Backend)。
- **取消行为**: 点击 Cancel 后恢复打开设置对话框时的快照 (不写入文件)。
- **重置行为**: 点击 Reset All → 确认对话框 → 恢复出厂默认值 → 写入文件 → 立即应用。

---

## 13. 键盘快捷键

### 13.1 全局快捷键

| 快捷键 | 功能 | 适用场景 |
|--------|------|---------|
| `Ctrl+O` | 导入图片 (打开文件选择对话框) | 任意 |
| `Ctrl+S` | 保存管线 (Save PipelineConfig) | 编辑模式 |
| `Ctrl+,` | 打开设置对话框 | 任意 |
| `Escape` | 关闭菜单/对话框/清除选择 | 任意 |
| `F11` | 全屏切换 | 任意 |

### 13.2 胶片条 (Sidebar)

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+A` | 全选所有图片 |
| `Escape` | 清除所有选择 |
| `Delete` | 从列表移除选中图片 (不删除文件) |
| `Ctrl+Click` (鼠标) | 追加选择/取消单张 |
| `Shift+Click` (鼠标) | 范围选择 (从上次点击到当前) |
| `↑ / ↓` | 移动当前选中项 (上下切换) |
| `Ctrl+↑ / Ctrl+↓` | 移动当前选中项并追加选择 |

### 13.3 DAG 画布

| 快捷键 | 功能 |
|--------|------|
| `Delete` | 删除选中的节点或连线 |
| `Ctrl+D` | 复制选中的节点 |
| `Ctrl+Z` | 撤销上一步操作 |
| `Ctrl+Y` | 重做被撤销的操作 |
| `Ctrl+E` | 执行管线 (等同于点击 ▶ Run) |
| `Space + 拖动` (鼠标) | 平移画布 (临时切换到手型工具) |
| `Ctrl + 滚轮` (鼠标) | 缩放画布 (向前放大, 向后缩小) |
| `鼠标中键拖动` | 平移画布 |
| `双击空白` | 适应窗口 (Fit) |
| `右键空白` | 弹出 Add Node 上下文菜单 |

### 13.4 预览区

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+=` | 放大预览 |
| `Ctrl+-` | 缩小预览 |
| `Ctrl+0` | 适应窗口 (Fit to Window) |
| `Ctrl+1` | 100% 原始像素 (1:1) |
| `← / →` | 切换上一张/下一张图片 |
| `Ctrl+P` | 导出当前预览为 PNG |

### 13.5 批量处理

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Shift+B` | 发送选中图片到批量队列 |
| `Ctrl+Shift+Enter` | 开始批量处理 (Start Batch) |
| `Ctrl+Shift+P` | 暂停/恢复批量处理 |
| `Ctrl+Shift+Escape` | 停止批量处理 (Stop) |

### 13.6 窗口控制

| 快捷键 | 功能 |
|--------|------|
| `Alt+F4` / `Ctrl+W` | 关闭应用 (Windows/Linux) |
| `Cmd+Q` | 关闭应用 (macOS) |
| `Ctrl+M` | 最小化窗口 |
| `F11` | 全屏切换 |

---

## 14. 典型工作流程教程

### 14.1 场景一: 旅行摄影 — RAW 到社交媒体就绪

**背景**: 你刚结束一次旅行，有 200 张 RAW 照片需要处理。目标是输出适合 Instagram/朋友圈分享的 JPEG XL 照片。

**步骤**:

1. **导入全部照片**
   - 点击 Import → 选择所有 200 张 ARW 文件 → 打开。
   - 等待缩略图加载完成 (约 10-20 秒)。

2. **构建基础管线**
   - 从 PluginBrowser 双击 raw_input → ai_denoise → colorspace → transform → jxl_encoder。
   - 按顺序连线: raw_input → ai_denoise → colorspace → transform → jxl_encoder。

3. **配置 raw_input 节点**
   - 选中 raw_input 节点。
   - raw_mode = Auto (默认) ✓。
   - output_format = 16-bit Integer (默认) ✓。
   - apply_white_balance = Apply (默认) ✓。

4. **配置 AI 降噪 (自适应)**
   - 选中 ai_denoise 节点。
   - Model = Standard v2 (balanced)。
   - **使用表达式实现自适应**: 双击 denoise_strength 参数 → 输入表达式 `clamp(iso / 12800, 0.15, 0.9)`。
     - ISO 100 → 强度 0.01 (几乎不降噪)
     - ISO 3200 → 强度 0.25 (轻降噪)
     - ISO 12800 → 强度 0.9 (强力降噪)
   - detail_preservation = 60 (保留细节)。
   - ai_backend = 根据 GPU 选择 (CUDA/CoreML)。

5. **配置色彩空间**
   - 选中 colorspace 节点。
   - source_color_space = Auto-detect ★。
   - target_color_space = sRGB ★ (社交媒体标准)。
   - rendering_intent = Relative Colorimetric ★。
   - embed_icc = Embed (On) ✓。

6. **配置缩放**
   - 选中 transform 节点。
   - resize_mode = Long Edge ★。
   - long_edge_px = 2048 (Instagram 推荐)。
   - filter_type = Lanczos3 ★ (最佳缩小质量)。
   - 其余保持默认。

7. **配置输出格式**
   - 选中 jxl_encoder 节点。
   - quality = 85 (平衡质量和文件大小)。
   - bit_depth = 8 (广泛兼容)。
   - effort = 5。

8. **保存管线**
   - 点击 Save (Ctrl+S) → 保存为 "Travel_Instagram_v1.ppjson"。

9. **创建分组 (高 ISO)**
   - 点击 GroupTree 底部的 [Auto-group ▾] → By ISO Range。
   - 接受默认区间 → 创建分组。
   - 如有 "High ISO" 组 (ISO>=1600)，可在该组的 ContextBar 中微调降噪参数 (例如 denoise_strength 表达式改为 `clamp(iso / 12800, 0.25, 1.0)`)。

10. **发送到批量并执行**
    - 在胶片条中 Ctrl+A 全选 → 点击 To Batch。
    - 点击 [Batch Processing 200] 切换到批量模式。
    - 设置输出目录: `D:/Photos/Travel_Instagram/`。
    - 设置文件名模板: `{date}/{filename}`。
    - Format = JPEG XL, Quality = 85, Parallel = 4。
    - 点击 ▶ Start Batch。
    - 等待处理完成 (约 20-30 分钟, 取决于硬件)。

**结果**: 200 张优化后的 JPEG XL 互联网就绪照片，高 ISO 的自动使用更强降噪。

### 14.2 场景二: 专业肖像 — RAW 到高质量 HEIF

**背景**: 你拍摄了一组室内人像 (50 张)，使用 Sony A7R5 + 85mm f/1.4。需要高质量输出并嵌入版权信息。

**步骤**:

1. **导入照片**
   - Import → 选择 50 张 ARW 文件。

2. **构建管线**
   - raw_input → lens_correct → ai_denoise → colorspace → exif_rw → heif_encoder。
   - 连线: raw_input → lens_correct → ai_denoise → colorspace → exif_rw → heif_encoder。

3. **配置镜头校正**
   - 选中 lens_correct 节点。
   - correction_mode = Auto-detect from EXIF ★。
   - correct_distortion = Correct ✓。
   - correct_tca = Correct ✓ (消除紫边)。
   - correct_vignetting = Correct ✓ (大光圈暗角校正)。
   - 辅助视图 StatusText 显示: "Detected: Sony ILCE-7RM5 + FE 85mm F1.4 GM"。

4. **配置 AI 降噪**
   - denoise_strength = 40 (室内 ISO 通常较低)。
   - detail_preservation = 75 (人像需要保留皮肤纹理)。
   - color_noise_reduction = 50。

5. **配置色彩空间**
   - target_color_space = Display P3 (Apple 生态最佳)。
   - rendering_intent = Perceptual (人像色彩自然过渡)。

6. **配置 EXIF 版权信息**
   - 选中 exif_rw 节点。
   - Artist = "你的名字"。
   - Copyright = "© 2025 你的名字. All rights reserved."。
   - Keywords = "人像,室内,85mm,肖像"。
   - Rating = 3 (默认)。

7. **配置输出格式**
   - heif_encoder: quality=95, bit_depth=10, chroma=444。
   - 高质量输出，适合存档和分享。

8. **对个别图片微调**
   - 在胶片条中浏览，找到个别需要调整的图片。
   - 在 Image 层覆盖参数 (如某张欠曝: ai_denoise.strength=60)。

9. **执行**
   - 全选 → To Batch → Start Batch。
   - parallel=2 (高分辨率人像处理较慢)。

### 14.3 场景三: 延时摄影 — 批量时间戳修正 + GPS

**背景**: 你用两台相机拍摄了延时摄影，一台相机时间慢了 5 分钟，需要统一时间戳并从 GPX 文件插入 GPS。

**步骤**:

1. **导入所有照片**
   - Import → 选择两台相机的 NEF 文件 (共 800 张)。

2. **按相机型号自动分组**
   - 点击 GroupTree 底部的 [Auto-group ▾] → By Camera Model。
   - 创建 "Nikon Z8" 和 "Nikon Z6III" 两个分组。

3. **针对慢了 5 分钟的相机设置时间偏移**
   - 在 ContextBar 点击 [Nikon Z8]。
   - 添加 time_shift 节点到管线: raw_input → time_shift → jxl_encoder。
   - 在 [Nikon Z8] 层级覆盖: shift_minutes = +5, source_timezone = UTC, target_timezone = local。

4. **配置 GPS (从 GPX 文件)**
   - 添加 gps_set 节点: raw_input → time_shift → gps_set → jxl_encoder。
   - gps_mode = GPX Track。
   - gpx_file = 选择 GPX 文件 (从 GPS 手表导出)。
   - time_offset_seconds = 0 (相机和 GPS 时间已同步)。
   - max_interpolation_gap = 300 (间隔超过 5 分钟不插值)。

5. **执行**
   - 全选 → To Batch → Start Batch。
   - Parallel = 8 (延时照片多为 JPEG basic 或 compressed RAW, 处理较快)。

**结果**: 800 张照片全部获得正确的时间戳和 GPS 坐标，两台相机的时间统一。

### 14.4 场景四: 存档工作流 — RAW 到无损 TIFF 归档

**背景**: 你将 500 张珍贵的胶片扫描 RAW 需要永久数字化存档。要求最高质量、最大兼容性。

**步骤**:

1. **导入照片**
   - Import → 选择 500 张 DNG 文件。

2. **构建管线**
   - raw_input → lens_correct → colorspace → tiff_encoder。
   - 不使用 ai_denoise (胶片扫描通常噪点可控，且降噪可能损失细节)。

3. **配置 raw_input**
   - raw_mode = auto。
   - output_format = 16-bit Integer。
   - apply_white_balance = As-Shot (保留原始扫描白平衡)。

4. **配置镜头校正**
   - correction_mode = auto。
   - 全部校正开启 (distortion/TCA/vignetting/geometry)。

5. **配置色彩空间**
   - target_color_space = ProPhoto RGB (最大色域存档)。
   - rendering_intent = Relative Colorimetric。
   - embed_icc = Embed ✓。

6. **配置 TIFF 输出**
   - compression = LZW (无损压缩，广泛兼容)。
   - bigtiff = BigTIFF (高分辨率可能超 4GB)。
   - pixel_format = 16-bit Integer。
   - embed_icc = Embed ✓。

7. **执行**
   - 全选 → To Batch。
   - 设置输出目录: `/Archive/2025/Film_Scans/`。
   - 文件名模板: `{filename}`。
   - Parallel = 2 (TIFF 文件大，磁盘 I/O 密集)。

**结果**: 500 张 16-bit TIFF 的永久存档文件，保留最大色域和最高质量。

### 14.5 场景五: 高级色彩分级 — 电影级 LUT + HDR 输出

**背景**: 你拍摄了一组短片 (从视频提取的帧)，需要应用电影级色彩分级并输出为 HDR HEIF。

**步骤**:

1. **导入照片**
   - Import → 选择所有提取的帧 (PNG 或 TIFF)。

2. **构建管线**
   - raw_input → colorspace (转 ACEScg) → lut3d (加载电影 LUT) → colorspace (转 BT.2020) → heif_encoder。
   - 双 colorspace 节点的原因: 第一个转到 ACEScg 工作空间应用 LUT, 第二个转到 BT.2020 进行 HDR 输出。

3. **配置第一个 colorspace 节点 (输入到工作空间)**
   - source_color_space = Auto-detect ★。
   - target_color_space = ACEScg (cinema 标记)。
   - rendering_intent = Relative Colorimetric。

4. **配置 lut3d 节点 (色彩分级)**
   - lut_path = 选择你的 .cube 文件 (如 "FilmLook_Kodak2383.cube")。
   - intensity = 90%。
   - input_color_space = ACEScg (与 LUT 设计匹配)。
   - interpolation = Tetrahedral ★。
   - 观察 Vectorscope 辅助视图确认色调分布符合预期。

5. **配置第二个 colorspace 节点 (工作空间到 HDR)**
   - source_color_space = ACEScg。
   - target_color_space = BT.2020 PQ (HDR)。
   - rendering_intent = Relative Colorimetric。
   - embed_icc = Embed ✓。
   - gamut_mapping = Compress ★。

6. **配置 HEIF 输出**
   - quality = 95, bit_depth = 10, chroma = 444。
   - HDR 10-bit 输出。

7. **对部分帧微调**
   - 若某些帧的 LUT 效果过浓: 在 Image 层覆盖 lut3d.intensity=70%。

8. **执行**
   - 全选 → To Batch → Start Batch。

**结果**: 一组电影级色彩分级的 HDR HEIF 帧序列。

---

## 15. PipelineConfig JSON 参考

### 15.1 概述

PipelineConfig 是 Photopipeline 的管线配置文件格式，使用 `.ppjson` 扩展名。它是标准 JSON 格式，包含完整的管线定义 — 节点、连线、参数和覆盖设置。

### 15.2 完整示例

以下是一个包含 raw_input → ai_denoise → colorspace → heif_encoder 管线的 PipelineConfig JSON 示例:

```json
{
  "metadata": {
    "name": "HDR Pipeline v1",
    "version": "1.0",
    "description": "RAW to HEIF with AI denoise and Display P3 color",
    "author": "Zhang Haozhi",
    "created": "2025-03-15T10:00:00Z",
    "modified": "2025-03-15T14:30:00Z",
    "schema_version": "2.0"
  },
  "pipeline": {
    "nodes": [
      {
        "id": "node_001",
        "plugin_id": "photopipeline.plugins.raw_input",
        "label": "RAW Input",
        "position": { "x": 100, "y": 200 },
        "params": {
          "raw_mode": "auto",
          "output_format": "u16",
          "half_size": false,
          "apply_white_balance": true
        }
      },
      {
        "id": "node_002",
        "plugin_id": "photopipeline.plugins.ai_denoise",
        "label": "AI Denoise",
        "position": { "x": 400, "y": 200 },
        "params": {
          "denoise_model": "standard_v2",
          "denoise_strength": {
            "type": "expression",
            "expression": "clamp(iso / 12800, 0, 1)"
          },
          "detail_preservation": 60,
          "color_noise_reduction": 75,
          "ai_backend": "onnx_cuda",
          "tile_size": 1024,
          "use_fp16": true
        }
      },
      {
        "id": "node_003",
        "plugin_id": "photopipeline.plugins.colorspace",
        "label": "Color Space",
        "position": { "x": 700, "y": 200 },
        "params": {
          "source_color_space": "auto",
          "target_color_space": "display_p3",
          "rendering_intent": "relative_colorimetric",
          "black_point_compensation": true,
          "embed_icc": true
        }
      },
      {
        "id": "node_004",
        "plugin_id": "photopipeline.plugins.heif_encoder",
        "label": "HEIF Output",
        "position": { "x": 1000, "y": 200 },
        "params": {
          "quality": 95.0,
          "lossless": false,
          "bit_depth": "10",
          "chroma_subsampling": "444",
          "encoder_effort": 4,
          "tune": "ssim"
        }
      }
    ],
    "edges": [
      {
        "id": "edge_001",
        "source_node": "node_001",
        "source_port": "output",
        "target_node": "node_002",
        "target_port": "input"
      },
      {
        "id": "edge_002",
        "source_node": "node_002",
        "source_port": "output",
        "target_node": "node_003",
        "target_port": "input"
      },
      {
        "id": "edge_003",
        "source_node": "node_003",
        "source_port": "output",
        "target_node": "node_004",
        "target_port": "input"
      }
    ]
  },
  "groups": [
    {
      "name": "High ISO",
      "condition": {
        "field": "iso",
        "operator": ">=",
        "value": 1600
      },
      "overrides": {
        "node_002": {
          "denoise_strength": 0.85,
          "color_noise_reduction": 85
        }
      }
    },
    {
      "name": "Night Shots",
      "condition": {
        "field": "time_range",
        "operator": "between",
        "value": ["21:00", "05:00"]
      },
      "overrides": {
        "node_004": {
          "quality": 90
        }
      }
    }
  ],
  "overrides": [
    {
      "image": "DSC_0036.ARW",
      "node": "node_002",
      "params": {
        "denoise_strength": 0.95
      }
    }
  ],
  "batch": {
    "parallel": 4,
    "output_pattern": "{date}/{filename}",
    "on_conflict": "skip",
    "resume": true,
    "output_directory": "D:/Photos/Output/",
    "default_format": "heif"
  }
}
```

### 15.3 字段说明

#### metadata (管线元数据)

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `name` | string | 是 | 管线名称 (显示在 TitleBar) |
| `version` | string | 是 | 管线版本 (语义化版本) |
| `description` | string | 否 | 管线用途描述 |
| `author` | string | 否 | 作者名称 |
| `created` | ISO 8601 | 否 | 创建时间 |
| `modified` | ISO 8601 | 否 | 最后修改时间 |
| `schema_version` | string | 是 | 配置格式版本 (当前 "2.0") |

#### nodes (节点数组)

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `id` | string | 是 | 节点唯一 ID (如 "node_001") |
| `plugin_id` | string | 是 | 插件 ID (如 "photopipeline.plugins.ai_denoise") |
| `label` | string | 否 | 节点显示标签 |
| `position` | {x,y} | 否 | 节点在 DAG 画布上的坐标 |
| `params` | map | 是 | 参数字典: param_name → value 或 expression 对象 |

**表达式参数格式**:

```json
{
  "denoise_strength": {
    "type": "expression",
    "expression": "clamp(iso / 12800, 0, 1)"
  }
}
```

#### edges (连线数组)

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `id` | string | 是 | 连线唯一 ID |
| `source_node` | string | 是 | 上游节点 ID |
| `source_port` | string | 是 | 上游端口名 (通常 "output") |
| `target_node` | string | 是 | 下游节点 ID |
| `target_port` | string | 是 | 下游端口名 (通常 "input") |

#### groups (分组数组)

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `name` | string | 是 | 分组名称 |
| `condition` | object | 否 | 自动匹配条件 (field + operator + value) |
| `overrides` | map | 否 | 分组默认覆盖参数 |

#### overrides (逐图覆盖数组)

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `image` | string | 是 | 图片文件名 |
| `node` | string | 是 | 节点 ID |
| `params` | map | 是 | 覆盖的参数和值 |

#### batch (批量设置)

| 字段 | 类型 | 必填 | 说明 |
|------|------|:---:|------|
| `parallel` | integer | 否 | 并行处理数 (1-32, 默认 4) |
| `output_pattern` | string | 否 | 文件名模板 |
| `on_conflict` | string | 否 | 冲突策略: "skip" / "overwrite" / "rename" |
| `resume` | boolean | 否 | 是否启用断点续传 |
| `output_directory` | string | 否 | 输出目录 |
| `default_format` | string | 否 | 默认输出格式 |

### 15.4 在命令行中使用 PipelineConfig

Photopipeline 提供命令行工具直接使用 PipelineConfig:

```bash
# 使用管线配置文件处理单张图片
photopipeline run config.ppjson --input DSC_0034.ARW

# 使用管线配置文件处理整个目录
photopipeline run config.ppjson --input-dir /photos/raw/ --output-dir /photos/output/

# 验证管线配置文件的正确性
photopipeline validate config.ppjson

# 列出管线中的所有节点和参数
photopipeline inspect config.ppjson
```

---

## 16. 常见问题 FAQ

### 16.1 安装与启动

**Q1: 应用启动后显示 "Backend Disconnected"，怎么办?**

A: 这是最常见的启动问题。请按以下步骤排查:
1. 确认 `photopipeline-server` 二进制文件存在且在系统 PATH 中。
2. 检查端口 50051 是否被其他程序占用: `netstat -ano | findstr 50051` (Windows) 或 `lsof -i :50051` (macOS/Linux)。
3. 尝试手动启动后端: 打开终端, 运行 `photopipeline serve --port 50051`, 观察输出是否有错误。
4. 前往 Settings → Backend, 确认 Server Path 和 Port 正确。
5. 重启应用 (Auto-start 开启时后端会自动重启)。

**Q2: 支持哪些操作系统?**
A: Windows 10/11 (21H2+), macOS 12+, Linux (Wayland/X11)。不支持 Windows 7/8, 不支持 macOS 11 及更早版本。

**Q3: 安装包多大?**
A: Windows MSI 约 80MB, macOS DMG 约 95MB, Linux AppImage 约 100MB。安装后占据约 250-350MB 磁盘空间 (含运行时和依赖)。

### 16.2 图片管理

**Q4: 导入大量图片 (500+) 时应用卡死怎么办?**
A: 建议:
1. 分批导入, 每次不超过 200 张。
2. 使用命令行批量导入: `photopipeline import --recursive /path/to/photos/`。
3. 减小缩略图大小到 S (80px) 以加快加载。

**Q5: 如何在不删除磁盘文件的情况下从列表移除图片?**
A: 选中图片 → 按 Delete 键或右键 → Remove。这仅从 Photopipeline 的列表中移除, 不影响磁盘上的文件。

**Q6: 支持哪些 RAW 格式?**
A: ARW (Sony), CR2/CR3 (Canon), NEF (Nikon), DNG (通用), RAF (Fujifilm), ORF (Olympus), RW2 (Panasonic), PEF (Pentax)。不支持手机的 ProRAW (DNG) — 这些通常可以导入但色彩科学可能不匹配。

### 16.3 管线编辑

**Q7: 如何创建分支管线 (一个节点输出到多个节点)?**
A: 从一个节点的输出端口拖出连线到第一个下游节点, 再从同一个输出端口拖出另一条连线到第二个下游节点。DAG 支持一对多输出 (分支), 但不能循环连接。

**Q8: 为什么我不能连接两个节点?**
A: 可能的原因:
1. 类型不兼容: Metadata 节点不能连接到 Pixel 节点。
2. 尝试创建循环连接: A→B→C→A 会被系统自动拒绝。
3. 输入端口已有连线: 大多数节点只有一个输入端口, 如果已被占用需先删除旧连线。

**Q9: Disable 节点和 Delete 节点有什么区别?**
A: Disable (右键 → Disable) 让节点变灰, 执行时跳过但不删除。Delete (选中 → Delete 键) 将节点及连线永久删除。Disable 适合 A/B 测试 (比较有无某节点的效果); Delete 适合清理不需要的节点。

**Q10: 撤销 (Undo) 的次数有限制吗?**
A: 默认支持 50 步撤销操作。超过 50 步时最旧的记录被丢弃。切换模式 (编辑模式 ↔ 批量模式) 不会清除撤销历史。

### 16.4 参数与覆盖

**Q11: 参数显示为灰色无法编辑, 为什么?**
A: 可能原因:
1. 当前 ContextBar 选中了 "All" (只读查看模式) — 切换到 Template/Group/Image 层级。
2. 管线正在执行中 (参数锁定) — 等待执行完成。
3. 后端断连 (全部操作禁用) — 检查后端连接状态。
4. 参数在当前层级处于继承状态 (⬜) — 点击 ⬜ 激活编辑。

**Q12: 表达式编辑器支持哪些函数?**
A: clamp, min, max, abs, lerp, round。可用变量: iso, aperture, shutter, focal_length, ev, filename。详见 7.6 节。

**Q13: Image 层覆盖和 Group 层覆盖冲突时, 哪个生效?**
A: Image 层 (优先级 3) 高于 Group 层 (优先级 2)。如果某参数在 Image 层有覆盖, 该值生效; 否则检查 Group 层; 再否则检查 Template 层; 最后取插件内置默认值。

**Q14: 如何批量清除某分组的所有覆盖?**
A: 在 ContextBar 点击该分组的标签 → 在每个覆盖的参数上悬停 → 点击 × (恢复继承)。或右键分组 → Edit Group → 删除所有默认覆盖参数。

### 16.5 批量处理

**Q15: 批量处理中如何只为特定图片设置不同参数?**
A: 使用逐图覆盖 (Per-Image Override):
1. 在批量模式的右栏底部 Override 区域。
2. 从下拉菜单选择图片。
3. 点击 [+ Add parameter override] → 选择插件和参数 → 设置值。
4. 该图片执行时将使用覆盖值。

**Q16: 批量处理中的 Parallel 设置多少合适?**
A: 建议设置为**物理核心数的 50-75%**:
- 4 核 CPU: Parallel = 2-3
- 8 核 CPU: Parallel = 4-6
- 16 核 CPU: Parallel = 8-12
- GPU 加速 (CUDA): 可以设置更高 (GPU 处理与 CPU 任务交错)。

过高的 Parallel 值可能导致内存耗尽 (每张图片的中间数据占 ~200-500MB)。

**Q17: 暂停批量处理后, 关闭应用再重新打开能恢复吗?**
A: 可以。如果 resum=true (默认), 应用会保存批量进度到 `%APPDATA%/Photopipeline/batch_resume.json`。重新打开应用后, Batch 标签应显示上次的队列和进度。

**Q18: 如何处理批量处理后的失败项目?**
A:
1. 查看失败原因: 中栏队列列表中每行显示错误信息 (如 "Failed — GPU out of memory")。
2. 修正问题: 可能是内存不足 (减少 Parallel / 减小 Tile Size) 或文件损坏。
3. 清除失败项目: 点击 Clear Done → 失败项目移出队列。
4. 重新发送失败的项目: 在编辑模式的胶片条中找到 → 重新发送到批量。

### 16.6 性能与硬件

**Q19: AI 降噪非常慢, 如何加速?**
A:
1. 确保使用 GPU 后端: Settings → Backend → GPU Backend = CUDA (NVIDIA) 或 CoreML (Apple Silicon)。
2. 减小 tile_size: 如果 GPU 显存不足, 系统会 fallback 到 CPU, 速度慢 5-10 倍。将 tile_size 从 1024 减小到 512 或 256。
3. 使用轻量级模型: denoise_model = Lightweight v1 (速度最快)。
4. 使用 FP16: use_fp16 = FP16 (半精度推理比 FP32 快 ~2 倍)。

**Q20: 系统内存使用过多怎么办?**
A:
1. 限制批量 Parallel 数。
2. Settings → Advanced → Max Cache Size 设为较低值 (如 256 MB)。
3. Settings → Advanced → Tile Size 减小 (降低单块内存占用)。
4. 定期清理缓存目录: `%APPDATA%/Photopipeline/cache/`。

**Q21: 输出文件比预期大很多?**
A:
1. 检查 encoder 的 quality 设置 (建议: HEIF 90-95, JXL 85-90, AVIF 80-85)。
2. 检查是否误启用了 lossless 模式。
3. 对于 HEIF/AVIF: 减小 chroma_subsampling (444 → 422 → 420)。
4. 对于 TIFF: 使用 compression=LZW 或 Deflate。

### 16.7 输出与格式

**Q22: 应该选择哪种输出格式?**
A: 快速决策树:
- 需要最广泛兼容 → **PNG** (8-bit)
- Apple 设备照片 → **HEIF** (10-bit, 444)
- Web 发布 (最小体积) → **AVIF** (quality=80, chroma=420)
- 长期归档 → **JPEG XL** (16-bit, quality=95) 或 **TIFF** (LZW)
- 专业打印 → **TIFF** (16-bit, LZW)
- HDR 内容 → **HEIF** (10-bit, BT.2020 PQ) 或 **JPEG XL** (16-bit)

**Q23: 如何只修改元数据 (不重新编码像素)?**
A: 在管线中仅使用 Metadata 类型插件 (exif_rw, gps_set, time_shift), 不需要 encoder 节点。元数据修改直接作用于原始文件。

**Q24: 输出文件自动命名为 `DSC_0034_processed.heic`, 如何自定义?**
A: 在批量模式的 Output Settings 中修改 Template。支持的占位符: `{filename}`, `{ext}`, `{date}`, `{time}`, `{camera}`, `{iso}`, `{index}`。例如 `{date}/{camera}_{iso}ISO.{ext}` → `2025-03-15/ILCE-7RM5_6400ISO.heic`。

### 16.8 管线文件

**Q25: .ppjson 管线文件可以分享给其他人吗?**
A: 可以。PipelineConfig 是标准 JSON 文件, 包含完整的管线定义和参数。其他人加载后只需调整输入图片路径。建议使用版本控制管理 .ppjson 文件 (如 Git)。

**Q26: 命令行和 GUI 使用同一个管线文件吗?**
A: 是的。.ppjson 文件在 GUI 中保存后, 可以在命令行中使用: `photopipeline run config.ppjson --input-dir /path/to/photos/`。命令行也接受同样的 PipelineConfig JSON 格式。

**Q27: 管线文件损坏了怎么办?**
A: Photopipeline 在保存时会先写入临时文件, 然后原子性替换。如果文件损坏, 检查同目录下的 `.ppjson.bak` 备份文件 (最近一次的保存)。也可以手动修复 JSON 格式错误 (使用 JSON 验证器检查)。

---

**全手册完。版本 2.0, 最后更新 2026-05-29。**

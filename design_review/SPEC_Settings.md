# Settings Dialog — 详细设计规格书

## 1. 布局

```
┌─ Settings ──────────────────────────┐
│ General │ Backend │ Output │ Advanced│ ← TabList
├─────────────────────────────────────┤
│  (tab content — scrollable 400px)   │
├─────────────────────────────────────┤
│              [Cancel] [Reset] [Save]│ ← Footer
└─────────────────────────────────────┘
```

尺寸: 520×440px, NoResize, CenterOwner

## 2. Tab 内容

### General

| 设置 | 控件 | 默认 |
|------|:---:|------|
| Theme | Dropdown (Dark/Light/System) | Dark |
| Language | Dropdown (English/中文/日本語) | English |
| Max Recent Files | SpinButton (5-50) | 10 |
| Check Updates | Switch | On |
| Telemetry | Switch | Off |

### Backend

| 设置 | 控件 | 默认 |
|------|:---:|------|
| Server Path | Input+📂 | `photopipeline-server` |
| Port | SpinButton (1024-65535) | 50051 |
| Auto-start | Switch | On |
| GPU Backend | Dropdown (Auto/CUDA/CPU/CoreML/OpenVINO) | Auto |
| Log Level | Dropdown (Info/Debug/Warn/Error) | Info |

### Output

| 设置 | 控件 | 默认 |
|------|:---:|------|
| Default Format | Dropdown (HEIF/JXL/AVIF/TIFF/PNG) | HEIF |
| Default Directory | Input+📂 | — |
| JPEG Quality | Slider (0-100) | 95 |
| Embed Metadata | Switch | On |
| Thumbnail Size | SpinButton (64-512) | 120 |

### Advanced

| 设置 | 控件 | 默认 |
|------|:---:|------|
| Tile Size | SpinButton (256-4096) | 1024 |
| Cache Directory | Input+📂 | `%APPDATA%/Photopipeline/cache` |
| Max Cache Size | SpinButton (128-8192) | 1024 MB |
| ExifTool Path | Input+📂 | `exiftool` |
| Reset All | Button | 确认对话框 |

## 3. 持久化

- 格式: JSON → `%APPDATA%/Photopipeline/appsettings.json`
- Save: 写入文件 + 立即应用主题变更
- Cancel: 恢复打开时的快照
- Reset: 恢复出厂默认值 (确认对话框)

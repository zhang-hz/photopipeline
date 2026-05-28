# Batch Processing — 详细设计规格书

---

## 1. 双模式架构

```
┌─ TitleBar ──────────────────────────────────────────────────────┐
│ ◆ Photopipeline    [Pipeline Editor] [Batch Processing 16]  ◐ ⚙│
└─────────────────────────────────────────────────────────────────┘
```

**两个模式标签**（标题栏右上角）:
- `[Pipeline Editor]` — 编辑模式（默认）
- `[Batch Processing N]` — 批量模式，N = 队列中图片数

点击标签切换整个主内容区域。批量模式标签上的 Badge 数字动态反映队列大小。

### 模式切换触发

| 触发 | 行为 |
|------|------|
| 点击 Batch 标签 | 从编辑模式切换到批量模式 |
| 点击 Pipeline Editor 标签 | 从批量模式切换回编辑模式 |
| 编辑模式中 Send to Batch | 图片加入队列，Badge 数字 +N，保持在编辑模式 |
| 批量模式中点 ▶ Start Batch | 开始执行（不切换模式） |

---

## 2. 编辑模式 (Pipeline Editor)

不重复描述。详见主界面规格书。

---

## 3. 批量模式布局

```
┌─ Left (260px) ─┬─ Center (flex) ────────┬─ Right (340px) ────┐
│ Pipeline Summary│ Batch Queue + Progress │ Output Settings     │
│                 │                        │ + Per-Image Override│
│ Nodes           │ ▶ Start ⏸ ⏹          │                     │
│ Output Config   │ ████████░░ 68%        │ Directory           │
│ Groups          │ 11 done · 2 failed    │ Template            │
│                 │                        │ Format / Quality    │
│                 │ ● DSC_0034 Done       │ Parallel / Conflict │
│                 │ ● PANO_001 Done       │                     │
│                 │ ● DSC_0036 Failed     │ ┌─ Override ──────┐ │
│                 │ ● night_012 Processing│ │ night_012.NEF   │ │
│                 │ ● city_001 Queued     │ │ Denoise: 0.8 🟡 │ │
│                 │ ● city_002 Queued     │ │ + Add override  │ │
│                 │                        │ └────────────────┘ │
├─────────────────┴────────────────────────┴────────────────────┤
│ ● Backend: Connected | Mem: 1.2GB | GPU: CUDA | Pipeline: HDR │
└───────────────────────────────────────────────────────────────┘
```

### 3.1 左栏: 管线摘要 (260px)

| 区域 | 内容 | 说明 |
|------|------|------|
| Pipeline Name | HDR_v1 | 当前管线名称 |
| Nodes | raw_input → ai_denoise → colorspace → heif_encoder | 只读迷你视图，彩色圆点 |
| Output | Format/Quality/Directory | 从输出设置提取 |
| Groups | High ISO(4), Night(3), GPS(5) | 分组概览 |

**交互**: 只读，不可编辑。双击 Pipeline Name 可切换到编辑模式定位该管线。

### 3.2 中栏: 队列 + 进度 (flex)

#### 控制栏

| 按钮 | 状态逻辑 |
|------|---------|
| ▶ Start Batch | 空闲/暂停 → 开始执行 |
| ⏸ Pause | 运行中 → 暂停（保留进度） |
| ⏹ Stop | 运行中/暂停 → 取消剩余项 |
| Clear Done | 移除已完成项 |

**进度头部:**

```
████████████████░░░░░░  68%
11 done · 2 failed · 16 total
⏱ 00:03:15 · ~00:01:30 · 14 img/min
```

#### 队列列表

每行的列:

| 列 | 宽度 | 内容 |
|------|:---:|------|
| 状态圆点 | 8px | queued(灰) / processing(蓝脉冲) / done(绿) / failed(红) |
| 文件名 | 180px | fontWeight 500 |
| 分辨率 | 120px | "6000×4000 ARW" |
| 大小 | 60px | "24 MB" |
| 状态文字 | auto | "Done" / "Processing · Tile 6/8" / "Failed — reason" |

**Processing 行特殊样式**: 蓝色边框 + 背景, 脉冲动画

### 3.3 右栏: 输出设置 + 逐图覆盖 (340px)

#### Output Settings

| 设置 | 控件 | 默认 |
|------|:---:|------|
| Directory | Input + 📂 | — |
| Template | Input | `{date}/{filename}` |
| Format | Dropdown (HEIF/JXL/TIFF/PNG/AVIF) | HEIF |
| Quality | Slider (0-100) | 95 |
| Parallel | SpinButton (1-32) | 4 |
| Conflict | Dropdown (Skip/Overwrite/Rename) | Skip |

#### Per-Image Override

```
┌─ Override ──────────────────────────────┐
│ [Select queued image ▾]                 │ ← Dropdown 选择图片
│                                          │
│ Overrides for night_012.NEF              │
│ ┌──────────────────────────────────────┐ │
│ │ Denoise Strength: [0.8___]        🟡 │ │ ← 覆盖参数行
│ │ Exposure: [-0.5___]               🟡 │ │
│ │ [+ Add parameter override]          │ │ ← 添加新覆盖
│ └──────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

## 4. 队列项状态详解

| 状态 | 圆点 | 文字 | 样式 |
|------|:---:|------|------|
| Queued | 灰 | "Queued" | 默认 |
| Processing | 蓝(脉冲) | "Processing · Tile N/M" | 蓝色边框+背景 |
| Done | 绿 | "Done" | 默认 |
| Failed | 红 | "Failed — reason" | 默认 |

## 5. 进度指标计算

| 指标 | 公式 |
|------|------|
| 完成率 | (done+failed) / total × 100% |
| 速度 | (done+failed) / elapsed_minutes |
| ETA | (elapsed / pct) × (100-pct) |
| 已用时间 | 从 Start 开始计时 |

## 6. 状态栏（批量模式）

```
● Backend: Connected | Mem: 1.2GB / 2GB | GPU: CUDA · 70°C | Pipeline: HDR_v1 | 16 images
```

比编辑模式多显示：内存使用、GPU 温度、管线名称、图片计数。

## 7. 工作流

```
编辑模式
  ① Import images → Filmstrip
  ② Build pipeline → DAG
  ③ Select images → Send to Batch (Badge +N)
  ④ Click [Batch Processing N] → 切换到批量模式

批量模式
  ⑤ Review pipeline summary (只读)
  ⑥ Configure output settings
  ⑦ (Optional) Per-image override
  ⑧ ▶ Start Batch → 实时进度
  ⑨ All done → 导出完成
  ⑩ Click [Pipeline Editor] → 回到编辑模式
```

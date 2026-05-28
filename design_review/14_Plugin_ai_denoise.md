# GUI 设计审查 — #14 AI Denoise 插件

---

## 基本信息

| 属性 | Rust 定义 | C# UI |
|------|----------|-------|
| ID | `photopipeline.plugins.ai_denoise` | `denoise`（**不匹配**） |
| 名称 | `AI Denoise` | `Denoise`（**不同**） |
| 分类 | Enhance | Enhance |
| 描述 | AI-based noise reduction using ONNX models | AI-based noise reduction for luminance and chrominance noise |
| 硬件需求 | min_ram=2GB, GPU recommended | 未显示 |

## 一、GUI Schema 设计（Rust）

```rust
GuiSchema {
    sections: [
        model (Card),           // AI 模型选择
        strength (Card),        // 降噪强度
        hardware (CollapsibleCard), // 硬件加速选项
    ],
    icon: "sparkles",
    color: "#a855f7" (紫色),
    preview: PreviewMode::BeforeAfter {
        default_split: 0.5,
        orientation: Horizontal,
        lock_zoom: true,        // 锁缩放（确保对比一致性）
    },
    aux_views: vec![
        AuxView::Histogram,     // 直方图
        AuxView::ProgressBar,   // AI 推理进度
        AuxView::StatusText,    // 状态文字（模型加载状态等）
    ],
    min_panel_width: 360,
}
```

**设计预期：**
- 三个参数卡片：模型选择、降噪强度、硬件加速（可折叠）
- 火花图标 + 紫色主题
- Before/After 锁定缩放对比（确保公平对比）
- **3 个辅助视图：直方图、进度条、状态文字**
- 最宽的面板（360px）— AI 降噪需要更多参数空间

## 二、C# UI 显示

```
┌──────────────────────────────────┐
│ Denoise                      v1.0│
│ AI-based noise reduction for...  │
│            [Enhance]             │
├──────────────────────────────────┤
│ Parameters — Denoise             │
│ ┌──────────────────────────────┐ │
│ │ Strength:[0.5   ▲▼]        │ │ ← float
│ │ Luma:    [0.5   ▲▼]        │ │ ← float
│ │ Chroma:  [0.5   ▲▼]        │ │ ← float
│ └──────────────────────────────┘ │
│ [Reset] [Add to Pipeline]        │
└──────────────────────────────────┘
```

### 参数控件对比

| 参数 | Rust 类型 | C# 控件 | 匹配度 |
|------|----------|---------|:-----:|
| strength | float(0-1, step=0.01) | NumberBox | ✅ 但缺 step |
| luma | float(0-1, step=0.01) | NumberBox | ✅ 但缺 step |
| chroma | float(0-1, step=0.01) | NumberBox | ✅ 但缺 step |

所有 3 个浮点数参数都应为 **Slider**（带步进），但 C# 用 NumberBox 替代。

## 三、GuiSchema 利用程度

| GuiSchema 定义 | 值 | C# UI |
|---------------|-----|-------|
| icon | "sparkles" | ❌ 未使用 |
| color | "#a855f7"（紫色） | ❌ 未使用 |
| 3 个 Section | model / strength / hardware | ❌ 无分区 |
| CollapsibleCard（hardware） | 可折叠 | ❌ 不支持 |
| PreviewMode | BeforeAfter(lock_zoom) | ❌ Preview 不与插件关联 |
| **AuxView::Histogram** | 直方图 | ❌ **未实现** |
| **AuxView::ProgressBar** | AI 推理进度 | ❌ **未实现** |
| **AuxView::StatusText** | 模型状态文字 | ❌ **未实现** |
| min_panel_width = 360 | 最宽 | ❌ 未使用 |

## 四、关键问题

1. **缺少模型选择** — Rust 端支持多个 ONNX 模型（Standard/High Quality/Fast 等），C# 完全没有模型选择 UI
2. **缺少硬件加速配置** — Rust 端支持 CPU/GPU/NPU 后端选择，C# 无此选项
3. **缺少辅助视图** — Histogram、ProgressBar、StatusText 全部缺失
4. **浮点参数应为 Slider** — 0-1 范围的参数更适合滑块交互

## 五、完成度评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 插件卡片 | 70% | 文字完整，缺模型信息/硬件要求显示 |
| 参数控件 | 50% | 基础参数类型 OK，缺模型选择/硬件配置 |
| 参数交互 | 30% | 0-1 浮点应用 Slider 而非 NumberBox |
| GuiSchema 利用 | 5% | 几乎全部忽略 |
| 辅助视图 | 0% | 直方图/进度条/状态文字全部缺失 |
| 预览集成 | 0% | PreviewMode 不作用于 PreviewView |
| **综合** | **25%** | **作为最复杂的 AI 插件，GUI 实现的差距最大** |

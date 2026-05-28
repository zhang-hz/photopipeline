# 插件 #06: ai_denoise — 详细设计规格书

> 后端: `feat/unified-binary` · `crates/plugins/src/ai_denoise.rs`

---

## 1. 元数据

| 字段 | 值 |
|------|-----|
| 名称 | AI Denoise |
| ID | `photopipeline.plugins.ai_denoise` |
| 分类 | Enhance |
| 描述 | AI-powered image denoising using ONNX Runtime |
| 标签 | ai, denoise, onnx, gpu |
| 能力 | PixelProcessor + **AiProcessor** |
| 内存 | **2048 MB** |
| GPU | 推荐 (CUDA preferred) |

## 2. GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `"sparkles"` |
| color | `"#a855f7"` (紫罗兰) |
| preview | BeforeAfter (lock_zoom: true) |
| aux_views | **[Histogram, ProgressBar, StatusText]** |
| min_panel_width | **360px** (最宽面板) |

## 3. 参数

### 3.1 Model (Card, 展开, 1 param + ModelInfo 卡片)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `denoise_model` | Enum (4选项) | `"standard_v2"` | Dropdown |

选项: Lightweight v1(★,fast) / Standard v2(balanced) / High Quality v2(quality,slow) / RAW Denoise v1(raw)

选中模型后，动态显示 **ModelInfo 卡片**：

```
┌─ ModelInfo ──────────────────────────────┐
│ PhotoPipeline Denoise Standard v2         │
│ v2.0.0 · HuggingFace · ONNX Runtime       │
│ Input: [1,3,1024,1024] · VRAM: ~2GB       │
│ Balanced denoising for ISO 100-12800      │
└───────────────────────────────────────────┘
```

### 3.2 Strength (Card, 展开, 3 params)

| 参数 | 类型 | 默认 | 控件 | 高级 |
|------|------|------|:---:|:---:|
| `denoise_strength` | Slider (0–100, ticks[0,25,50,75,100]) | 50 | Slider + 刻度 | |
| `detail_preservation` | Slider (0–100, ticks[0,25,50,75,100]) | 50 | Slider + 刻度 | |
| `color_noise_reduction` | Slider (0–100, ticks[0,50,100]) | 75 | Slider + 刻度 | **是** |

### 3.3 Hardware (CollapsibleCard, 折叠, 3 params)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `ai_backend` | Enum (5选项) | `"onnx_cpu"` | Dropdown |
| `tile_size` | Integer (0–4096, step=64) | 0 | **SpinButton + px** |
| `use_fp16` | Boolean (FP16/FP32) | `true` | Switch |

ai_backend 选项: ONNX CPU(★,cpu) / ONNX CUDA(gpu,cuda) / TensorRT(gpu,nvidia) / CoreML ANE(apple) / OpenVINO(intel)

## 4. 新增辅助视图

| 视图 | ai_denoise 首次引入 |
|------|-------------------|
| **ProgressBar** | 显示 AI 推理进度 (Tile 6/8 · 78%) |
| StatusText | 显示当前模型/后端/GPU 信息 |

## 5. 控件累计（最终版）

| 控件 | 01 | 02 | 03 | 04 | 05 | 06 |
|------|:---:|:---:|:---:|:---:|:---:|:---:|
| Dropdown | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Switch | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Input (text) | ✅ | — | — | — | ✅ | — |
| SpinButton | — | ✅ | — | — | — | ✅ |
| Slider (Int/Float) | — | ✅ | — | — | — | — |
| ParamType::Slider | — | — | — | ✅ | — | ✅ |
| FilePath (file) | — | — | ✅ | ✅ | — | — |
| FilePath (dir) | — | — | — | — | ✅ | — |
| ModelInfo 卡片 | — | — | — | — | — | ✅ |

## 6. 后端适配确认

- [x] denoise_model 4 个枚举值（含标签:fast/balanced/quality/slow/raw）
- [x] 3 个 ParameterType::Slider 全部 show_ticks=true
- [x] color_noise_reduction advanced=true
- [x] ai_backend 5 个枚举值（含平台标签:gpu/cuda/apple/intel）
- [x] tile_size Integer step=64, SpinButton 控件
- [x] use_fp16 Boolean FP16/FP32
- [x] GuiSchema: sparkles/#a855f7/BeforeAfter(lockZoom)/[Histogram,ProgressBar,StatusText]/360px

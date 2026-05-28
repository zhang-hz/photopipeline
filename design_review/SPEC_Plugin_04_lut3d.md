# 插件 #04: lut3d — 详细设计规格书

> 后端来源: `feat/unified-binary` · `crates/plugins/src/lut3d.rs`

---

## 1. 插件元数据

| 字段 | 值 |
|------|-----|
| 名称 | 3D LUT |
| ID | `photopipeline.plugins.lut3d` |
| 版本 | 1.0.0 |
| 分类 | Color |
| 描述 | Apply 3D Look-Up Tables for color grading and film emulation |
| 标签 | lut, grading, color, cube, film |
| 能力 | PixelProcessor |
| 内存 | 256 MB |
| GPU | **推荐** (`requires_gpu: true`) |
| 像素 | 输入+输出 |

## 2. GuiSchema

| 字段 | 值 |
|------|-----|
| icon | `"grid3x3"` |
| color | `"#ec4899"` (粉) |
| preview | BeforeAfter (lock_zoom: true) |
| aux_views | **[Histogram, Vectorscope]** |
| min_panel_width | 320px |

## 3. 参数定义

### 3.1 LUT File (Card, 展开)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `lut_path` | **FilePath** (.cube/.3dl/.look) | `""` | Input + 📂 |
| `lut_format` | Enum (4选项) | `"cube"` | Dropdown |

### 3.2 Transform (Card, 展开)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `intensity` | **ParameterType::Slider** (0–100, step=1, ticks=[0,25,50,75,100]) | `100.0` | **Slider + 刻度** |
| `input_color_space` | Enum (4选项) | `"srgb"` | Dropdown + `[advanced]` |
| `clamp_output` | Boolean (Clamp/Pass Through) | `true` | Switch |

### 3.3 Interpolation (CollapsibleCard, 折叠)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `interpolation_method` | Enum (2选项) | `"tetrahedral"` | Dropdown + `[advanced]` |

## 4. 新增: ParameterType::Slider

lut3d 的 `intensity` 是第一个使用 `ParameterType::Slider` 的参数，与此前的 Integer/Float 类型有本质区别：

```
此前: Integer/Float → 前端自己决定用 Slider 还是 SpinButton
现在: ParameterType::Slider → 后端明确要求用 Slider 渲染
```

| Slider 字段 | 值 | 前端渲染 |
|-----------|-----|---------|
| `min` | 0.0 | 刻度起点 |
| `max` | 100.0 | 刻度终点 |
| `step` | 1.0 | 步进 |
| `show_ticks` | **true** | 显示刻度标记 |
| `ticks` | [0, 25, 50, 75, 100] | 5 个刻度 |
| `show_value` | **true** | 右侧显示当前数值 |

**Fluent 渲染:**
```
Intensity  ═══════○════════  100%
          0    25   50   75  100
```

## 5. 新增辅助视图: Vectorscope

| 属性 | 值 |
|------|-----|
| 类型 | `AuxView::Vectorscope` |
| 显示 | 色相/饱和度极坐标图 |
| 用途 | 色彩分级监视 — 查看 LUT 对色调分布的影响 |

## 6. 控件累计

| 控件 | raw | transform | colorspace | lut3d |
|------|:---:|:---:|:---:|:---:|
| Dropdown | ✅ | ✅ | ✅ | ✅ |
| Switch | ✅ | ✅ | ✅ | ✅ |
| Input (text) | ✅ | — | — | — |
| SpinButton | — | ✅ | — | — |
| Slider (from Integer/Float) | — | ✅ | — | — |
| FilePath | — | — | ✅ | ✅ |
| **ParameterType::Slider** | — | — | — | ✅ (new) |

| 辅助视图 | raw | transform | colorspace | lut3d |
|---------|:---:|:---:|:---:|:---:|
| Histogram | — | ✅ | ✅ | ✅ |
| GamutDiagram | — | — | ✅ | — |
| **Vectorscope** | — | — | — | ✅ (new) |

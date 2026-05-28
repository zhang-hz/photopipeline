# 插件 #01: raw_input — 详细设计规格书

> 后端来源: `feat/unified-binary` · `crates/plugins/src/raw_input.rs`  
> 前端框架: Fluent 2 + React · 面板宽度: 420px · 数据源: `PluginService.GetNodeSchema()`

---

## 1. 插件元数据

| 字段 | 值 | 来源 |
|------|-----|------|
| 名称 | RAW Input | `fn name()` |
| 插件 ID | `photopipeline.plugins.raw_input` | `fn id()` |
| 版本 | 1.0.0 | `fn version()` |
| 分类 | Input | `PluginCategory::Input` |
| 描述 | Read RAW camera files (ARW, CR2, CR3, NEF, DNG, RAF, ORF, RW2, PEF) | `fn description()` |
| 标签 | `input`, `raw`, `camera`, `decode` | `TAGS` |
| 能力 Trait | FormatProcessor | `impl FormatProcessor` |
| 最低内存 | 512 MB | `HardwareRequirement.min_ram_mb` |
| 像素输入 | 否 | `requires_pixel_access() → false` |
| 像素输出 | 是 | `produces_pixel_output() → true` |

---

## 2. GuiSchema

| 字段 | 值 | 说明 |
|------|-----|------|
| layout | `GuiLayout::Standard` | 标准分区布局 |
| icon | `"camera"` | 面板标题图标 |
| color | `"#ef4444"` | 图标背景色 / 分类标识色 |
| preview | `PreviewMode::None` | 无预览 |
| aux_views | `[]` | 无辅助视图 |
| min_panel_width | 320 | 面板最小宽度 |

### 分区结构

| 分区 ID | 标题 | 样式 | 默认状态 |
|---------|------|------|---------|
| `raw_format` | RAW Format | Card | 展开 |
| `output` | Output | Card | 展开 |
| `dcraw_options` | dcraw Options | CollapsibleCard | 折叠 |

---

## 3. 参数详细定义

### 3.1 分区: RAW Format

| 属性 | 值 |
|------|-----|
| 分区 ID | `raw_format` |
| 标题 | RAW Format |
| 描述 | RAW file format detection and processing |
| 样式 | Card (始终可见) |
| 可折叠 | 否 |
| 默认折叠 | 否 |

#### 参数: `raw_mode`

| 字段 | 值 |
|------|-----|
| 参数 ID | `raw_mode` |
| 标签 | Decode Mode |
| 描述 | How to process the RAW file |
| 类型 | Enum |
| 默认值 | `"auto"` |
| 必填 | 否 |
| 高级选项 | 否 |
| 允许覆盖 | 是 |
| 支持表达式 | 否 |

**枚举选项：**

| 值 | 标签 | 描述 | 推荐 |
|------|------|------|:---:|
| `"auto"` | Auto | Detect from file, use best method | ★ |
| `"dcraw"` | dcraw | Use dcraw for raw conversion | |
| `"libraw"` | LibRaw | Use LibRaw via FFI (when available) | |
| `"rawtherapee"` | RawTherapee | Use RawTherapee CLI | |

**Fluent 控件：** `<Dropdown>`
- 显示格式: `"Auto ★ — Detect from file, use best method"`
- 推荐项添加 ★ 标记

---

### 3.2 分区: Output

| 属性 | 值 |
|------|-----|
| 分区 ID | `output` |
| 标题 | Output |
| 描述 | RAW decoding output options |
| 样式 | Card (始终可见) |
| 默认折叠 | 否 |

#### 参数: `output_format`

| 字段 | 值 |
|------|-----|
| 参数 ID | `output_format` |
| 标签 | Output Pixel Format |
| 描述 | Pixel format for decoded output |
| 类型 | Enum |
| 默认值 | `"u16"` |
| 高级选项 | 否 |
| 允许覆盖 | 是 |

**枚举选项：**

| 值 | 标签 | 描述 | 推荐 |
|------|------|------|:---:|
| `"u16"` | 16-bit | Standard 16-bit integer | ★ |
| `"f32"` | 32-bit float | Floating-point for HDR processing | |

**Fluent 控件：** `<Dropdown>`

#### 参数: `half_size`

| 字段 | 值 |
|------|-----|
| 参数 ID | `half_size` |
| 标签 | Half Size |
| 描述 | Decode at half resolution for faster previews |
| 类型 | Boolean |
| 默认值 | `false` |
| 真标签 | "Half" |
| 假标签 | "Full" |
| 高级选项 | 否 |
| 允许覆盖 | 是 |

**Fluent 控件：** `<Switch>`
- 标签文字: 动态切换 "Full" / "Half"
- 默认显示: "Full" (false)

#### 参数: `apply_white_balance`

| 字段 | 值 |
|------|-----|
| 参数 ID | `apply_white_balance` |
| 标签 | White Balance |
| 描述 | Apply camera white balance during decode |
| 类型 | Boolean |
| 默认值 | `true` |
| 真标签 | "Apply" |
| 假标签 | "As-Shot" |
| 高级选项 | **是** |
| 允许覆盖 | 是 |

**Fluent 控件：** `<Switch>` + `[advanced]` 标记
- 标签文字: 动态切换 "Apply" / "As-Shot"
- 默认显示: "Apply" (true)
- 标签旁显示黄色 `advanced` 徽章

---

### 3.3 分区: dcraw Options

| 属性 | 值 |
|------|-----|
| 分区 ID | `dcraw_options` |
| 标题 | dcraw Options |
| 描述 | dcraw-specific settings |
| 样式 | CollapsibleCard |
| 默认折叠 | **是** |
| 高级 | 是 |

#### 参数: `dcraw_path`

| 字段 | 值 |
|------|-----|
| 参数 ID | `dcraw_path` |
| 标签 | dcraw Path |
| 描述 | Path to the dcraw binary |
| 类型 | String |
| 默认值 | `"dcraw"` |
| 占位符 | "/usr/bin/dcraw" |
| 最大长度 | 1024 |
| 高级选项 | **是** |
| 允许覆盖 | 是 |

**Fluent 控件：** `<Input>` + `[advanced]` 标记

#### 参数: `dcraw_extra_args`

| 字段 | 值 |
|------|-----|
| 参数 ID | `dcraw_extra_args` |
| 标签 | Extra Arguments |
| 描述 | Additional dcraw command-line arguments |
| 类型 | String |
| 默认值 | `""` (空) |
| 占位符 | "-H 2" |
| 最大长度 | 512 |
| 高级选项 | **是** |
| 允许覆盖 | 是 |

**Fluent 控件：** `<Input>` + `[advanced]` 标记

---

## 4. UI 布局结构

```
┌──────────────────────────────────────┐
│ Plugin Details · raw_input           │ ← sec-hdr
├──────────────────────────────────────┤
│ [All] [Template] [High ISO] [DSC_0034]│ ← ctx-bar (覆盖层级)
├──────────────────────────────────────┤
│ 📷 RAW Input                    v1.0 │ ← plugin-header
│ photopipeline.plugins.raw_input      │
│ [Input] [raw] [decode]               │ ← tags
│ FormatProcessor · RAM ≥ 512 MB        │ ← hw-info
├──────────────────────────────────────┤
│ Read RAW camera files (ARW, CR2...)  │ ← plugin-desc
├──────────────────────────────────────┤
│ ▼ RAW Format                   1 param│ ← acc-header
│  Decode Mode   [Auto ★ ▾]      ⬜   │ ← param-row
│  default: auto · allow_override: ✓   │ ← param-desc
├──────────────────────────────────────┤
│ ▼ Output                      3 params│
│  Pixel Format  [16-bit ★ ▾]   ⬜   │
│  Half Size     [Full ●――○]     ⬜   │ ← Switch
│  White Balance [Apply ●――○]   🟡   │ ← advanced + overridden
├──────────────────────────────────────┤
│ ▶ dcraw Options         collapsed    │ ← CollapsibleCard (默认折叠)
├──────────────────────────────────────┤
│ PREVIEW                              │
│ ┌──────────────────────────────────┐ │
│ │ PreviewMode::None               │ │ ← 虚线占位
│ └──────────────────────────────────┘ │
├──────────────────────────────────────┤
│ AUXILIARY VIEWS                      │
│ ┌──────────────────────────────────┐ │
│ │ vec![] — No auxiliary views     │ │ ← 虚线占位
│ └──────────────────────────────────┘ │
├──────────────────────────────────────┤
│ icon:camera color:#ef4444 min:320px  │ ← gui-footer
├──────────────────────────────────────┤
│ [🗑 Remove raw_input from Pipeline]  │ ← footer-area
└──────────────────────────────────────┘
```

---

## 5. 参数值 → 控件映射逻辑

前端从 `GetNodeSchema()` 接收到 JSON 格式的 ParameterSchema 后，按以下规则确定每个参数使用的 Fluent 控件：

```
parseParamField(field):
  match field.field_type:
    Enum(options) → <Dropdown>
      每个 option: "{label}{recommended ? ' ★' : ''} — {description}"
      默认选中 value == field.default 的项
    
    Boolean(label_true, label_false) → <Switch>
      label = value ? label_true : label_false
      默认状态: field.default == true ? checked : unchecked
    
    String(placeholder, max_length) → <Input>
      placeholder = field.placeholder
      maxLength = field.max_length
      默认值: field.default

  共同属性:
    标签显示: field.label + (field.advanced ? " [advanced]" : "")
    覆盖标记: 从覆盖系统查询 → ⬜ (inherited) / 🟡 (override)
    参数描述: 灰色小字 "default: <value> · allow_override: ✓"
```

## 6. 覆盖标记行为

| 上下文 | `raw_mode` | `apply_white_balance` | `dcraw_path` |
|--------|:--------:|:-------------------:|:----------:|
| Template | 可编辑 (定义) | 可编辑 (定义) | 可编辑 (定义) |
| Group | **⬜ 继承** | **🟡 覆盖 = false** | ⬜ 继承 |
| Image DSC_0034 | ⬜ 继承 Group | ⬜ 继承 Group (false) | ⬜ 继承 Template |

`apply_white_balance` 在 Group 层被覆盖为例：Group "High ISO" 将白平衡设为 `false` (As-Shot)，覆盖了 Template 层的 `true` (Apply)。

## 7. 与其他组件的交互

| 交互 | 触发 | 响应 |
|------|------|------|
| DAG 中点击 raw_input 节点 | 用户点击 | 右栏切换到本插件详情 |
| 底部插件卡片点击 raw_input | 用户点击 | DAG 中的 raw_input 节点高亮，右栏切换 |
| 双击/拖放 raw_input 卡片 | 用户操作 | 在 DAG 中新增 raw_input 节点 (此插件无输入端口) |
| 修改参数 | 用户编辑控件 | 参数值更新 → 覆盖标记变化 → 管线标记为 dirty |
| 点击 Remove | 用户点击 | 节点从 DAG 删除 → 右栏清空 |
| 切换上下文栏 | 用户点击 Template/Group/Image | 参数面板切换覆盖层级 → 继承/覆盖标记重新计算 |

## 8. 后端适配确认清单

- [x] 插件名称 "RAW Input" 与后端 `fn name()` 一致
- [x] 插件 ID `photopipeline.plugins.raw_input` 与 `fn id()` 一致
- [x] 版本 v1.0.0 与 `fn version()` 一致
- [x] 分类 Input 与 `PluginCategory::Input` 一致
- [x] 描述文本与 `fn description()` 一致
- [x] 标签 `input/raw/camera/decode` 与 `TAGS` 一致
- [x] 3 个分区 ID / 标题 / 样式与 GuiSchema 一致
- [x] 6 个参数的 field_id / label / type / default / description 与 ParameterSchema 一致
- [x] Enum 选项值与 ParameterType::Enum.options 一致
- [x] Boolean label_true/label_false 正确使用
- [x] advanced 标记与 ParameterField.advanced 一致
- [x] allow_override 与 ParameterField.allow_override 一致
- [x] supports_expression 与 ParameterField.supports_expression 一致
- [x] GuiSchema icon "camera" / color "#ef4444" / preview None / aux_views [] 全部对应

# 插件 #08: gps_set — 详细设计规格书

> 后端: `feat/unified-binary` · `crates/plugins/src/gps_set.rs`

---

## 1. 元数据

| 字段 | 值 |
|------|-----|
| 名称 | GPS Coordinate Manager |
| ID | `photopipeline.plugins.gps_set` |
| 分类 | Metadata |
| 描述 | Set GPS coordinates manually or interpolated from GPX track logs |
| 能力 | MetadataProcessor · 64 MB |
| 标签 | gps, coordinate, gpx, location, map |
| GuiSchema | icon=`"map-pin"`, color=`"#10b981"`(绿), preview=None, aux=**[Map]**, min=340px |

## 2. 参数

### GPS Source (Card)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `gps_mode` | Enum {Manual★, GPX Track, Clear} | `"manual"` | Dropdown |
| `gpx_file` | FilePath (*.gpx) | `""` | Input+📂 |

`gpx_file` 仅在 `mode = "gpx_track"` 时显示。

### Map Picker（辅助视图，在 Manual Coordinates 上方）

搜索并在地图上选点，结果自动填入下方坐标字段。

| 元素 | 内容 |
|------|------|
| 搜索框 | Input + Search 按钮 + 地图供应商选择 |
| 供应商 | 🇨🇳 **Amap**(高德, 默认), 🇨🇳 **Baidu**(百度), 🌐 **Google** |
| 结果列表 | 地点名 + 坐标, 点击选中 |
| 地图 | 130px 高, 📍标记 + 坐标标签, +/− 缩放 |
| 选址规则 | **中国境内** → Amap/Baidu · **中国境外** → Google |

交互: 搜索 → 选结果/点击地图 → lat/lon 自动填入下方字段。

### Manual Coordinates (Card, 由 Map Picker 填充)

| 参数 | 类型 | 默认 | 精度 | 控件 |
|------|------|------|------|:---:|
| `latitude` | Float (-90~90, step=1e-6) | 0.0 | 6位 | SpinButton(蓝边框=来自地图) |
| `longitude` | Float (-180~180, step=1e-6) | 0.0 | 6位 | SpinButton(蓝边框=来自地图) |
| `altitude` | Float (-500~9000, step=0.1) | 0.0 | 1位 | SpinButton(手工输入) |

仅 Altitude 手工输入（地图 API 通常不返回海拔）。

### GPX Options (CollapsibleCard, 折叠)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `time_offset_seconds` | Integer (-86400~86400) | 0 | SpinButton (s) |
| `max_interpolation_gap` | Integer (1~3600) | 300 | SpinButton (s) + `[advanced]` |

## 3. Map Picker 详细设计

```
┌─ Map Picker ───────────────────────────┐
│ [Search location...    ] [Search] [🇨🇳Amap▾]│
│ Results for "Chengdu":                   │
│ 📍 Chengdu, Sichuan, CN — 30.5728, 104.0668│ ← 选中(蓝色左边框)
│ 📍 Chengdu Panda Base — 30.7386, 104.1390 │
│ 📍 Tianfu Square — 30.6570, 104.0660       │
│                                          │
│ ┌──────────────────────────────────────┐ │
│ │           🗺 Map Area               │ │
│ │              📍                     │ │
│ │        30.5728, 104.0668            │ │
│ │                          [+] [-]   │ │
│ │ Amap © 2025                         │ │
│ └──────────────────────────────────────┘ │
│ Provider: Amap (China) | Click to place  │
└──────────────────────────────────────────┘
```

## 4. 数据流

```
搜索 "Chengdu" → Amap Geocoding API → 返回候选列表
用户选择/点击地图 → lat/lon 值 → 写入 latitude/longitude 字段
字段蓝色边框 = "来自地图自动填充"
用户手工修正 → 边框恢复灰色（表示已覆盖地图值）
```

## 5. 新增控件累计

| # | 插件 | 新增 |
|---|------|------|
| 08 | gps_set | **Map Picker** (搜索+选点+地图), 供应商切换规则 |

# 插件 #09: time_shift — 详细设计规格书

> 后端: `feat/unified-binary` · `crates/plugins/src/time_shift.rs`

---

## 1. 元数据

| 字段 | 值 |
|------|-----|
| 名称 | Time Shift |
| ID | `photopipeline.plugins.time_shift` |
| 分类 | Metadata |
| 描述 | Adjust DateTimeOriginal by hours, minutes, and seconds with timezone support |
| 标签 | time, date, timezone, shift, metadata |
| 能力 | MetadataProcessor · 64 MB |
| GuiSchema | icon=`"clock"`, color=`"#f59e0b"`(琥珀), preview=None, aux=[], min=320px |

## 2. 参数

### Time Adjustment (Card, 展开, 3 params)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `shift_hours` | Integer (-23~23) | 0 | SpinButton (h) |
| `shift_minutes` | Integer (-59~59) | 0 | SpinButton (min) |
| `shift_seconds` | Integer (-59~59) | 0 | SpinButton (s) |

**实时预览**: 参数变化时动态显示原始时间 → 偏移后时间:

```
Preview: 2025-03-15 18:42:30 → 2025-03-15 20:12:30 (+01:30:00)
```

### Timezone (Card, 展开, 2 params)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `source_timezone` | Enum (9选项) | `"UTC"`(★) | Dropdown |
| `target_timezone` | Enum (9选项) | `"local"`(★) | Dropdown |

时区列表: UTC★, Local, Asia/Shanghai, Asia/Tokyo, America/New_York, Europe/London, Europe/Paris, Australia/Sydney, Pacific/Auckland

**换算预览**: 底部显示完整换算链:

```
Result: 18:42 UTC → 02:42 CST (+08:00 TZ + 01:30 shift = 20:12 CST)
```

### Batch Options (CollapsibleCard, 默认折叠)

| 参数 | 类型 | 默认 | 控件 |
|------|------|------|:---:|
| `offset_mode` | Enum | `"fixed"` | Dropdown (fixed/incremental/filename) |
| `increment_seconds` | Integer | 60 | SpinButton (mode=incremental时) |

## 3. 预览设计

Live Preview 区域是 time_shift 的核心交互：

```
┌─ Preview ─────────────────────────────────────┐
│ 2025-03-15 18:42:30 → 2025-03-15 20:12:30     │
│ (+01:30:00)                                    │
└────────────────────────────────────────────────┘
```

当用户调整 h/min/s 或时区时，预览即时更新。每条已选图片生成对应的预览行。

## 4. 特点

- 无预览模式、无辅助视图 — 纯元数据操作
- SpinButton × 3 用于精确时间偏移
- 大 Dropdown × 2 用于 IANA 时区选择
- 实时时间预览反馈

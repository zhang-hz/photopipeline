# Candidate Files 面板 — 详细设计规格书

---

## 1. 组件层级

```
<Sidebar>
  <SidebarHeader />       — 标题 + 文件计数
  <SidebarToolbar />      — Import / Clear / To Batch
  <SortBar />             — 排序 + 缩略图大小
  <MultiSelectBar />      — 条件显示（≥2 选中时）
  <FilmstripList>         — 缩略图列表
    <ImageCard />   (×N)  — 单张图片卡片
    <DropZone />          — 拖放区域
  </FilmstripList>
  <EmptyState />          — 空态占位
  <GroupTree>             — 分组管理
    <GroupItem />  (×N)
    <GroupAdd />
  </GroupTree>
</Sidebar>
```

## 2. 各子组件

### 2.1 SidebarHeader

```
┌─ Candidate Files ──── 12 images ─┐
```

- 标题: 10px, fontWeight 600, uppercase, neutralFg3
- 计数: 9px, neutralFg4

### 2.2 SidebarToolbar

```
[📂 Import] [🗑 Clear] [📤 To Batch]
```

| 按钮 | 类型 | 交互 |
|------|:---:|------|
| Import | **primary** | 打开文件选择对话框，支持多选 |
| Clear | subtle | 清空所有图片 |
| To Batch | subtle | 将选中图片发送到批量队列 |

### 2.3 SortBar

```
Sort: [Name ▾]    Size: [S] [M] [L]
```

- Sort 下拉: Name / Size / Format / ISO
- S/M/L 按钮: 切换缩略图显示大小 (80/120/180px 加载尺寸)

### 2.4 MultiSelectBar

```
┌─ 📋 3 images selected │ +Group │ To Batch │ Clear ─┐
```

- **条件显示**: `selectedIndices.size > 1` 时出现
- 背景: rgba(213,153,0,0.06)
- 文字: warningFg

### 2.5 ImageCard

**3 种状态:**

| 状态 | 边框 | 勾选框 | 背景 |
|------|------|:---:|------|
| default | transparent | 隐藏 | neutralBg2 |
| hover | transparent | 隐藏 | neutralBg3 |
| single-selected | brandFg1 solid | 隐藏 | rgba(brand,0.06) |
| multi-selected | brandFg1 **dashed** | 显示 ✓ | rgba(brand,0.03) |

**布局:**

```
┌──────────────────────────────────┐
│ [✓] 🖼  DSC_0034.ARW             │  勾选框(18×18, 仅多选时显示)
│       6000×4000 ARW · ISO 6400    │
│       🟡 High ISO             24MB │  分组标签 + 文件大小
└──────────────────────────────────┘
```

**子元素:**

| 元素 | 规格 |
|------|------|
| 缩略图 | 50×34px, borderRadius 4px, 居中 emoji/图标 |
| 文件名 | 12px, fontWeight 500, 溢出省略号 |
| 元数据 | 10px, neutralFg4, "6000×4000 ARW · ISO 6400" |
| 分组标签 | 8px, warningBg + warningFg, 仅已分组时显示 |
| 文件大小 | 10px, neutralFg4, 右对齐 |

**交互:**

| 操作 | 行为 |
|------|------|
| 单击 | 单选（清除其他选择） |
| Ctrl+Click | 追加/取消单张 |
| Shift+Click | 范围选择 |
| 右键 | 弹出上下文菜单 |
| 拖拽到 Group | 加入分组 |
| 拖拽到 Batch | 加入批量队列 |

### 2.6 上下文菜单

```
┌──────────────────────┐
│ Open in Explorer     │
│ Copy Path            │
├──────────────────────┤
│ Select All     Ctrl+A│
│ Clear Selection Esc  │
│ Invert Selection     │
├──────────────────────┤
│ Add to Group →      │ → [High ISO, Night, + New Group]
│ Send to Batch       │
├──────────────────────┤
│ Remove         Del   │
└──────────────────────┘
```

### 2.7 EmptyState

当 `files.length === 0` 时显示:

```
┌──────────────────────┐
│                      │
│        📂            │
│   No images loaded   │
│ Click Import or drag │
│    files here        │
│                      │
└──────────────────────┘
```

### 2.8 DropZone

拖放时显示蓝色虚线区域:

```
┌──────────────────────┐
│ Drop images here     │
│   to import          │
└──────────────────────┘
```

### 2.9 GroupTree

```
┌─ Groups ───────────────────────┐
│ ● High ISO (≥1600)      4  ✎ 🗑│  hover 出现操作
│ ● Night (21-05)         3  ✎ 🗑│
│ ● GPS: Chengdu          5  ✎ 🗑│
│                                │
│ [+ Create Group…]              │  虚线按钮
│ [Auto-group ▾]                 │  虚线按钮
└────────────────────────────────┘
```

**GroupItem 交互:**

| 操作 | 行为 |
|------|------|
| 悬停 | 显示 ✎🗑 操作按钮 |
| ✎ | 打开编辑对话框（名称/条件/默认参数） |
| 🗑 | 确认 → 删除分组（保留图片覆盖到 Template） |
| 双击 | 选中该分组的所有图片 |

**Auto-group 菜单:**
- By ISO Range…
- By GPS Cluster…
- By Time Interval…
- By Camera Model…

**创建分组对话框:**

```
Create Group
┌──────────────────────────────┐
│ Name: [________________]     │
│ Condition (optional):        │
│  Field: [ISO ▾]              │
│  Op:    [≥ ▾]                │
│  Value: [1600___]            │
│                              │
│ Default overrides:           │
│  [Add parameter override ▾] │
│                              │
│       [Cancel]  [Create]     │
└──────────────────────────────┘
```

## 3. 键盘快捷键

| 快捷键 | 行为 |
|--------|------|
| `Ctrl+A` | 全选 |
| `Escape` | 清除选择 |
| `Delete` | 删除选中图片 |
| `Ctrl+Click` | 追加选择 |
| `Shift+Click` | 范围选择 |

## 4. 数据流

```
用户 Import → 文件对话框 → ImageService.Load() → 获取元数据
  → ImageService.GetThumbnail() → 生成缩略图
  → FilmstripList 渲染 ImageCard

用户选中图片 → selectedIndices 更新
  → 单选 → 触发 Preview 更新
  → 多选 → MultiSelectBar 出现 → 右栏多图模式

用户发送 To Batch → BatchViewModel.BatchQueue ← 选中图片
```

## 5. 状态覆盖

| 状态 | 条件 | 显示 |
|------|------|------|
| 空态 | files=0 | EmptyState |
| 加载中 | importing | Import 按钮 disabled + Spinner |
| 正常 | files>0 | ImageCard 列表 |
| 多选 | selected>1 | MultiSelectBar + 虚线边框 |
| 拖放悬停 | dragover | DropZone 蓝色虚线 |

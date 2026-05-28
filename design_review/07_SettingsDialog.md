# GUI 设计审查 — #07 SettingsDialog 设置对话框

---

## 一、界面设计（UI Design）

### 1.1 设计文档定义

ARCHITECTURE.md 和 README 中未详细规定设置对话框的具体内容。从功能倒推，设置对话框应涵盖：
- 主题切换（亮/暗）
- 后端服务器配置（路径/端口/自动启动）
- 输出默认值（格式/目录/质量/元数据）
- 缩略图尺寸
- 窗口位置/尺寸

### 1.2 实际实现的界面

SettingsDialog.xaml 实现：

```
┌──────────────────────────────────────┐
│ Settings                         — □ ✕ │
├──────────────────────────────────────┤
│ ┌─ General ──┬── Backend ─┬─ Output ┐│
│ │             │            │         ││
│ │ Theme       │ Server Path│Default  ││
│ │ [Dark ▼]   │ [________] │Format   ││
│ │             │ [...]      │[PNG ▼] ││
│ │ Max Recent │            │         ││
│ │ Files       │ Port       │Default  ││
│ │ [10   ▲▼]  │ [50051 ▲ ] │Output   ││
│ │             │            │Directory││
│ │             │ Auto-start │[______] ││
│ │             │ [Toggle ON]│ [...]   ││
│ │             │            │         ││
│ │             │            │JPEG Qual││
│ │             │            │[===○=]80││
│ │             │            │         ││
│ │             │            │Embed    ││
│ │             │            │Metadata ││
│ │             │            │[Toggle] ││
│ │             │            │         ││
│ │             │            │Thumbnail││
│ │             │            │[120▲▼] ││
│ └─────────────┴────────────┴─────────┘│
│                          [Cancel][Reset][Save]│
└──────────────────────────────────────┘
```

**布局结构：**
- TabControl 三标签页：General / Backend / Output
- 固定大小：520×440，不可调整，CenterOwner 启动位置
- 底部操作栏：Cancel / Reset / Save 三个按钮

**各标签页内容：**

**General：**
- Theme：ComboBox（绑定 Themes → {"Dark", "Light"}）
- Max Recent Files：NumberBox（5-50）

**Backend：**
- Server Path：TextBox + 浏览按钮(...)
- Port：NumberBox（1024-65535）
- Auto-start：ToggleSwitch

**Output：**
- Default Format：ComboBox（绑定 OutputFormats）
- Default Output Directory：TextBox + 浏览按钮(...)
- JPEG Quality：Slider（0-100）+ 百分比显示
- Embed Metadata：ToggleSwitch
- Thumbnail Size：NumberBox（64-512）

### 1.3 视觉元素

| 元素 | 实际实现 | 评估 |
|------|---------|:----:|
| 标签页 | TabControl + TabItem | ✅ |
| 输入控件 | TextBox / NumberBox / ComboBox / ToggleSwitch / Slider | ✅ |
| 标签 | TextBlock + 二级文字色 | ✅ |
| 按钮 | Cancel（Secondary）/ Reset（Secondary）/ Save（Accent） | ✅ |
| 固定宽按钮 | 3 个按钮均 80px 宽 | ✅ |
| 堆版面 MaxWidth | 360px 限制保持可读性 | ✅ |
| 窗口固定 | 520×440，NoResize | ⚠️ 不可调整大小 |

### 1.4 状态覆盖

| 状态 | 实际实现 | 评估 |
|------|---------|:----:|
| **新建态** | 加载当前设置 | ✅ |
| **编辑态** | 各控件直接编辑 | ✅ |
| **取消** | 恢复快照（代码后置 SnapshotCurrent → OnClosed 恢复） | ✅ |
| **保存错误** | MessageBox 错误提示 | ✅ |
| **重置确认** | MessageBox Yes/No 确认 | ✅ |
| **重置执行** | ResetCommand.ExecuteAsync → 恢复默认值 | ✅ |

---

## 二、功能设计（Functional Design）

### 2.1 职责边界

SettingsViewModel + SettingsDialog 负责：
- 提供应用设置的编辑界面
- 主题选择
- 后端服务器配置
- 输出默认值配置
- 设置持久化（JSON 文件，通过 SettingsService）
- 取消时恢复设置快照

### 2.2 数据流向

```
用户打开设置 → SettingsDialog 构造
  → SettingsViewModel 加载当前设置
  → SnapshotCurrent() 保存快照

用户编辑 → 直接绑定 ViewModel 属性

用户点击 Save → SaveCommand.ExecuteAsync()
  → SettingsService.SaveAsync()
  → ApplyTheme() → 全局主题变更 → Close()

用户点击 Cancel → Close() → OnClosed → LoadFrom(_snapshot)
  → 恢复原始值

用户点击 Reset → 确认对话框
  → ResetCommand.ExecuteAsync() → 默认值
```

### 2.3 用户操作流程

| 步骤 | 操作 | 系统响应 |
|------|------|---------|
| 1 | 点击主窗口设置按钮 | ShowSettingsCommand → 打开 dialog |
| 2 | 在各标签页编辑 | 实时绑定更新 VM 属性 |
| 3a | 点击 Save | 持久化 + 应用主题 → 关闭 |
| 3b | 点击 Cancel | 恢复快照 → 关闭 |
| 3c | 点击 Reset | 确认 → 恢复默认值 |

### 2.4 与其他组件的协作

| 协作方 | 关系 | 实现 |
|-------|------|------|
| SettingsService | 设置持久化 | JSON 文件读写 |
| App（ApplyTheme） | 主题应用 | App.ApplyTheme() 静态方法 |
| MainViewModel | 窗口状态（未联动） | 窗口位置/尺寸未设置到 Settings |

---

## 三、实现程度（Implementation Assessment）

### 3.1 已实现

- ✅ 3 标签页（General / Backend / Output）
- ✅ 主题选择（Dark/Light ComboBox）
- ✅ 后端服务器路径/端口/自动启动
- ✅ 默认输出格式/目录/质量/元数据
- ✅ 缩略图尺寸
- ✅ 最大最近文件数
- ✅ 保存/恢复设置（SettingsService）
- ✅ 取消恢复快照（OnClosed 回滚）
- ✅ 重置确认对话框 + 默认值恢复

### 3.2 未实现

| 缺失项 | 设计依据 | 严重度 |
|-------|---------|:-----:|
| ❌ **窗口位置/尺寸设置** | AppSettings 有字段但 SettingsDialog 无 UI | 📝 功能缺失 |
| ❌ **后端健康检查/状态显示** | UX 可用性 | 📝 体验缺失 |
| ❌ **快捷键配置** | 高级功能 | 📝 功能缺失 |
| ❌ **语言/区域设置** | 跨平台应用标准 | 📝 功能缺失 |

### 3.3 MVVM 模式问题

| 问题 | 描述 | 影响 |
|------|------|------|
| 按钮使用 Click 事件 | Save/Cancel/Reset 使用代码后置事件处理器，而非 Command 绑定 | ⚠️ 违反 MVVM 模式 |
| Cancel 无 ViewModel 命令 | 取消逻辑完全在代码后置 | ⚠️ 不可测试 |
| 快照在代码后置 | 快照的创建和恢复绕过 ViewModel | ⚠️ 不可测试 |

### 3.4 完成度评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 设置覆盖范围 | 80% | 主题/后端/输出配置完整；缺窗口位置/快捷键等 |
| UI 布局 | 90% | 标签页分类清晰，布局规整 |
| MVVM 一致性 | 50% | 绑定 OK 但按钮事件通过代码后置 |
| 状态管理 | 90% | 保存/取消/重置流程完整 |
| 视觉设计 | 80% | 简洁，但窗口固定大小不可调 |
| **综合** | **80%** | **功能完整，MVVM模式有瑕疵但功能无缺失** |

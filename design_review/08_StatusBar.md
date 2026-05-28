# GUI 设计审查 — #08 StatusBar 状态栏

---

## 一、界面设计（UI Design）

### 1.1 设计文档定义

ARCHITECTURE.md 第 8.1 节的布局图中，底部区域称为"信息栏"（位于中栏底部）：

```
┌──────────────────────┐
│     信息栏            │
└──────────────────────┘
```

设计文档未详细说明信息栏的具体内容。从功能和布局推断，信息栏应为全局状态信息展示区域。

### 1.2 实际实现的界面

MainWindow.xaml 中底部状态栏：

```
┌──────────────────────────────────────────────────┐
│ Ready           [SnackbarPresenter]  Backend: 🟢 Connected│
└──────────────────────────────────────────────────┘
```

**布局结构（3 列）：**
1. 左列（Width=Auto）：StatusMessage（TextBlock，12px，二级文字色）
2. 中列（Width=*）：SnackbarPresenter（WPF-UI 通知控件，8px 边距）
3. 右列（Width=Auto）：后端状态指示器
   - Ellipse（8×8px）— 绿色（SystemFillColorSuccessBrush）或红色（SystemFillColorCriticalBrush）
   - "Backend:" + BackendStatus 文字

**视觉风格：**
- Border 背景：CardBackgroundFillColorDefaultBrush
- Padding：10,6
- 上边框：DividerStrokeColorDefaultBrush，1px
- 后端状态圆点：DataTrigger 绑定 IsBackendHealthy → True=绿色 / False=红色

### 1.3 状态覆盖

| 状态 | 实际实现 | 评估 |
|------|---------|:----:|
| **正常态** | 绿色圆点 + "Connected" | ✅ |
| **后端断连** | 红色圆点 + "Disconnected"/"Reconnecting..." | ✅ |
| **启动中** | 灰色圆点（默认状态）+ "Starting..." | ✅ |
| **通知消息** | SnackbarPresenter 弹出通知 | ✅ |
| **状态消息** | StatusMessage TextBlock 更新 | ✅ |

---

## 二、功能设计（Functional Design）

### 2.1 职责边界

状态栏是全局信息汇聚点，整合来自各子系统的状态信息。

### 2.2 数据流向

```
后端状态变化 → MainViewModel._isBackendHealthy + _backendStatus
  → DataTrigger → 圆点颜色 + 状态文字

子 VM 状态消息 → PropertyChanged 事件 → MainViewModel.SubscribeChildStatusMessages()
  → StatusMessage 属性更新

通知事件 → MainViewModel.ShowNotification() → SnbarPresenter.Show()
```

### 2.3 与 MainWindow 的关系

状态栏是 MainWindow.xaml 的直接部分，不独立封装为 UserControl。

---

## 三、实现程度（Implementation Assessment）

### 3.1 已实现

- ✅ 全局状态消息显示（StatusMessage TextBlock）
- ✅ 后端健康状态（绿色/红色圆点 + 文字）
- ✅ SnackbarPresenter 通知
- ✅ 子 VM 状态消息自动转发

### 3.2 未实现

| 缺失项 | 设计依据 | 严重度 |
|-------|---------|:-----:|
| ❌ **进度信息** | 设计文档"信息栏"预期更丰富 | 📝 功能简约 |
| ❌ **后端手动重连按钮** | UX 标准 | 📝 交互缺失 |
| ❌ **状态图标/MessageBox 集成** | UX 标准 | 📝 视觉简约 |

### 3.3 完成度评分

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 状态消息 | 100% | 消息显示 + 跨 VM 转发完整 |
| 后端健康指示 | 100% | 颜色变化 + 状态文字 |
| 通知 | 100% | SnackbarPresenter 可用 |
| 交互性 | 30% | 纯展示，无可点击操作 |
| 视觉设计 | 80% | 简洁但信息密度低 |
| **综合** | **85%** | **功能完整但简约，符合状态栏应有角色** |

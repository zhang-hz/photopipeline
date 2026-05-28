# DAG Pipeline Editor — 详细设计规格书

---

## 1. 组件

```
<PipelineEditor>
  <DAGToolbar />      — New/Save/Load/Validate/Run
  <DAGCanvas>         — 画布
    <DAGEdge /> ×N    — 贝塞尔曲线连线
    <DAGNode /> ×N    — 节点
    <MiniMap />       — 迷你地图
    <DropHint />      — 拖放提示
  </DAGCanvas>
  <PluginBrowser />   — 底部插件选择横条 (已单独设计)
</PipelineEditor>
```

## 2. DAGNode

```
┌──────────────────────┐
│ auto · 12 files      │ ← 徽章 (可选)
│ raw_input            │ ← 插件名 (fontWeight 600)
│ Input                │ ← 类型 (9px, neutralFg4)
│           ⊡          │ ← 输出端口 (蓝色方块)
│     ⊡                │ ← 输入端口 (绿色方块)
└──────────────────────┘
```

**3 种状态:** default (s2边框) / hover (fg4边框) / selected (brand边框+发光)

**端口:** 输入 ⊡ 绿色(successFg) / 输出 ⊡ 蓝色(brandFg1)，hover 放大 1.3x

## 3. 交互

| 操作 | 行为 |
|------|------|
| 单击节点 | 选中 → 右栏切换参数面板 |
| 拖拽节点 | 移动位置 |
| 拖拽输出端口 | 画虚线跟随光标 → 放到输入端口 = 创建连线 |
| 右键节点 | Copy/Duplicate/Disable/Delete |
| 右键画布 | Add Node 菜单 |
| Del 键 | 删除选中节点 |
| Ctrl+D | 复制选中节点 |
| Ctrl+Z/Y | 撤消/重做 |

## 4. 连线 (DAGEdge)

- 贝塞尔曲线 (C 命令)
- 颜色 brandFg1, 2px, opacity 0.55
- 环检测: 创建连线时检查 → 拒绝循环边

## 5. MiniMap

- 128×84px, 右下角
- 蓝色视口矩形
- 点击跳转 · 拖拽平移

## 6. 添加节点 3 种方式

| 方式 | 操作 |
|------|------|
| 拖放 | 从底部插件横条拖入 DAG 画布 |
| 双击 | 双击底部插件卡片 → 自动添加到末尾 |
| 右键 | 右键画布空白 → Add Node → 选择插件 |

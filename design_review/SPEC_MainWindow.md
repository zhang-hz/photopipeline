# Photopipeline 主界面设计规格书

> **设计系统**: Fluent 2 · **框架**: React 19 + @fluentui/react-components v9  
> **主题**: Dark · **基准分辨率**: 1500×920 · **最小分辨率**: 1280×720

---

## 1. 整体布局

```
┌──────────────────────────────────────────────────────────────────┐
│ TitleBar                                     h=44px              │
├───────────────┬────────────────────────────┬─────────────────────┤
│ Sidebar       │ Content                    │ Panel               │
│ w=272px       │ flex=1, min=400px          │ w=440px             │
│               │                            │                     │
│ Candidate     │ Pipeline Editor            │ Plugin Control      │
│ Files         │                            │ Panel               │
│               │                            │                     │
├───────────────┴────────────────────────────┴─────────────────────┤
│ StatusBar                                    h=36px              │
└──────────────────────────────────────────────────────────────────┘
```

| 区域 | 宽度 | 高度 | 说明 |
|------|:---:|:---:|------|
| TitleBar | 100% | 44px | Fluent 2 自定义标题栏 |
| Sidebar | 272px | flex | 固定宽度，可拖拽调整 |
| Content | flex:1 | flex | 最小 400px |
| Panel | 440px | flex | 固定宽度，可拖拽调整 |
| StatusBar | 100% | 36px | 全局状态信息 |

### Fluent 2 Token 引用

| Token | 值 | 用途 |
|-------|-----|------|
| `borderRadiusXLarge` | 12px | 窗口外框圆角 |
| `borderRadiusLarge` | 8px | 卡片、画布区域圆角 |
| `borderRadiusMedium` | 4px | 按钮、输入框圆角 |
| `borderRadiusSmall` | 2px | 徽章、小标记圆角 |
| `strokeThin` | 1px | 常规边框 |
| `strokeThick` | 2px | 聚焦/选中边框 |
| `spacingS/M/L` | 8/12/16px | 组件间距 |
| `neutralBg1` | #141414 | Sidebar 背景 |
| `neutralBg2` | #1f1f1f | Content/Panel 背景 |
| `neutralBg3` | #292929 | Hover 态 |
| `neutralStroke1` | #383838 | 分隔线 |

---

## 2. 组件树

```
<FluentProvider theme={webDarkTheme}>
  <App>
    <TitleBar />
    <MainLayout>
      <Sidebar>
        <SidebarHeader />
        <SidebarToolbar />
        <MultiSelectBar />
        <FilmstripList>
          <ImageCard />  (×N)
        </FilmstripList>
        <GroupTree>
          <GroupItem />  (×N)
          <GroupAdd />
        </GroupTree>
      </Sidebar>
      
      <Content>
        <ContentHeader />
        <DAGToolbar />
        <DAGCanvas>
          <DAGNode />    (×N)
          <DAGEdge />    (×N)
          <MiniMap />
        </DAGCanvas>
      </Content>
      
      <Panel>
        <ContextBar />
        <PluginBrowser>
          <PluginSearch />
          <PluginGrid>
            <PluginCard />  (×N)
          </PluginGrid>
        </PluginBrowser>
        <ControlPanel>
          <ParamSection />  (×N)
          <ExpressionEditor />
          <AuxView />
        </ControlPanel>
        <RemoveButton />
      </Panel>
    </MainLayout>
    <StatusBar />
  </App>
</FluentProvider>
```

---

## 3. 各组件规格

### 3.1 TitleBar

```
┌──────────────────────────────────────────────────────────────────┐
│ ◆ Photopipeline — HDR Pipeline v1                    ◐    ⚙    │
└──────────────────────────────────────────────────────────────────┘
```

| 属性 | 值 |
|------|-----|
| 高度 | 44px |
| 背景 | `linear-gradient(180deg, rgba(255,255,255,0.02) → transparent)` |
| 拖拽区域 | 整条可拖拽（`-webkit-app-region: drag`） |
| 按钮 | 不可拖拽（`no-drag`），hover 显示 neutralBg4 背景 |

**子元素：**

| 元素 | 位置 | 规格 |
|------|------|------|
| Logo | 左 | 22×22px, brandFg1 背景, borderRadiusMedium, 白色文字 |
| 标题 | 左 | fontSizeBody1 (12px), fontWeight 600 |
| 主题按钮 | 右 | 32×32px, borderRadiusMedium, hover: neutralBg3 |
| 设置按钮 | 右 | 同上 |

---

### 3.2 Sidebar

#### 3.2.1 SidebarHeader

```
┌──────────────────────────┐
│ CANDIDATE FILES   12 images │
└──────────────────────────┘
```

- 高度: 32px, padding: 8px 16px
- 字体: fontSizeCaption1 (10px), fontWeight 600, neutralFg3
- 文字: 大写字母, letterSpacing 0.6px
- 右侧计数: fontSizeCaption1, neutralFg4

#### 3.2.2 SidebarToolbar

```
┌──────────────────────────┐
│ 📂 Import │ 🗑 Clear │ 📤 Batch │
└──────────────────────────┘
```

- padding: 8px 12px
- 按钮: fui-Button--small (24px 高)
- Import: fui-Button--primary
- 其余: fui-Button--subtle

#### 3.2.3 MultiSelectBar

```
┌──────────────────────────┐
│ 📋 3 selected │ +Group │ To Batch │ Clear │
└──────────────────────────┘
```

- 默认隐藏（`display: none`），有复数选中时显示
- 背景: rgba(213,153,0,0.06)
- 文字: warningFg (#d59900)
- 按钮: fui-Button--subtle--small

#### 3.2.4 ImageCard

```
普通状态:                      选中状态:                      多选状态:
┌────────────────────────┐   ┌────────────────────────┐   ┌─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐
│ ▓▓▓▓  DSC_0034.ARW     │   │ ▓▓▓▓  DSC_0034.ARW     │   │ ✓ ▓▓▓▓  PANO_001.DNG  │
│ ▓▓▓▓  6000×4000 ARW    │   │ ▓▓▓▓  6000×4000 ARW    │   │   ▓▓▓▓  8256×5504 DNG  │
│       🟡 High ISO  24MB │   │       🟡 High ISO  24MB │   │                   45MB │
└────────────────────────┘   └────────────────────────┘   └─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘
  默认透明边框                  蓝色实线边框                   蓝色虚线边框 + 勾选
```

**状态定义：**

| 状态 | 边框 | 背景 | 勾选框 |
|------|------|------|:-----:|
| default | transparent | neutralBg2 | 隐藏 |
| hover | transparent | neutralBg3 | 隐藏 |
| selected | brandFg1, solid | rgba(brand,0.06) | 隐藏 |
| multi-selected | brandFg1, dashed | rgba(brand,0.03) | 显示 |

**子元素：**

| 元素 | 规格 |
|------|------|
| 缩略图 | 52×36px, borderRadiusMedium (4px), neutralBg3 背景 |
| 文件名 | fontSizeBody1 (12px), fontWeight 500, 溢出省略号 |
| 元数据 | fontSizeCaption1 (10px), neutralFg4, "6000×4000 · ARW · ISO 6400" |
| 分组标签 | 仅在已分组时显示, warningBg 背景, warningFg 文字, borderRadiusSmall |
| 文件大小 | fontSizeCaption1, neutralFg4, 右对齐 |
| 勾选框 | 多选时显示, 18×18px, borderRadiusSmall (2px), brandFg1 背景, 白色 ✓ |

#### 3.2.5 GroupTree

```
┌──────────────────────────┐
│ GROUPS                   │
│ ● High ISO (≥1600)    4  │  悬停出现 ✎ 🗑
│ ● Night (21-05)       3  │
│ ● GPS: Chengdu        5  │
│                          │
│ + Create Group…          │  虚线边框按钮
│ Auto-group ▾             │  虚线边框按钮
└──────────────────────────┘
```

**GroupItem 状态：**

| 状态 | 行为 |
|------|------|
| default | 显示名称 + 计数徽章 |
| hover | neutralBg3 背景, 显示 ✎🗑 操作按钮 |
| ✎ 点击 | 打开编辑对话框（名称/条件/默认参数） |
| 🗑 点击 | 确认对话框："将影响X张图片的覆盖参数" |

**Auto-group 下拉菜单项：**
- By ISO Range…
- By GPS Cluster…
- By Time Interval…
- By Camera Model…

---

### 3.3 Content (Pipeline Editor)

#### 3.3.1 ContentHeader

```
┌──────────────────────────────────────────┐
│ ● Pipeline Editor         5 nodes · 120% │
└──────────────────────────────────────────┘
```

- 左侧状态灯: 8px 圆形, successFg (绿色)
- 标题: fontSizeCaption1 (10px), 大写, fontWeight 600
- 右侧: 节点数 + 缩放比例

#### 3.3.2 DAGToolbar

```
┌──────────────────────────────────────────┐
│ 📄New 💾Save 📂Load │ ✓Validate │ ▶Run ⏹Cancel │ 🔍+ 🔍− ⊞Fit │
└──────────────────────────────────────────┘
```

| 按钮 | Fluent 组件 | appearance |
|------|-----------|-----------|
| New, Save, Load | Button | subtle |
| Validate | Button | subtle |
| Run | Button | **primary** |
| Cancel | Button | subtle |
| Zoom+/-, Fit | Button | subtle (icon-only) |

分隔符: 1px × 20px, neutralStroke1

#### 3.3.3 DAGCanvas

**画布：**
- 背景: neutralBg1 (#141414)
- 网格: 32px, rgba(255,255,255,0.015)
- 缩放: Ctrl+滚轮, 按钮 ±/Fit
- 平移: Space+拖拽 / 中键拖拽
- 右键空白: 弹出 Add Node 菜单

**DAGNode：**

```
┌──────────────────┐
│ auto · 12 files  │  ← 徽章 (仅 auto 节点)
│                  │
│ raw_input        │  ← 插件名 (fontWeight 600)
│ Input            │  ← 分类 (caption1, neutralFg4)
│            ⊡     │  ← 输出端口 (蓝色方块)
└──────────────────┘
```

| 状态 | 边框 | 阴影 |
|------|------|------|
| default | neutralStroke2, 1.5px | shadow4 |
| hover | neutralFg4 | shadow4 |
| selected | brandFg1, 1.5px | 0 0 0 3px rgba(brand,0.15) + shadow8 |

**端口：**

| 类型 | 形状 | 颜色 | 位置 |
|------|------|------|------|
| Input | 14×14px, borderRadiusSmall (方块) | successFg 边框 | 左侧居中 |
| Output | 14×14px, borderRadiusSmall (方块) | brandFg1 边框 | 右侧居中 |

**连线操作：**
- 从输出端口拖拽 → 显示虚线跟随光标
- 放到输入端口 → 创建贝塞尔曲线
- 悬停连线 → 加粗 + 高亮 (opacity 0.55→0.9)
- 选中连线 → 右键可删除

**MiniMap：**
- 128×84px, 右下角固定
- 半透明背景 + 蓝色视口矩形
- 点击跳转视口, 拖拽平移视口

**右键菜单：**

| 右键目标 | 菜单项 |
|---------|--------|
| 画布空白 | Add Node → [插件列表] / Paste (Ctrl+V) |
| 节点 | Copy / Duplicate / Disconnect / Disable / Delete |
| 连线 | Delete Edge |

---

### 3.4 Panel (Plugin Control Panel)

#### 3.4.1 ContextBar

```
┌──────────────────────────────────────────────┐
│ All  │  Template  │  High ISO  │  DSC_0034   │
└──────────────────────────────────────────────┘
```

| 状态 | 样式 |
|------|------|
| default | neutralFg2, 透明底, transparent 底边 |
| hover | neutralFg1, neutralBg3 底 |
| active | brandFg1, brandFg1 底边 (2px 粗) |

交互: 点击切换编辑上下文 (All → Template → Group → Image)

#### 3.4.2 PluginBrowser

```
┌──────────────────────────────────────────────┐
│ PLUGINS                                      │
│ 🔍 Search plugins...              [All ▾]    │
│                                              │
│ ┌──────────┬──────────┬──────────┐          │
│ │ raw_input│ transform│colorspace│          │  3列网格
│ │ Input    │Transform │ Color    │          │  gap: 4px
│ ├──────────┼──────────┼──────────┤          │
│ │ lut3d    │ai_denoise│lens_corr │          │
│ │ Color    │ Enhance  │ Correct  │          │
│ └──────────┴──────────┴──────────┘          │
└──────────────────────────────────────────────┘
```

**PluginCard 状态：**

| 状态 | 边框 | 背景 |
|------|------|------|
| default | transparent | neutralBg2 |
| hover | neutralStroke1 | neutralBg3 |
| selected (匹配DAG选中节点) | brandFg1 | rgba(brand,0.06) |

**双击 / 拖放 → 添加到 DAG 画布**

#### 3.4.3 ControlPanel

**参数分区 (AccordionItem)：**

```
展开状态:                        折叠状态:
┌──────────────────────────┐   ┌──────────────────────────┐
│ ▼ Resize      inherited  │   │ ▶ Crop        inherited  │
├──────────────────────────┤   └──────────────────────────┘
│       Width: ═══○═══   │
│              1920 px  ⬜  │
│  Inherited from Template  │
│                          │
│      Height: ═══○═══   │
│              1080 px  ⬜  │
│                          │
│      Filter: Lanczos3 ▼ ⬜ │
└──────────────────────────┘
```

**覆盖状态徽章：**

| 徽章 | 颜色 | 含义 |
|------|------|------|
| `inherited` | neutral | 本层无覆盖，全部继承上级 |
| `N overrides` | warning | N 个参数在本层被覆盖 |
| `values vary` | warning | 多选时值不一致 |

**参数行 (ParamRow) 布局：**

```
Label (80px, 右对齐) | 控件 (flex) | 值 (mono) | 单位 | 覆盖标记
─────────────────────────────────────────────────────
Width                 ═══○═══      1920      px     ⬜
```

**覆盖标记交互：**

| 标记 | 当前状态 | 点击行为 |
|:---:|------|---------|
| ⬜ | 继承 | 激活编辑 → 变为 🟡 |
| 🟡 | 覆盖 | 显示 × 恢复按钮 |
| 🔵 | 表达式 | 双击打开表达式编辑器 |

**控件类型映射 (根据 ParameterSchema.ValueType)：**

| ValueType | Fluent 组件 |
|-----------|-----------|
| Integer | `<Input type="number">` 或 `<Slider>` (有 min/max 时) |
| Float | `<Slider>` (有 min/max/step) 或 `<Input>` |
| Boolean | `<Switch>` |
| Enum | `<Dropdown>` + `<Option>` |
| String | `<Input>` |
| FilePath | `<Input>` + `<Button icon={<FolderOpen/>}>` |
| Color | 自定义 ColorPicker |
| Coordinate | 双 Input (lat/lon) |

#### 3.4.4 ExpressionEditor

```
┌──────────────────────────────┐
│ 𝑓 Expression                 │
│ clamp(iso / 12800, 0, 1)     │
│                              │
│ DSC_0034: 6400→0.50          │
│ night_012: 3200→0.25         │
└──────────────────────────────┘
```

- 仅在选中节点且 `supports_expression=true` 时显示
- 紫色主题 (#b084f4)
- 可用变量列表: `iso`, `aperture`, `shutter`, `focal_length`, `ev`, `filename`
- 实时预览: 对每张选中的图片计算表达式结果

#### 3.4.5 AuxView

```
┌──────────────────────────────┐
│ HISTOGRAM                    │
│ ▂▃▅▆▇▆▅▃▂▁                  │
│                              │
│ WAVEFORM                     │
│ (仅当 GuiSchema.aux_views    │
│  包含对应类型时显示)          │
└──────────────────────────────┘
```

- 根据 GuiSchema.aux_views 动态渲染
- 支持类型: Histogram, Waveform, Vectorscope, GamutDiagram, Map, FocusPeaking

#### 3.4.6 RemoveButton

```
┌──────────────────────────────┐
│ 🗑 Remove from Pipeline       │
└──────────────────────────────┘
```

- fui-Button--danger, dangerFg 边框
- 全宽
- 点击 → 节点从 DAG 中移除, 相关连线自动删除

---

### 3.5 StatusBar

```
┌──────────────────────────────────────────────────────────────────┐
│ ▶ Batch: 8/12  ██████████░░░  65%  12 done  3 failed              │
│ ⏱ 00:03:15  ~00:01:20  12 img/min  │  Mem:512MB · GPU:Ready  │ ● Connected │
└──────────────────────────────────────────────────────────────────┘
```

| 元素 | 规格 |
|------|------|
| 高度 | 36px |
| 批量指示灯 | ▶ 播放图标, fontSize 16px |
| ProgressBar | flex:1 最大 180px, 4px 高, brandFg1 填充 |
| 计数 | "12 done" neutralFg2, "3 failed" dangerFg |
| 后端状态 | successFg 绿点 + "Connected" |

---

## 4. 交互流程

### 4.1 导入图片 → 创建管线

```
用户点击 Import
  → 文件选择对话框
  → 图片加入 FilmstripList
  → DAG 自动创建 raw_input 节点 (标记 "auto · 12 files")
  → ImageCard 显示缩略图 + 元数据
  → 自动分组 (如启用)
```

### 4.2 添加插件到管线

```
方式1: 拖放
  PluginCard → 拖拽到 DAGCanvas → 放开 → DAGNode 创建

方式2: 双击
  双击 PluginCard → DAGNode 自动添加到末尾

方式3: 右键菜单
  右键 DAGCanvas → Add Node → 选择插件 → DAGNode 创建
```

### 4.3 编辑参数 → 执行管线

```
选中 DAGNode
  → Panel 切换到该节点参数
  → ContextBar 显示当前覆盖层级
  → 调整参数 → 覆盖标记变化: ⬜→🟡
  
点击 ▶ Run
  → DAGToolbar 状态: Run 禁用, Cancel 显示
  → 各 DAGNode 状态灯: 绿色 = OK, 闪烁 = 执行中
  → StatusBar 进度更新
  → 完成 → Run 恢复
```

### 4.4 参数覆盖交互

```
选择 Template 上下文
  → 所有参数可编辑 (这是定义层)
  → 修改值 → 影响所有继承该值的作用域

选择 Group 上下文  
  → 继承值显示 ⬜ (只读)
  → 点击 ⬜ → 激活编辑 → 显示 🟡
  → 修改值 → 仅影响该分组内的图片

选择 Image 上下文 (单选)
  → 显示 "当前值" + "来源" 灰色小字
  → 来源可能是 Template / Group / Image (override)

选择 Image 上下文 (多选)
  → 值一致的显示 ⬜
  → 值不一致的显示 🟡 + "values vary"
  → 提供 "Unify to value" 和 "Clear overrides" 批量操作
```

---

## 5. 状态覆盖矩阵

| 状态 | Sidebar | Content | Panel | StatusBar |
|------|---------|---------|-------|-----------|
| **空态 (无图片)** | 空列表, 显示占位提示 | 空画布, "Add images to begin" | 插件浏览器可浏览 | 后端状态 |
| **加载中** | Import 按钮禁用, Spinner | 缩略图加载中 | — | — |
| **编辑中** | 列表正常 | DAG 可编辑 | 参数可编辑 | 批量空闲 |
| **执行中** | 正常 | Run 禁用, Cancel 显示, 节点闪烁 | 参数只读 | 进度条更新 |
| **暂停** | 正常 | Run 恢复 | 参数只读 | 暂停指示 |
| **错误** | 错误图片标红 | 错误节点标红 | 错误参数高亮 | 红色状态 |
| **后端断连** | 禁用导入 | 禁用执行 | 禁用操作 | 红色圆点, "Disconnected" |

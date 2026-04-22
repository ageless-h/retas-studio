# RETAS STUDIO macOS 重构项目

## 项目目标

**从头重构 RETAS STUDIO**，创建原生 macOS 应用程序，实现核心动画制作功能。

## 项目状态

**当前阶段**: 核心架构完成，基础代码骨架已创建

### 已完成
- ✅ 逆向分析原始软件架构
- ✅ 提取核心类和数据结构
- ✅ 设计 macOS 原生架构
- ✅ 创建 Swift Package 项目骨架
- ✅ 实现核心数据模型
- ✅ 实现基本工具系统
- ✅ 创建 Metal 渲染管线

### 进行中
- ⏳ 完善渲染系统
- ⏳ 实现文件 I/O
- ⏳ 添加更多工具

## 目录结构

```
retas mac/
├── RetasStudio/                # Swift 项目
│   ├── Package.swift           # SPM 配置
│   └── Sources/
│       ├── App/                # SwiftUI 应用
│       │   └── RetasStudioApp.swift
│       └── Core/               # 核心模块
│           ├── Models/         # 数据模型
│           │   ├── Layer.swift
│           │   ├── Document.swift
│           │   ├── Timeline.swift
│           │   ├── Stroke.swift
│           │   └── Commands.swift
│           ├── Tools/           # 工具系统
│           │   └── Tools.swift
│           └── Rendering/       # 渲染引擎
│               ├── MetalRenderer.swift
│               └── Shaders.metal
├── docs/                       # 文档
│   ├── ARCHITECTURE.md         # 架构设计
│   ├── ANALYSIS.md             # 原始软件分析
│   ├── MIGRATION_PLAN.md       # 移植计划
│   └── HASP_SOLUTIONS.md       # 加密狗问题
└── tools/                      # 工具脚本
```

## 从原始软件提取的核心类

### 图层系统
```
CLayer (基类)
├── CRasterLayer          - 光栅图层
│   ├── CRasNormalLayer   - 普通图层
│   ├── CRasDrawLayer     - 描线图层
│   └── CRasDraftLayer    - 草稿图层
├── CVectorLayer          - 矢量图层
├── CCameraLayer          - 摄像机图层
├── CTextLayer            - 文字图层
└── CGuideLayer           - 参考线图层
```

### 工具系统
```
CPencilTool    - 铅笔
CPenTool       - 钢笔  
CBrushTool     - 笔刷
CEraserTool    - 橡皮
CSelectTool    - 选择
CHandTool      - 手型
CZoomTool      - 缩放
CBucketTool    - 填充
```

### 时间轴系统
```
CScoreDocument - 摄影表文档
CScoreWindow   - 摄影表窗口
CKeyFrame      - 关键帧
CCelInfo       - 赛璐珞信息
```

## 技术栈

| 模块 | 技术 |
|------|------|
| UI | SwiftUI + AppKit |
| 图形渲染 | Metal |
| 矢量渲染 | CoreGraphics |
| 数据存储 | SQLite + JSON |
| 图像处理 | Core Image + vImage |

## 构建和运行

```bash
cd "/Users/huzhiheng/Documents/RETAS.STUDIO.6.6.0/retas mac/RetasStudio"

# 构建
swift build

# 运行
swift run RetasStudioApp

# 或在 Xcode 中打开
open Package.swift
```

## 核心数据结构

### Document (文档)
```swift
class Document {
    var layers: [LayerID: Layer]
    var timeline: Timeline
    var width: Int = 1920
    var height: Int = 1080
}
```

### Layer (图层)
```swift
protocol Layer {
    var id: LayerID { get }
    var name: String { get set }
    var visible: Bool { get set }
    var opacity: Double { get set }
    var blendMode: BlendMode { get set }
}
```

### Timeline (时间轴)
```swift
class Timeline {
    var layers: [LayerID]
    var frameRate: Double = 24.0
    var totalFrames: Int = 144
}
```

## 下一步开发

### Phase 1: 基础功能 (当前)
1. 完善画布渲染
2. 实现基本绘图工具
3. 添加图层管理 UI

### Phase 2: 动画功能
1. 时间轴编辑
2. 关键帧系统
3. 洋葱皮效果

### Phase 3: 高级功能
1. 矢量图层
2. 特效系统
3. 视频导出

## 文档资源

- [架构设计](docs/ARCHITECTURE.md) - 详细的技术架构文档
- [原始软件分析](docs/ANALYSIS.md) - RETAS STUDIO 逆向分析
- [移植计划](docs/MIGRATION_PLAN.md) - 开发路线图

---
*项目创建: 2026-04-20*
*最后更新: 2026-04-20*

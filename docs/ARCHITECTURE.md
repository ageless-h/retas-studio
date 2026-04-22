# RETAS STUDIO 重构项目 - 架构设计

## 项目目标

从头重构 RETAS STUDIO，创建原生 macOS 应用程序，实现核心功能。

## 从逆向分析提取的核心类结构

### 基础框架类 (RC 前缀)
```
RCString, RCStringArray      - 字符串
RCFile, RCFilePath           - 文件操作
RCArchive, RCArchiveFile     - 序列化
RCRect, RCPoint, RCSize      - 几何类型
RCBitmap, RCPict             - 图像
RCGdi, RCVOffscreen          - 图形渲染
RCWindow, RCControl          - UI 控件
RCRegKey                     - 配置存储
RCThread, RCCriticalSection  - 多线程
```

### 领域模型类 (C 前缀)

#### 图层系统
```
CLayer (基类)
├── CRasterLayer          - 光栅图层
│   ├── CRasNormalLayer   - 普通图层
│   ├── CRasDrawLayer     - 描线图层
│   ├── CRasDraftLayer    - 草稿图层
│   ├── CRasSelectLayer   - 选择图层
│   └── CRasPaintMonoLayer - 单色上色
├── CVectorLayer          - 矢量图层
│   ├── CVectorLineLayer  - 矢量线
│   ├── CVectorPaintLayer - 矢量上色
│   └── CVectorSelectLayer
├── CFrameLayer           - 帧图层
├── CShapeLayer           - 形状图层
├── CTextLayer            - 文字图层
├── CCameraLayer          - 摄像机图层
├── CGuideLayer           - 参考线图层
├── CGridLayer            - 网格图层
├── CRulerLayer           - 尺图层
└── CFloatingLayer        - 浮动图层
```

#### 工具系统
```
CTool (基类)
├── CPencilTool           - 铅笔工具
├── CPenTool              - 钢笔工具
├── CBrushTool            - 笔刷工具
├── CEraserTool           - 橡皮工具
├── CSelectTool           - 选择工具
│   ├── CSelectRectTool   - 矩形选择
│   ├── CSelectLassoTool  - 套索选择
│   ├── CSelectWandTool   - 魔术棒
│   └── CSelectTraceTool  - 描边选择
├── CMoveTool             - 移动工具
├── CRotateTool           - 旋转工具
├── CZoomTool             - 缩放工具
├── CHandTool             - 手型工具
├── CBucketTool           - 填充工具
├── CSpuitTool            - 吸管工具
├── CTextTool             - 文字工具
├── CShapeTool            - 形状工具
└── CFilterTool           - 滤镜工具
```

#### 时间轴/摄影表
```
CScoreDocument           - 摄影表文档
CScoreWindow             - 摄影表窗口
CScoreView               - 摄影表视图
CScoreLayerInfo          - 图层信息
CKeyFrame                - 关键帧
CCelInfo                 - 赛璐珞信息
CCelDoItem               - 赛璐珞操作项
```

#### 文档系统
```
CDocument (基类)
├── CCelDocument         - 赛璐珞文档
├── CScoreDocument       - 摄影表文档
└── CColorChartDocument  - 色卡文档
```

## macOS 重构架构

### 技术栈选择

| 模块 | 技术 | 原因 |
|------|------|------|
| UI框架 | SwiftUI + AppKit | SwiftUI 现代化，AppKit 处理复杂窗口 |
| 图形渲染 | Metal | 高性能，原生支持 |
| 矢量渲染 | CoreGraphics + 自定义 | 兼顾性能和兼容性 |
| 数据持久化 | SQLite + JSON | 结构化存储 + 配置文件 |
| 图像处理 | Core Image + vImage | 系统优化 |
| 文件格式 | 自定义二进制 + 开放格式 | 兼容性和可扩展性 |

### 模块划分

```
RetasStudio/
├── App/
│   ├── RetasStudioApp.swift
│   ├── AppDelegate.swift
│   └── AppState.swift
├── Core/
│   ├── Models/
│   │   ├── Layer.swift
│   │   ├── Frame.swift
│   │   ├── Cel.swift
│   │   ├── Timeline.swift
│   │   ├── Keyframe.swift
│   │   └── Document.swift
│   ├── Rendering/
│   │   ├── Renderer.swift
│   │   ├── MetalRenderer.swift
│   │   └── VectorRenderer.swift
│   ├── Tools/
│   │   ├── Tool.swift
│   │   ├── PenTool.swift
│   │   ├── BrushTool.swift
│   │   └── ...
│   └── Storage/
│       ├── FileFormat.swift
│       ├── Serializer.swift
│       └── ProjectBundle.swift
├── UI/
│   ├── Views/
│   │   ├── CanvasView.swift
│   │   ├── TimelineView.swift
│   │   ├── LayerPanel.swift
│   │   ├── ToolPanel.swift
│   │   └── ColorPanel.swift
│   ├── ViewModels/
│   │   ├── CanvasViewModel.swift
│   │   ├── TimelineViewModel.swift
│   │   └── ...
│   └── Components/
│       ├── CustomControls.swift
│       └── ...
├── ImportExport/
│   ├── ImageFormats/
│   ├── VideoFormats/
│   └── LegacyFormats/
└── Resources/
    ├── Assets.xcassets
    └── ...
```

## 数据结构设计

### 图层 (Layer)
```swift
struct LayerID: Hashable, Codable { let value: UUID }
struct FrameID: Hashable, Codable { let value: UUID }

protocol Layer: Codable, Identifiable {
    var id: LayerID { get }
    var name: String { get set }
    var visible: Bool { get set }
    var locked: Bool { get set }
    var opacity: Double { get set }
    var blendMode: BlendMode { get set }
    var frames: [FrameID: Frame] { get set }
}

struct RasterLayer: Layer {
    let id: LayerID
    var name: String
    var visible: Bool = true
    var locked: Bool = false
    var opacity: Double = 1.0
    var blendMode: BlendMode = .normal
    var frames: [FrameID: Frame] = [:]
    var width: Int
    var height: Int
    var bitsPerPixel: Int = 32
}

struct VectorLayer: Layer {
    let id: LayerID
    var name: String
    var visible: Bool = true
    var locked: Bool = false
    var opacity: Double = 1.0
    var blendMode: BlendMode = .normal
    var frames: [FrameID: Frame] = [:]
    var strokes: [Stroke] = []
}
```

### 帧 (Frame)
```swift
struct Frame: Codable, Identifiable {
    let id: FrameID
    var frameNumber: Int
    var duration: Int = 1
    var keyframe: Bool = false
    var label: String?
    
    // 图像数据 (光栅)
    var imageData: Data?
    
    // 或矢量数据
    var strokes: [Stroke]?
}

struct Stroke: Codable {
    var points: [Point]
    var brushSettings: BrushSettings
    var color: Color
}

struct Point: Codable {
    var x: Double
    var y: Double
    var pressure: Double
    var tilt: Point?
    var timestamp: Double
}
```

### 时间轴 (Timeline)
```swift
struct Timeline: Codable {
    var layers: [LayerID]
    var frameRate: Double = 24.0
    var totalFrames: Int
    var currentTime: Int = 0
    var inPoint: Int = 0
    var outPoint: Int
    
    var markers: [Marker] = []
    var guides: [Guide] = []
}

struct Marker: Codable, Identifiable {
    let id: UUID
    var frame: Int
    var name: String
    var color: Color
}
```

### 文档 (Document)
```swift
struct Document: Codable, Identifiable {
    let id: UUID
    var name: String
    var version: String = "1.0"
    var createdAt: Date
    var modifiedAt: Date
    
    var width: Int
    var height: Int
    var resolution: Double = 72.0
    
    var layers: [LayerID: Layer]
    var timeline: Timeline
    var colorPalette: ColorPalette
    
    var camera: Camera
    var guides: [Guide]
}

struct Camera: Codable {
    var position: CGPoint
    var zoom: Double = 1.0
    var rotation: Double = 0.0
    var safeArea: CGRect?
}
```

## 文件格式设计

### 项目包结构 (.retasproj)
```
MyProject.retasproj/
├── project.json          # 项目元数据
├── timeline.json         # 时间轴数据
├── layers/
│   ├── layer-001.json    # 图层元数据
│   ├── layer-001/
│   │   ├── frame-001.png # 帧图像
│   │   ├── frame-002.png
│   │   └── ...
│   └── ...
├── assets/
│   ├── colors.json       # 色板
│   ├── brushes.json      # 笔刷预设
│   └── templates/        # 模板
├── preview/
│   └── thumbnail.png
└── metadata/
    └── history.json      # 编辑历史
```

## 下一步

1. 创建 Xcode 项目骨架
2. 实现核心数据模型
3. 构建基础渲染管线
4. 实现基本工具集
5. 添加文件 I/O 支持

---
*文档版本: 1.0*
*创建日期: 2026-04-20*

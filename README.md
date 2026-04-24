# RETAS Studio

跨平台2D动画制作软件，基于 Rust + Tauri + wgpu 技术栈。

## 项目目标

**从头重构 RETAS STUDIO**，创建现代化、跨平台的2D动画制作应用程序。

## 项目状态

**当前版本**: v0.2.0 (开发中)  
**最后更新**: 2026-04-24

### 已完成功能

| 模块 | 功能 | 状态 |
|------|------|------|
| 核心架构 | Rust 跨平台架构设计 | ✅ |
| 数据模型 | Document, Layer, Timeline | ✅ |
| 渲染 | wgpu GPU 渲染管线 | ✅ |
| 前端 | Tauri 2 + React + TypeScript | ✅ |
| 画布 | CanvasKit (Skia WASM) 渲染 | ✅ |
| 图层 | 创建/删除/可见性/锁定/透明度 | ✅ |
| 时间轴 | 帧编辑/播放控制 | ✅ |
| 混合模式 | Normal/Multiply/Screen 等 8 种 | ✅ |
| 撤销重做 | 完整 Undo/Redo 系统 | ✅ |
| 摄影表 | XSheet 网格视图/关键帧管理 | ✅ |
| 洋葱皮 | 前后帧显示/透明度调节 | ✅ |
| 选择工具 | 矩形/椭圆/套索/魔棒 | ✅ |
| 快捷键 | 工具切换/笔刷大小调整 | ✅ |

### 进行中

- ⏳ 压感支持 (Wacom 数位板)
- ⏳ 视频导出 (MP4/WebM)
- ⏳ 补间动画系统
- ⏳ 矢量工具完善

## 技术栈

| 模块 | 技术 |
|------|------|
| 语言 | Rust |
| UI 框架 | Tauri 2 + React + TypeScript |
| 前端组件 | BlueprintJS + Lucide React |
| 画布 | CanvasKit (Skia WASM) |
| 图形渲染 | wgpu 23 (WebGPU/Metal/Vulkan/DX12) |
| 矢量渲染 | Lyon |
| 图像处理 | image-rs |
| 序列化 | Serde + JSON |

## 目录结构

```
retas-ageless/
├── crates/
│   ├── retas-core/           # 核心数据结构
│   │   └── src/
│   │       ├── document.rs   # 文档模型
│   │       ├── layer.rs      # 图层系统
│   │       └── advanced/     # 高级功能
│   ├── retas-render/         # GPU 渲染
│   │   └── src/shaders/      # WGSL 着色器
│   ├── retas-vector/         # 矢量图形
│   └── retas-io/             # 文件格式
│       └── src/export/       # 导出模块
│
├── retas-tauri/              # Tauri 应用
│   ├── src/                  # React 前端
│   │   ├── components/       # UI 组件
│   │   │   ├── XSheetPanel.tsx
│   │   │   ├── OnionSkinPanel.tsx
│   │   │   ├── SelectionToolPanel.tsx
│   │   │   └── UnifiedCanvas.tsx
│   │   ├── hooks/            # React hooks
│   │   └── api.ts            # Tauri API
│   └── src-tauri/            # Rust 后端
│       └── src/lib.rs        # Tauri 命令
│
├── docs/                     # 文档
│   ├── ROADMAP.md            # 开发路线图
│   ├── TODO.md               # 待办事项
│   └── ARCHITECTURE.md       # 架构文档
│
└── Reference/                # 参考资料
    └── csp-guide/            # CSP 功能参考
```

## 构建和运行

### 环境要求
- Rust 1.75+
- Node.js 18+
- macOS / Windows / Linux

### 开发模式

```bash
cd retas-tauri
npm install
npm run tauri dev
```

### 生产构建

```bash
cd retas-tauri
npm run tauri build
```

### 运行测试

```bash
cargo test --workspace
cd retas-tauri && npm test
```

## 文档资源

| 文档 | 说明 |
|------|------|
| [开发路线图](docs/ROADMAP.md) | 功能规划与进度 |
| [待办事项](docs/TODO.md) | 具体任务清单 |
| [架构文档](docs/CROSS_PLATFORM_ARCHITECTURE.md) | 技术架构设计 |
| [开发指南](docs/DEVELOPMENT.md) | 开发流程与规范 |
| [文件格式](docs/FILE_FORMATS.md) | CEL/DGA/SCS 格式 |

## 核心数据结构

### Document (文档)
```rust
use retas_core::Document;

let mut doc = Document::new("My Animation", 1920.0, 1080.0);
doc.settings.frame_rate = 24.0;
doc.timeline.end_frame = 100;
```

### Layer (图层)
```rust
use retas_core::{RasterLayer, Layer};

let mut raster = RasterLayer::new("Background");
raster.base.opacity = 0.8;
let layer = Layer::Raster(raster);
```

### Selection (选区)
```rust
use retas_core::advanced::selection::{Selection, SelectionMask};

let sel = Selection::rectangular(Rect::new(0.0, 0.0, 100.0, 100.0));
```

## 贡献

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

MIT License

---
*项目创建: 2026-04-20*

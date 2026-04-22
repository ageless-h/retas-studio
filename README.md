# RETAS Studio

跨平台2D动画制作软件，基于 Rust + Tauri + wgpu 技术栈。

## 项目目标

**从头重构 RETAS STUDIO**，创建现代化、跨平台的2D动画制作应用程序。

## 项目状态

**当前阶段**: 核心架构完成，Tauri 前端可用

### 已完成
- ✅ 逆向分析原始软件架构
- ✅ 提取核心类和数据结构
- ✅ 设计 Rust 跨平台架构
- ✅ 实现核心数据模型 (retas-core)
- ✅ 实现 GPU 渲染管线 (retas-render, wgpu)
- ✅ 实现矢量图形系统 (retas-vector)
- ✅ 实现文件 I/O (CEL/DGA/SCS) (retas-io)
- ✅ Tauri 前端框架 (React + TypeScript)
- ✅ CanvasKit 画布渲染
- ✅ 图层管理系统
- ✅ 时间轴编辑
- ✅ Undo/Redo 系统
- ✅ 多种混合模式 (Normal, Multiply, Screen, Overlay, HardLight, SoftLight, Difference, Exclusion)

### 进行中
- ⏳ 完善绘图工具 (钢笔、笔刷、填充)
- ⏳ 洋葱皮效果
- ⏳ 视频导出
- ⏳ 关键帧动画系统

## 目录结构

```
retas-studio/
├── Cargo.toml                    # Rust workspace
├── crates/
│   ├── retas-core/               # 核心数据结构与类型
│   │   ├── src/
│   │   └── tests/
│   ├── retas-render/             # wgpu GPU 渲染
│   │   ├── src/
│   │   └── src/shaders/          # WGSL 着色器
│   ├── retas-vector/             # 矢量图形与路径
│   │   ├── src/
│   │   └── tests/
│   └── retas-io/                 # 文件格式支持 (CEL/DGA/SCS)
│       ├── src/
│       ├── src/export/           # 导出模块 (PNG/JPEG/GIF/SVG/SWF)
│       └── tests/
│
├── retas-tauri/                  # Tauri 应用程序
│   ├── src/                      # React + TypeScript 前端
│   │   ├── components/           # UI 组件
│   │   ├── hooks/                # React hooks
│   │   ├── utils/                # 工具函数
│   │   ├── api.ts                # Tauri command 封装
│   │   └── App.tsx               # 主应用
│   └── src-tauri/                # Rust Tauri 后端
│       ├── src/
│       └── Cargo.toml
│
└── docs/                         # 文档
    ├── ANALYSIS.md               # 原始 RETAS 逆向分析
    ├── CLASS_REFERENCE.md        # C++ 类结构参考
    ├── CROSS_PLATFORM_ARCHITECTURE.md  # 架构设计
    ├── DEVELOPMENT.md            # 开发指南
    ├── FILE_FORMATS.md           # 文件格式规范
    ├── FEATURE_COMPARISON.md     # 功能对比
    └── IMPLEMENTATION_ROADMAP.md # 实施路线图
```

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

## 构建和运行

### 环境要求
- Rust 1.75+
- Node.js 18+
- macOS / Windows / Linux

### 构建 Rust workspace

```bash
cd retas-studio
cargo build
```

### 运行测试

```bash
cargo test --workspace
```

### 构建并运行 Tauri 应用

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

## 文档资源

- [架构设计](docs/CROSS_PLATFORM_ARCHITECTURE.md) - 详细的技术架构文档
- [原始软件分析](docs/ANALYSIS.md) - RETAS STUDIO 逆向分析
- [开发指南](docs/DEVELOPMENT.md) - Rust 开发流程与代码示例
- [文件格式规范](docs/FILE_FORMATS.md) - CEL/DGA/SCS 格式详解

---
*项目创建: 2026-04-20*
*最后更新: 2026-04-22*

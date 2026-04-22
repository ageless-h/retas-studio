# RETAS Studio 开发指南

## 快速开始

### 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 构建项目

```bash
cd retas-studio
cargo build
```

### 运行测试

```bash
cargo test
```

### 运行应用

```bash
cargo run
```

---

## 项目架构

### 模块依赖关系

```
retas-ui
    ├── retas-render
    │       └── retas-core
    ├── retas-vector
    │       └── retas-core
    ├── retas-io
    │       └── retas-core
    └── retas-core
```

### 各模块职责

| 模块 | 职责 | 关键类型 |
|------|------|----------|
| `retas-core` | 核心数据结构 | Point, Rect, Color, Layer, Document |
| `retas-vector` | 矢量图形 | BezierCurve, Stroke, Path |
| `retas-render` | GPU 渲染 | RenderDevice, Renderer, Texture |
| `retas-io` | 文件格式 | CelFile, DgaFile, ScsFile |
| `retas-ui` | 用户界面 | RetasApp, Canvas, Timeline |

---

## 核心数据结构

### Point (点)

```rust
use retas_core::Point;

let p1 = Point::new(10.0, 20.0);
let p2 = Point::new(30.0, 40.0);

let distance = p1.distance_to(&p2);
let midpoint = p1.midpoint(&p2);
let interpolated = p1.lerp(&p2, 0.5);
```

### Color (颜色)

```rust
use retas_core::{Color8, Color16, ColorF};

// 8位颜色
let c = Color8::from_rgb(255, 128, 64);
let c = Color8::from_hex(0xFF8040);

// 16位颜色 (RETAS 原生精度)
let c16 = Color16::new(32768, 16384, 8192, 65535);

// 浮点颜色 (HSV 支持)
let cf = ColorF::from_hsv(120.0, 1.0, 1.0); // 绿色
let (h, s, v) = cf.to_hsv();
```

### Matrix2D (变换矩阵)

```rust
use retas_core::Matrix2D;

let translate = Matrix2D::translation(10.0, 20.0);
let scale = Matrix2D::scale(2.0, 2.0);
let rotate = Matrix2D::rotation(std::f64::consts::PI / 4.0);

let combined = translate.multiply(&scale).multiply(&rotate);
let transformed = combined.transform_point(&point);
```

### Layer (图层)

```rust
use retas_core::{RasterLayer, VectorLayer, CameraLayer, Layer};

let mut raster = RasterLayer::new("Background");
raster.base.opacity = 0.8;
raster.base.visible = true;

let layer = Layer::Raster(raster);
```

### Document (文档)

```rust
use retas_core::{Document, Project};

let mut doc = Document::new("My Animation", 1920.0, 1080.0);
doc.settings.frame_rate = 24.0;
doc.timeline.end_frame = 100;

let layer_id = doc.add_layer(layer);
```

---

## 矢量图形

### BezierCurve (贝塞尔曲线)

```rust
use retas_vector::{BezierCurve, BezierControlPoint};

let mut curve = BezierCurve::new();
curve.add_point(BezierControlPoint::corner(Point::new(0.0, 0.0)));
curve.add_point(BezierControlPoint::smooth(
    Point::new(50.0, 50.0),
    Point::new(25.0, 25.0),
    Point::new(75.0, 75.0),
));
curve.close();

let flattened = curve.flatten(0.5); // 容差 0.5 像素
```

### Stroke (笔画)

```rust
use retas_vector::{Stroke, StrokeStyle, PressurePoint};

let style = StrokeStyle::new(Color8::BLACK, 5.0);
let mut stroke = Stroke::new(style);

stroke.add_point(PressurePoint::new(Point::new(0.0, 0.0), 1.0));
stroke.add_point(PressurePoint::new(Point::new(10.0, 10.0), 0.8));

stroke.simplify(1.0); // 简化曲线
```

### Path (路径)

```rust
use retas_vector::Path;

let rect = Path::rect(0.0, 0.0, 100.0, 100.0);
let circle = Path::circle(50.0, 50.0, 25.0);

let custom = Path::new()
    .move_to(Point::new(0.0, 0.0))
    .line_to(Point::new(100.0, 0.0))
    .curve_to(
        Point::new(150.0, 50.0),
        Point::new(100.0, 100.0),
        Point::new(50.0, 100.0),
    )
    .close();
```

---

## 文件格式

### 读取 CEL 文件

```rust
use retas_io::CelFile;

let cel = CelFile::open("animation.cel")?;

println!("Document: {}", cel.header.name);
println!("Size: {}x{}", cel.header.width, cel.header.height);
println!("Frames: {}", cel.header.frame_count);

for frame in &cel.frames {
    let data = cel.get_frame_data(frame)?;
    // data 是解压后的像素数据
}

let document = cel.to_document();
```

### 读取 DGA 文件

```rust
use retas_io::DgaFile;

let dga = DgaFile::open("drawing.dga")?;

for stroke in &dga.strokes {
    println!("Stroke {} ({} points)", stroke.id, stroke.point_count);
}
```

### 读取 SCS 文件

```rust
use retas_io::ScsFile;

let scs = ScsFile::open("scene.scs")?;

for layer in &scs.layers {
    println!("Layer: {} (type {})", layer.name, layer.layer_type);
}
```

---

## GPU 渲染

### 初始化渲染器

```rust
use retas_render::{RenderDevice, Renderer};

let device = RenderDevice::new().await?;
let renderer = Renderer::new().await?;
```

### 创建纹理

```rust
use retas_render::RenderTexture;

let texture = RenderTexture::new(
    &device.device,
    1920,
    1080,
    wgpu::TextureFormat::Rgba8UnormSrgb,
    Some("canvas"),
);

let solid = RenderTexture::solid_color(
    &device.device,
    &device.queue,
    1920,
    1080,
    Color8::WHITE,
    Some("white"),
);
```

### 渲染文档

```rust
renderer.render_document(&document, &texture);
```

---

## 调试

### 启用日志

```bash
RUST_LOG=debug cargo run
RUST_LOG=retas_render=trace cargo run
```

### 性能分析

```bash
cargo run --release
```

### 检查编译问题

```bash
cargo check
cargo clippy
```

---

## 测试

### 运行所有测试

```bash
cargo test
```

### 运行特定测试

```bash
cargo test test_point_distance
cargo test --package retas-core
```

### 测试覆盖率

```bash
cargo tarpaulin
```

---

## 代码风格

### 格式化

```bash
cargo fmt
```

### Lint

```bash
cargo clippy -- -D warnings
```

---

## 发布构建

```bash
cargo build --release
```

优化后的二进制文件位于：
```
target/release/retas
```

---

## 常见问题

### Q: 编译失败 "linker `cc` not found"

安装 Xcode Command Line Tools:
```bash
xcode-select --install
```

### Q: WGPU 初始化失败

确保系统有支持的 GPU 驱动：
- macOS: Metal (所有 Mac 都支持)
- Windows: DirectX 12 或 Vulkan
- Linux: Vulkan

### Q: 如何添加新的文件格式支持？

1. 在 `retas-io/src/` 创建新模块
2. 实现 `FileReader` trait
3. 在 `lib.rs` 中导出

---

## 贡献指南

1. Fork 仓库
2. 创建功能分支
3. 编写测试
4. 提交 Pull Request

### 提交信息格式

```
type(scope): description

[optional body]

[optional footer]
```

类型:
- `feat`: 新功能
- `fix`: 修复
- `docs`: 文档
- `refactor`: 重构
- `test`: 测试
- `chore`: 杂项

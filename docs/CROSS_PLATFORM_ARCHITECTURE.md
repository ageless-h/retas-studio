# RETAS STUDIO 跨平台克隆 - 架构设计

## 一、设计目标

创建一个**跨平台**的2D动画制作软件，支持：
- macOS (Apple Silicon + Intel)
- Windows
- Linux
- Web (WASM)

## 二、平台抽象层设计

### 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                         │
│  (平台无关的业务逻辑 - 纯 Rust/Kotlin Multiplatform)          │
├─────────────────────────────────────────────────────────────┤
│                    ABSTRACTION LAYER                         │
│  ┌──────────────┬──────────────┬───────────────────────────┐│
│  │ Graphics API │  File I/O    │  Platform Services        ││
│  │ (统一接口)    │  (统一接口)   │  (统一接口)                ││
│  └──────────────┴──────────────┴───────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│                    PLATFORM IMPLEMENTATION                   │
│  ┌────────────┬────────────┬────────────┬──────────────────┐│
│  │   macOS    │  Windows   │   Linux    │   Web/WASM       ││
│  │  Metal/CG  │  D2D/D3D   │  Vulkan    │   WebGL/WebGPU   ││
│  │  CoreText  │  DirectW   │  FreeType  │   Canvas API     ││
│  │  FS Events │  ReadDirCh │  inotify   │   File API       ││
│  └────────────┴────────────┴────────────┴──────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## 三、技术栈选择

### 方案 A: Rust + 通用 UI (推荐)

| 模块 | 技术选择 | 原因 |
|------|----------|------|
| 语言 | **Rust** | 跨平台、高性能、内存安全 |
| GUI | **Tauri 2** | Web 技术栈，跨平台，成熟生态 |
| 图形 | **wgpu 23** | 统一 GPU API (Metal/Vulkan/DX12/WebGPU) |
| 矢量 | **Lyon** | Rust 原生矢量渲染 |
| 图像 | **Image-rs** | 跨平台图像处理 |
| 音频 | **Rodio** 或 **CPAL** | 跨平台音频 |
| 序列化 | **Serde** | Rust 标准序列化 |

### 方案 B: Kotlin Multiplatform

| 模块 | 技术选择 |
|------|----------|
| 语言 | Kotlin Multiplatform |
| UI | Compose Multiplatform |
| 图形 | Skia (via Compose) |
| 桌面 | JVM Native |
| 移动端 | Kotlin Native |

### 方案 C: C++ + Qt (传统方案)

| 模块 | 技术选择 |
|------|----------|
| 语言 | C++17/20 |
| GUI | Qt 6 |
| 图形 | Qt Quick Scene Graph / RHI |
| 跨平台 | Qt 原生支持 |

## 四、核心模块设计 (基于逆向分析)

### 4.1 基础数据类型 (对应 RC* 类)

```rust
// src/core/types.rs

/// 几何类型
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct HsvColor {
    pub h: f64,
    pub s: f64,
    pub v: f64,
    pub a: f64,
}

/// 2D 变换矩阵 (对应 RCMatrix2D)
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Matrix2D {
    pub a: f64, pub b: f64,
    pub c: f64, pub d: f64,
    pub tx: f64, pub ty: f64,
}

impl Matrix2D {
    pub fn identity() -> Self {
        Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx: 0.0, ty: 0.0 }
    }
    
    pub fn translate(x: f64, y: f64) -> Self {
        Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx: x, ty: y }
    }
    
    pub fn rotate(angle: f64) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self { a: cos, b: sin, c: -sin, d: cos, tx: 0.0, ty: 0.0 }
    }
    
    pub fn scale(sx: f64, sy: f64) -> Self {
        Self { a: sx, b: 0.0, c: 0.0, d: sy, tx: 0.0, ty: 0.0 }
    }
}
```

### 4.2 图层系统 (对应 CCLayer* 类)

```rust
// src/core/layer.rs

use std::collections::HashMap;

/// 图层类型枚举
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum LayerType {
    Raster,
    Vector,
    Camera,
    Text,
    Shape,
    Guide,
    Sound,
}

/// 混合模式 (对应 RCPixelMixer*)
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

/// 图层 ID
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LayerId(pub uuid::Uuid);

/// 图层基类 trait
pub trait Layer: std::any::Any {
    fn id(&self) -> LayerId;
    fn name(&self) -> &str;
    fn set_name(&mut self, name: String);
    fn visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn locked(&self) -> bool;
    fn set_locked(&mut self, locked: bool);
    fn opacity(&self) -> f64;
    fn set_opacity(&mut self, opacity: f64);
    fn blend_mode(&self) -> BlendMode;
    fn set_blend_mode(&mut self, mode: BlendMode);
    fn layer_type(&self) -> LayerType;
}

/// 光栅图层 (对应 CCRasterLayer, CCRasNormalLayer 等)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RasterLayer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub width: u32,
    pub height: u32,
    pub frames: HashMap<u32, RasterFrame>,
    pub layer_subtype: RasterLayerSubtype,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum RasterLayerSubtype {
    Normal,
    Draw,
    Draft,
    PaintMono,
    PaintGrad,
    Select,
    TempLine,
}

/// 光栅帧
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RasterFrame {
    pub id: FrameId,
    pub frame_number: u32,
    pub data: Vec<u8>,
    pub format: PixelFormat,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum PixelFormat {
    Rgba8,
    Bgra8,
    Gray8,
    Indexed8,
}

/// 矢量图层 (对应 CCVectorLayer, CCVectorLineLayer 等)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VectorLayer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub strokes: Vec<Stroke>,
    pub keyframes: HashMap<u32, VectorKeyframe>,
    pub layer_subtype: VectorLayerSubtype,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum VectorLayerSubtype {
    Line,
    Paint,
    Select,
    Face,
    TempLine,
}

/// 矢量笔画 (对应 CCLayerDoItemVecDraw 相关)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Stroke {
    pub id: StrokeId,
    pub points: Vec<StrokePoint>,
    pub brush_settings: BrushSettings,
    pub color: Color,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StrokePoint {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
    pub tilt_x: f64,
    pub tilt_y: f64,
    pub timestamp: f64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BrushSettings {
    pub size: f64,
    pub opacity: f64,
    pub hardness: f64,
    pub spacing: f64,
    pub flow: f64,
    pub smoothing: f64,
    pub pressure_size: bool,
    pub pressure_opacity: bool,
}

/// 摄像机图层 (对应 CCompositeCameraLayer, CCameraFramePlane)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CameraLayer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub position: Animatable<Point>,
    pub zoom: Animatable<f64>,
    pub rotation: Animatable<f64>,
}
```

### 4.3 时间轴系统 (对应 CCScore* 类)

```rust
// src/core/timeline.rs

use std::collections::HashMap;

/// 时间轴 (对应 CCScoreDocument, CCScoreView)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Timeline {
    pub layer_order: Vec<LayerId>,
    pub frame_rate: f64,
    pub total_frames: u32,
    pub current_frame: u32,
    pub in_point: u32,
    pub out_point: u32,
    pub markers: Vec<Marker>,
}

/// 标记 (对应摄影表标记)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Marker {
    pub id: uuid::Uuid,
    pub frame: u32,
    pub name: String,
    pub color: Color,
}

/// 关键帧 (对应 CKeyFrame, CCScoreDoItemKeyFrame)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Keyframe<T> {
    pub frame: f64,
    pub value: T,
    pub interpolation: Interpolation,
    pub ease_in: Option<Point>,
    pub ease_out: Option<Point>,
}

/// 插值类型 (对应 CTween* 类)
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Interpolation {
    Linear,
    Step,
    Bezier,
    Hermite,
    Accelerate,
    Decelerate,
    Uniform,
    BSpline,
    FollowPath,
}

/// 可动画属性
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Animatable<T> {
    pub keyframes: Vec<Keyframe<T>>,
}

impl<T: Clone + std::fmt::Debug> Animatable<T> {
    pub fn value_at(&self, frame: f64) -> Option<&T> {
        // 二分查找关键帧
        let idx = self.keyframes.binary_search_by(|k| {
            k.frame.partial_cmp(&frame).unwrap_or(std::cmp::Ordering::Equal)
        }).ok()?;
        Some(&self.keyframes[idx].value)
    }
}
```

### 4.4 文档系统 (对应 CCDocument, CCCelDocument)

```rust
// src/core/document.rs

use std::collections::HashMap;

/// 项目文档
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub id: uuid::Uuid,
    pub name: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    
    pub width: u32,
    pub height: u32,
    pub resolution: f64,
    
    pub layers: HashMap<LayerId, LayerContainer>,
    pub timeline: Timeline,
    pub color_palette: ColorPalette,
    pub camera: CameraLayer,
}

/// 图层容器 (用于序列化多态)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum LayerContainer {
    Raster(RasterLayer),
    Vector(VectorLayer),
    Camera(CameraLayer),
    Text(TextLayer),
    Shape(ShapeLayer),
    Guide(GuideLayer),
    Sound(SoundLayer),
}

/// 色板 (对应 CCColorPalette, CCColorChartPalette)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ColorPalette {
    pub colors: Vec<Color>,
    pub name: String,
}

/// 文字图层 (对应 CCTextLayer, CCTextPlane)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TextLayer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub text_content: String,
    pub font_family: String,
    pub font_size: f64,
    pub color: Color,
}

/// 形状图层 (对应 CCShapeLayer, CCShapePlane)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ShapeLayer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub shapes: Vec<Shape>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Shape {
    pub id: uuid::Uuid,
    pub shape_type: ShapeType,
    pub fill_color: Option<Color>,
    pub stroke_color: Option<Color>,
    pub stroke_width: f64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ShapeType {
    Rectangle { rect: Rect },
    Ellipse { rect: Rect },
    Polygon { points: Vec<Point> },
    Path { commands: Vec<PathCommand> },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    CurveTo { control1: Point, control2: Point, end: Point },
    Close,
}
```

### 4.5 工具系统 (对应 CCTool* 类)

```rust
// src/core/tools.rs

/// 工具 trait (对应 CCTool)
pub trait Tool: std::any::Any {
    fn name(&self) -> &str;
    fn icon(&self) -> &str;
    fn shortcut(&self) -> Option<&str>;
    
    fn on_mouse_down(&mut self, event: ToolEvent, context: &mut ToolContext);
    fn on_mouse_move(&mut self, event: ToolEvent, context: &mut ToolContext);
    fn on_mouse_up(&mut self, event: ToolEvent, context: &mut ToolContext);
    fn on_cancel(&mut self, context: &mut ToolContext);
}

/// 工具事件
#[derive(Clone, Debug)]
pub struct ToolEvent {
    pub point: Point,
    pub pressure: f64,
    pub tilt: Option<(f64, f64)>,
    pub modifiers: ToolModifiers,
    pub timestamp: f64,
}

/// 修饰键 (对应快捷键系统)
#[derive(Clone, Copy, Debug, Default)]
pub struct ToolModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// 工具上下文
pub struct ToolContext<'a> {
    pub document: &'a mut Document,
    pub current_layer_id: Option<LayerId>,
    pub current_frame: u32,
    pub current_color: Color,
    pub brush_settings: BrushSettings,
}

/// 笔工具 (对应 CCPenTool, CCPencilTool)
pub struct PenTool {
    current_stroke: Option<Stroke>,
}

impl Tool for PenTool {
    fn name(&self) -> &str { "Pen" }
    fn icon(&self) -> &str { "pencil" }
    fn shortcut(&self) -> Option<&str> { Some("P") }
    
    fn on_mouse_down(&mut self, event: ToolEvent, context: &mut ToolContext) {
        let point = StrokePoint {
            x: event.point.x,
            y: event.point.y,
            pressure: event.pressure,
            tilt_x: event.tilt.unwrap_or((0.0, 0.0)).0,
            tilt_y: event.tilt.unwrap_or((0.0, 0.0)).1,
            timestamp: event.timestamp,
        };
        
        self.current_stroke = Some(Stroke {
            id: StrokeId(uuid::Uuid::new_v4()),
            points: vec![point],
            brush_settings: context.brush_settings.clone(),
            color: context.current_color,
        });
    }
    
    fn on_mouse_move(&mut self, event: ToolEvent, context: &mut ToolContext) {
        if let Some(ref mut stroke) = self.current_stroke {
            stroke.points.push(StrokePoint {
                x: event.point.x,
                y: event.point.y,
                pressure: event.pressure,
                tilt_x: event.tilt.unwrap_or((0.0, 0.0)).0,
                tilt_y: event.tilt.unwrap_or((0.0, 0.0)).1,
                timestamp: event.timestamp,
            });
        }
    }
    
    fn on_mouse_up(&mut self, _event: ToolEvent, context: &mut ToolContext) {
        if let Some(stroke) = self.current_stroke.take() {
            if let Some(layer_id) = context.current_layer_id {
                if let Some(LayerContainer::Vector(layer)) = context.document.layers.get_mut(&layer_id) {
                    layer.strokes.push(stroke);
                }
            }
        }
    }
    
    fn on_cancel(&mut self, _context: &mut ToolContext) {
        self.current_stroke = None;
    }
}
```

### 4.6 渲染系统 (对应 CComposite*, RCVOffscreen*)

```rust
// src/render/renderer.rs

use wgpu::*;

/// 渲染器 (对应 CCompositeRender)
pub struct Renderer {
    device: Device,
    queue: Queue,
    surface: Option<Surface<'static>>,
    config: Option<SurfaceConfiguration>,
    pipeline: RenderPipeline,
    blend_pipelines: HashMap<BlendMode, RenderPipeline>,
}

impl Renderer {
    pub async fn new() -> Result<Self, RendererError> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .ok_or(RendererError::NoAdapter)?;
        
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await?;
        
        let shader = device.create_shader_module(include_wgsl!("shaders.wgsl"));
        
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main Pipeline"),
            // ... 配置
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        
        Ok(Self {
            device,
            queue,
            surface: None,
            config: None,
            pipeline,
            blend_pipelines: HashMap::new(),
        })
    }
    
    /// 渲染图层 (对应 CComposite::render)
    pub fn render(&mut self, layers: &[LayerContainer], frame: u32) -> Result<Texture, RendererError> {
        let texture_desc = TextureDescriptor {
            label: Some("Render Target"),
            size: Extent3d { width: 1920, height: 1080, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        
        let texture = self.device.create_texture(&texture_desc);
        let view = texture.create_view(&TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::WHITE),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            render_pass.set_pipeline(&self.pipeline);
            
            // 从后向前渲染图层
            for layer in layers.iter().rev() {
                if !layer.visible() {
                    continue;
                }
                self.render_layer(layer, frame, &mut render_pass);
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        
        Ok(texture)
    }
    
    fn render_layer<'a>(&self, layer: &LayerContainer, frame: u32, pass: &mut RenderPass<'a>) {
        match layer {
            LayerContainer::Raster(l) => self.render_raster_layer(l, frame, pass),
            LayerContainer::Vector(l) => self.render_vector_layer(l, pass),
            _ => {}
        }
    }
    
    fn render_raster_layer<'a>(&self, layer: &RasterLayer, frame: u32, pass: &mut RenderPass<'a>) {
        if let Some(raster_frame) = layer.frames.get(&frame) {
            // 渲染光栅帧
        }
    }
    
    fn render_vector_layer<'a>(&self, layer: &VectorLayer, pass: &mut RenderPass<'a>) {
        for stroke in &layer.strokes {
            self.render_stroke(stroke, pass);
        }
    }
    
    fn render_stroke<'a>(&self, stroke: &Stroke, pass: &mut RenderPass<'a>) {
        // 将笔画转换为三角形并渲染
    }
}
```

## 五、文件格式设计

### 项目文件结构 (.retas)

```
project.retas/
├── manifest.json           # 项目元数据
├── document.json           # 文档结构
├── timeline.json           # 时间轴数据
├── layers/
│   ├── layer_001.json      # 图层元数据
│   ├── layer_001/
│   │   ├── frame_0001.rgba # 帧数据 (光栅)
│   │   ├── frame_0002.rgba
│   │   └── strokes.json    # 矢量笔画数据
│   └── ...
├── assets/
│   ├── brushes/            # 笔刷预设
│   ├── palettes/           # 色板
│   └── templates/          # 模板
├── audio/
│   └── track_001.wav       # 音频文件
├── cache/
│   └── thumbnails/         # 缩略图缓存
└── meta/
    └── history.json        # 编辑历史
```

### 二进制帧格式

```rust
// 帧文件头
#[repr(C)]
struct FrameHeader {
    magic: [u8; 4],        // "RFRM"
    version: u16,
    width: u32,
    height: u32,
    format: PixelFormatRaw,
    compressed: u8,
    reserved: [u8; 13],
}

#[repr(C)]
enum PixelFormatRaw {
    Rgba8 = 0,
    Bgra8 = 1,
    Gray8 = 2,
    Indexed8 = 3,
}
```

## 六、项目结构

```
retas-studio/
├── Cargo.toml                    # Rust 工作区
├── Cargo.lock
├── README.md
├── LICENSE
│
├── crates/
│   ├── retas-core/               # 核心库 (平台无关)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs          # 基础类型
│   │       ├── layer.rs          # 图层系统
│   │       ├── timeline.rs       # 时间轴
│   │       ├── document.rs       # 文档
│   │       ├── tools/            # 工具系统
│   │       │   ├── mod.rs
│   │       │   ├── pen.rs
│   │       │   ├── brush.rs
│   │       │   └── ...
│   │       ├── filters/          # 滤镜
│   │       │   ├── mod.rs
│   │       │   ├── blur.rs
│   │       │   ├── color.rs
│   │       │   └── ...
│   │       └── io/               # 文件 I/O
│   │           ├── mod.rs
│   │           ├── project.rs
│   │           └── import.rs
│   │
│   ├── retas-render/             # 渲染库
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs       # WGPU 渲染器
│   │       ├── shaders/          # 着色器
│   │       └── vector.rs         # 矢量渲染
│   │
│   ├── retas-audio/              # 音频库
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── playback.rs
│   │
│   └── retas-app/                # 应用程序
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── app.rs            # Iced 应用
│           ├── views/
│           │   ├── canvas.rs
│           │   ├── timeline.rs
│           │   ├── layers.rs
│           │   └── tools.rs
│           └── platform/         # 平台特定代码
│               ├── mod.rs
│               ├── macos.rs
│               ├── windows.rs
│               └── linux.rs
│
├── docs/                         # 文档
│   ├── architecture.md
│   ├── class_reference.md        # 类参考 (从 RETAS 提取)
│   └── file_format.md
│
├── assets/                       # 资源文件
│   ├── icons/
│   └── templates/
│
└── tests/                        # 测试
    └── integration/
```

## 七、依赖关系

```toml
# Cargo.toml (工作区)

[workspace]
members = [
    "crates/retas-core",
    "crates/retas-render",
    "crates/retas-audio",
    "crates/retas-app",
]

[workspace.dependencies]
# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 图形
wgpu = "0.19"
lyon = "1.0"

# UI
iced = { version = "0.12", features = ["wgpu"] }

# 图像
image = "0.24"

# 音频
rodio = "0.17"

# 工具
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# 并行
rayon = "1.8"

# 错误处理
thiserror = "1.0"
anyhow = "1.0"
```

## 八、开发路线图

### Phase 1: 核心基础 (1-2 月)
- [ ] 实现核心数据类型
- [ ] 实现图层系统
- [ ] 实现时间轴
- [ ] 实现文档序列化

### Phase 2: 渲染引擎 (2-3 月)
- [ ] WGPU 渲染器基础
- [ ] 光栅图层渲染
- [ ] 矢量笔画渲染
- [ ] 混合模式实现

### Phase 3: 工具系统 (2-3 月)
- [ ] 笔/铅笔工具
- [ ] 笔刷工具
- [ ] 填充工具
- [ ] 选择工具
- [ ] 变换工具

### Phase 4: UI 框架 (2-3 月)
- [ ] 主窗口布局
- [ ] 画布视图
- [ ] 时间轴 UI
- [ ] 图层面板
- [ ] 工具面板

### Phase 5: 高级功能 (3-4 月)
- [ ] 动画系统
- [ ] 滤镜效果
- [ ] 音频支持
- [ ] 视频导出

### Phase 6: 完善 (2-3 月)
- [ ] 性能优化
- [ ] 文件格式兼容
- [ ] 多语言支持
- [ ] 文档完善

---

*文档版本: 1.0*
*创建日期: 2026-04-20*

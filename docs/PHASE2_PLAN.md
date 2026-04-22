# Phase 2: 专业功能详细计划

## 概述

Phase 2 将软件从"可用"提升到"专业"级别，实现完整的绘制工作流。

## 模块 1: 压感支持系统 (Week 4-5)

### 1.1 数位板集成

#### 技术调研
```rust
// 需要调研的库
- [ ] winit 的 DeviceEvent 支持
- [ ] 或: wacom-rs (如果可用)
- [ ] 或: rusb + HID 协议
```

#### 实现方案
```rust
pub struct StylusInput {
    pub position: (f32, f32),
    pub pressure: f32,        // 0.0 - 1.0
    pub tilt: (f32, f32),     // 倾斜角度
    pub rotation: f32,        // 笔旋转 (支持的话)
    pub is_eraser: bool,      // 是否使用橡皮端
}

pub trait StylusHandler {
    fn on_stylus_down(&mut self, input: StylusInput);
    fn on_stylus_move(&mut self, input: StylusInput);
    fn on_stylus_up(&mut self, input: StylusInput);
}
```

#### 压感曲线
```rust
pub enum PressureCurve {
    Linear,           // 线性
    Soft,             // 软曲线 (小压力变化小)
    Hard,             // 硬曲线 (小压力变化大)
    Custom(Vec<(f32, f32)>), // 自定义曲线点
}

impl PressureCurve {
    pub fn apply(&self, pressure: f32) -> f32 {
        match self {
            PressureCurve::Linear => pressure,
            PressureCurve::Soft => pressure.powf(2.0),
            PressureCurve::Hard => pressure.sqrt(),
            PressureCurve::Custom(points) => { /* 插值计算 */ }
        }
    }
}
```

#### 集成点
- [ ] Canvas Program 监听 stylus 事件
- [ ] BrushTool 使用压感调整大小/不透明度
- [ ] 设置面板添加压感配置

### 1.2 动态笔刷系统

#### 笔刷参数
```rust
pub struct DynamicBrush {
    // 大小压感
    pub size_pressure: bool,
    pub size_min: f32,
    pub size_max: f32,
    pub size_curve: PressureCurve,
    
    // 不透明度压感
    pub opacity_pressure: bool,
    pub opacity_min: f32,
    pub opacity_max: f32,
    
    // 纹理
    pub texture: Option<TextureHandle>,
    pub texture_scale: f32,
    
    // 间距
    pub spacing: f32,  // 笔刷点间隔
}
```

#### 渲染逻辑
```rust
fn render_stroke_with_pressure(&self, points: &[StylusPoint]) {
    for window in points.windows(2) {
        let p1 = &window[0];
        let p2 = &window[1];
        
        // 计算压感参数
        let pressure = self.curve.apply(p1.pressure);
        let size = self.size_min + (self.size_max - self.size_min) * pressure;
        let opacity = self.opacity_min + (self.opacity_max - self.opacity_min) * pressure;
        
        // 绘制带压感的线段
        self.draw_pressure_line(p1.position, p2.position, size, opacity);
    }
}
```

### 1.3 UI 组件

#### 笔刷设置面板
- [ ] 大小压感开关
- [ ] 最小/最大大小滑块
- [ ] 压感曲线编辑器
- [ ] 不透明度压感开关
- [ ] 预览窗口 (显示测试笔触)

#### 笔刷预设管理器
```rust
pub struct BrushPreset {
    pub name: String,
    pub icon: ImageHandle,
    pub settings: DynamicBrush,
}

pub struct BrushPresetManager {
    pub presets: Vec<BrushPreset>,
    pub current: usize,
    pub categories: HashMap<String, Vec<usize>>,
}
```

**交付标准**:
- ✅ Wacom 数位板正常工作
- ✅ 压感控制笔刷大小
- ✅ 压感控制不透明度
- ✅ 可配置的压感曲线

---

## 模块 2: 矢量图层系统 (Week 6-7)

### 2.1 矢量数据结构

#### VectorLayer 结构
```rust
pub struct VectorLayer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    
    // 矢量数据
    pub paths: Vec<VectorPath>,
    pub groups: Vec<PathGroup>,
}

pub struct VectorPath {
    pub segments: Vec<PathSegment>,
    pub closed: bool,
    pub fill_color: Option<Color>,
    pub stroke: Option<StrokeStyle>,
}

pub enum PathSegment {
    Line { end: Point },
    Quadratic { control: Point, end: Point },
    Cubic { control1: Point, control2: Point, end: Point },
}
```

### 2.2 贝塞尔曲线编辑

#### 钢笔工具增强
```rust
pub struct PenToolState {
    pub mode: PenMode,
    pub current_path: Option<VectorPath>,
    pub hovered_point: Option<PointId>,
    pub selected_points: Vec<PointId>,
}

pub enum PenMode {
    Create,       // 创建新路径
    Edit,         // 编辑现有点
    AddPoint,     // 添加点
    DeletePoint,  // 删除点
    ConvertPoint, // 转换点类型
}
```

#### 控制点操作
- [ ] 移动控制点 (改变曲线形状)
- [ ] 调整控制柄 (保持平滑)
- [ ] 断开控制柄 (创建尖角)
- [ ] 对称控制柄 (平滑曲线)

### 2.3 矢量渲染

#### 渲染策略
```rust
impl VectorLayer {
    pub fn render(&self, renderer: &mut Renderer) -> canvas::Geometry {
        let mut frame = canvas::Frame::new(renderer, self.bounds.size());
        
        for path in &self.paths {
            let canvas_path = self.to_canvas_path(path);
            
            // 填充
            if let Some(fill) = &path.fill_color {
                frame.fill(&canvas_path, *fill);
            }
            
            // 描边
            if let Some(stroke) = &path.stroke {
                frame.stroke(&canvas_path, stroke.to_canvas_stroke());
            }
        }
        
        frame.into_geometry()
    }
}
```

#### 优化策略
- [ ] 路径缓存 (只在修改时重新生成)
- [ ] LOD 系统 (远距离简化曲线)
- [ ] GPU 加速 (如果可能)

### 2.4 UI 设计

#### 路径面板
- [ ] 路径列表
- [ ] 路径可见性
- [ ] 路径锁定
- [ ] 路径操作 (合并/拆分/删除)

#### 钢笔工具选项
- [ ] 模式选择按钮
- [ ] 对齐网格开关
- [ ] 智能参考线开关

**交付标准**:
- ✅ 创建和编辑贝塞尔曲线
- ✅ 闭合路径填充
- ✅ 路径描边样式
- ✅ 与光栅图层混合

---

## 模块 3: 上色系统 (Week 8-9)

### 3.1 闭合区域检测

#### 扫描线算法
```rust
pub fn find_closed_regions(
    canvas: &PixelCanvas,
    threshold: u8
) -> Vec<ClosedRegion> {
    let mut regions = Vec::new();
    let mut visited = BitSet::new(canvas.width * canvas.height);
    
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            if !visited.get(x, y) {
                if let Some(region) = flood_fill_detect(
                    canvas, x, y, threshold, &mut visited
                ) {
                    regions.push(region);
                }
            }
        }
    }
    
    regions
}
```

#### 区域数据结构
```rust
pub struct ClosedRegion {
    pub bounds: Rect,
    pub pixels: Vec<(u32, u32)>,
    pub boundary: Vec<Point>,
    pub edge_color: Color,
}
```

### 3.2 智能填充工具

#### PaintMan 风格填充
```rust
pub struct SmartFillTool {
    pub tolerance: u8,        // 颜色容差
    pub gap_closing: u8,      // 间隙闭合大小
    pub anti_aliasing: bool,  // 抗锯齿
    pub fill_mode: FillMode,
}

pub enum FillMode {
    Flat,          // 纯色填充
    Gradient,      // 渐变填充
    Texture,       // 纹理填充
}
```

#### 填充逻辑
```rust
impl SmartFillTool {
    pub fn fill_at(&self, canvas: &mut Canvas, x: u32, y: u32, color: Color) {
        // 1. 找到闭合区域
        let region = self.find_region(canvas, x, y);
        
        // 2. 间隙闭合 (如果启用)
        let closed_region = if self.gap_closing > 0 {
            self.close_gaps(region, self.gap_closing)
        } else {
            region
        };
        
        // 3. 填充区域
        match self.fill_mode {
            FillMode::Flat => self.fill_flat(canvas, &closed_region, color),
            FillMode::Gradient => self.fill_gradient(canvas, &closed_region, color),
            FillMode::Texture => self.fill_texture(canvas, &closed_region),
        }
    }
}
```

### 3.3 颜色管理

#### 调色板系统
```rust
pub struct ColorPalette {
    pub name: String,
    pub colors: Vec<PaletteColor>,
    pub categories: Vec<String>,
}

pub struct PaletteColor {
    pub color: Color,
    pub name: Option<String>,
    pub category: String,
    pub shortcut: Option<char>,
}
```

#### 颜色面板
- [ ] 调色板列表
- [ ] 颜色拾取
- [ ] 颜色历史
- [ ] 最近使用颜色

### 3.4 UI 设计

#### 填充工具选项
- [ ] 容差滑块 (0-255)
- [ ] 间隙闭合滑块
- [ ] 填充模式选择
- [ ] "参考图层" 选择器

#### 上色工作流
```
1. 线稿图层 (透明锁定)
2. 新建上色图层
3. 使用填充工具点击区域
4. 选择颜色填充
5. 重复直到完成
```

**交付标准**:
- ✅ 自动检测闭合区域
- ✅ PaintMan 风格填充
- ✅ 间隙闭合功能
- ✅ 颜色调色板

---

## 模块 4: 动画播放系统 (Week 8-9)

### 4.1 播放引擎

#### 播放控制器
```rust
pub struct PlaybackController {
    pub state: PlaybackState,
    pub fps: f32,
    pub frame_range: (u32, u32),
    pub current_frame: u32,
    pub loop_mode: LoopMode,
}

pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub enum LoopMode {
    Once,
    Loop,
    PingPong,  // 来回播放
}
```

#### 帧缓冲
```rust
pub struct FrameBuffer {
    pub cache: LruCache<u32, ImageHandle>,
    pub max_size: usize,
    pub preloaded_range: (u32, u32),
}

impl FrameBuffer {
    pub fn preload_range(&mut self, start: u32, end: u32) {
        for frame in start..=end {
            if !self.cache.contains(&frame) {
                let image = self.render_frame(frame);
                self.cache.put(frame, image);
            }
        }
    }
}
```

### 4.2 时间轴增强

#### 关键帧标记
- [ ] 关键帧图标
- [ ] 过渡帧指示
- [ ] 空白帧标记

#### 播放控制
- [ ] 播放按钮 (空格键)
- [ ] 帧前进/后退 (左右箭头)
- [ ] 跳转到帧 (Alt+点击)
- [ ] 播放范围设置

### 4.3 性能优化

#### 渲染优化
- [ ] 预渲染帧缓存
- [ ] 只渲染变化区域
- [ ] 降分辨率预览模式

#### 内存管理
- [ ] LRU 缓存策略
- [ ] 后台线程预加载
- [ ] 内存使用限制

**交付标准**:
- ✅ 24fps 流畅播放
- [ ] 循环播放
- [ ] 播放范围设置
- [ ] 关键帧导航

---

## 技术依赖

### 外部库
```toml
[dependencies]
# 压感支持 (调研中)
# wacom = "0.1"  # 或类似

# 几何算法
lyon = "1.0"  # 已在用

# 缓存
lru = "0.12"

# 并发
rayon = "1.8"
```

### 系统要求
- 压感测试需要 Wacom 数位板
- 性能测试需要大画布 (4K)

## 风险评估

| 模块 | 风险 | 缓解 |
|------|------|------|
| 压感 | macOS 驱动支持 | 提前测试原型 |
| 矢量 | 复杂度高 | 分阶段实现 |
| 上色 | 算法复杂 | 参考开源实现 |
| 播放 | 性能问题 | 早期性能测试 |

## 验收测试

### 压感测试
- [ ] Wacom 笔正常跟踪
- [ ] 压感曲线正确应用
- [ ] 不同笔刷预设工作

### 矢量测试
- [ ] 创建 1000+ 点路径流畅
- [ ] 编辑操作响应 < 16ms
- [ ] 与光栅图层正确混合

### 上色测试
- [ ] 复杂线稿正确检测区域
- [ ] 填充 4K 画布 < 1s
- [ ] 间隙闭合功能有效

### 播放测试
- [ ] 100 帧动画 24fps 流畅
- [ ] 内存使用 < 2GB
- [ ] 无卡顿或掉帧

---
*Phase 2 版本: 1.0*
*预估工期: 5-6 周*
*更新日期: 2026-04-21*

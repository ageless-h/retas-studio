# RETAS Studio 项目完善路线图

## 当前状态概览

根据代码库分析，当前实现状态如下：

| Crate | 状态 | 完成度 | 主要缺口 |
|-------|------|--------|----------|
| **retas-core** | 🟡 部分完成 | 85% | undo.rs 部分命令为空实现 |
| **retas-vector** | 🟡 部分完成 | 80% | tessellation.rs flatten 简化，path.rs ArcTo 未实现 |
| **retas-render** | 🔴 基础框架 | 40% | 各图层渲染为空实现，需要完成实际渲染逻辑 |
| **retas-io** | 🟢 基本完成 | 90% | 基本完整，可继续扩展格式支持 |
| **retas-ui** | 🟡 框架完成 | 60% | 文件打开/保存空实现，画布交互硬编码，播放未实现 |

---

## 阶段一：核心渲染系统 (Phase 1 - 优先级：🔴 最高)

### 目标
完成 GPU 渲染管线，使所有图层类型可正确渲染。

### 任务清单

#### 1.1 完成光栅图层渲染 (2-3天)
**文件**: `crates/retas-render/src/renderer.rs`

- [ ] 实现 `render_raster_layer` 完整逻辑
  - 从 layer 帧数据创建纹理
  - 应用混合模式（Normal, Multiply, Screen 等）
  - 应用 opacity 透明度
  - 渲染到目标纹理
- [ ] 添加混合模式 pipeline
- [ ] 添加 transform/affine 变换支持

#### 1.2 完成矢量图层渲染 (3-4天)
**文件**: `crates/retas-render/src/renderer.rs`

- [ ] 实现 `render_vector_layer`
  - 使用 retas-vector 的 tessellation 将笔画转为三角形
  - 渲染 strokes 到纹理
  - 支持 stroke style (cap, join, width)
  - 支持填充 (fill)

#### 1.3 完成摄像机图层渲染 (1-2天)
**文件**: `crates/retas-render/src/renderer.rs`

- [ ] 实现 `render_camera_layer`
  - 应用摄像机 transform (position, zoom, rotation)
  - 安全框 (safe area) 渲染

#### 1.4 完成文字图层渲染 (2天)
**文件**: `crates/retas-render/src/renderer.rs`

- [ ] 实现 `render_text_layer`
  - 字体渲染 (需要添加字体库，如 fontdue 或 cosmic-text)
  - 文字排版和渲染

#### 1.5 完善渲染管线 (2-3天)
**文件**: `crates/retas-render/src/pipeline.rs`, `shader.rs`

- [ ] 创建 blend mode pipeline 集合
- [ ] 实现常用混合模式 shaders
  - Normal, Multiply, Screen, Overlay
  - Add, Subtract, Difference
- [ ] 优化渲染性能 (批量渲染，减少 draw call)

---

## 阶段二：UI 核心功能 (Phase 2 - 优先级：🔴 最高)

### 目标
使 UI 完全可用，实现完整的用户交互流程。

### 任务清单

#### 2.1 完成文件操作 (1-2天)
**文件**: `crates/retas-ui/src/app.rs`

- [ ] 实现 `OpenDocument` 消息处理
  - 打开文件对话框
  - 调用 retas-io 加载 .retas 项目文件
  - 更新应用状态
- [ ] 实现 `SaveDocument` 消息处理
  - 保存文件对话框
  - 调用 retas-io 保存项目
  - 保存提示和覆盖确认

#### 2.2 修复画布交互 (2-3天)
**文件**: `crates/retas-ui/src/canvas.rs`

- [ ] 修复 mouse press 硬编码坐标问题
  - 使用真实鼠标位置
  - 坐标转换 (screen -> canvas)
- [ ] 完善画笔工具
  - 实时渲染笔划到画布
  - 支持 pressure/tilt
- [ ] 完善橡皮擦工具
- [ ] 添加选择工具基础支持
- [ ] 添加手型工具 (pan)
- [ ] 添加缩放工具 (zoom)

#### 2.3 完成时间轴播放 (1-2天)
**文件**: `crates/retas-ui/src/timeline.rs`

- [ ] 实现播放循环
  - 使用 async timer 或 iced subscription
  - 根据 frame_rate 计算帧间隔
- [ ] 添加洋葱皮 (onion skin) 支持
- [ ] 优化时间轴 UI 交互

#### 2.4 统一工具栏 (1天)
**文件**: `crates/retas-ui/src/toolbar.rs`, `app.rs`

- [ ] 消除 toolbar.rs 和 app.rs 的重复代码
- [ ] 统一工具切换逻辑

---

## 阶段三：Undo 系统完善 (Phase 3 - 优先级：🟡 中等)

### 目标
完成撤销/重做系统的所有命令实现。

### 任务清单

#### 3.1 完成绘图命令 (1-2天)
**文件**: `crates/retas-core/src/advanced/undo.rs`

- [ ] 实现 `StrokeCommand::execute`
  - 添加笔画到图层
- [ ] 实现 `StrokeCommand::undo`
  - 从图层移除笔画

#### 3.2 完成变换命令 (1天)
**文件**: `crates/retas-core/src/advanced/undo.rs`

- [ ] 实现 `TransformCommand::execute/undo`
  - 记录变换前后的矩阵
  - 应用/还原变换

#### 3.3 完成选择命令 (1天)
**文件**: `crates/retas-core/src/advanced/undo.rs`

- [ ] 实现 `SelectionCommand::execute/undo`

#### 3.4 完成填充命令 (1天)
**文件**: `crates/retas-core/src/advanced/undo.rs`

- [ ] 实现 `FillCommand::execute/undo`

---

## 阶段四：矢量系统完善 (Phase 4 - 优先级：🟡 中等)

### 目标
完善矢量图形处理，特别是曲面细分。

### 任务清单

#### 4.1 完善 Tessellation (2-3天)
**文件**: `crates/retas-vector/src/tessellation.rs`

- [ ] 实现真正的 `flatten_stroke`
  - 使用自适应细分
  - 处理曲线平滑度
- [ ] 改进 path fill tessellation
  - 支持复杂填充规则 (EvenOdd, NonZero)
  - 处理自交路径

#### 4.2 完成 ArcTo 支持 (1天)
**文件**: `crates/retas-vector/src/path.rs`

- [ ] 实现 `ArcTo` 到 Bezier 曲线转换
  - 使用近似算法将圆弧转为三次贝塞尔

---

## 阶段五：工具系统实现 (Phase 5 - 优先级：🔴 高)

### 目标
实现 RETAS 主要工具，对应 CLASS_REFERENCE.md 中的工具类。

### 任务清单

#### 5.1 基础绘图工具 (3-4天)
**文件**: 新建 `crates/retas-core/src/tools/`

- [ ] 实现 Pencil Tool (铅笔)
- [ ] 实现 Pen Tool (钢笔) - 贝塞尔曲线绘制
- [ ] 实现 Brush Tool (笔刷) - 已有基础，需完善
- [ ] 实现 Eraser Tool (橡皮) - 已有基础，需完善
- [ ] 实现 Airbrush Tool (喷枪)

#### 5.2 填充工具 (2-3天)
**文件**: `crates/retas-core/src/tools/`

- [ ] 实现 Bucket Fill (油漆桶)
  - 基于颜色的 flood fill
  - 支持 tolerance 容差
- [ ] 实现 Consecutive Fill (连续填充)
- [ ] 实现 Gradient Tool (渐变)

#### 5.3 选择工具 (2-3天)
**文件**: `crates/retas-core/src/tools/`

- [ ] 实现 Rectangle Select (矩形选择)
- [ ] 实现 Lasso Select (套索选择)
- [ ] 实现 Magic Wand (魔术棒) - 已有基础，需完善

#### 5.4 变换工具 (2天)
**文件**: `crates/retas-core/src/tools/`

- [ ] 实现 Move Tool (移动)
- [ ] 实现 Rotate Tool (旋转)
- [ ] 实现 Scale Tool (缩放)

#### 5.5 其他工具 (2-3天)
**文件**: `crates/retas-core/src/tools/`

- [ ] 实现 Line Tool (直线)
- [ ] 实现 Shape Tool (形状 - 矩形、椭圆、多边形)
- [ ] 实现 Text Tool (文字)
- [ ] 实现 Spuit Tool (吸管)

---

## 阶段六：文件格式支持 (Phase 6 - 优先级：🔴 高)

### 目标
实现对 RETAS 原生格式的完全支持，以及常用导入/导出格式。

### 任务清单

#### 6.1 原生格式完善 (2-3天)
**文件**: `crates/retas-io/src/`

- [ ] 完善 .retas 项目格式
  - manifest.json
  - layer 数据序列化
  - frame 图像数据存储
- [ ] 优化读写性能

#### 6.2 RETAS 旧版格式支持 (3-4天)
**文件**: `crates/retas-io/src/`

- [ ] 实现 CEL 格式读写 (已有基础，需完善)
- [ ] 实现 DGA 格式读写 (已有基础，需完善)
- [ ] 实现 SCS 格式读写 (已有基础，需完善)
- [ ] 测试与原版 RETAS 的兼容性

#### 6.3 导入/导出格式扩展 (2-3天)
**文件**: `crates/retas-io/src/export.rs`

- [ ] 完善图像导出
  - PNG (已有)
  - JPEG (已有)
  - TIFF
  - PSD (Photoshop)
- [ ] 完善动画导出
  - GIF (已有)
  - MP4/WebM/AVI/MOV (已有框架，需完善)
- [ ] 添加 SVG 导出
- [ ] 添加 OpenRaster (.ora) 支持

---

## 阶段七：高级功能 (Phase 7 - 优先级：🟢 低)

### 目标
实现专业动画软件的高级功能。

### 任务清单

#### 7.1 动画系统 (3-4天)
**文件**: `crates/retas-core/src/advanced/`

- [ ] 完善关键帧系统
- [ ] 实现补间动画 (tweening)
- [ ] 实现洋葱皮 (onion skin) 预览
- [ ] 实现音频同步

#### 7.2 滤镜系统 (2-3天)
**文件**: `crates/retas-core/src/advanced/effects.rs`

- [ ] 实现常用滤镜
  - Blur (Gaussian, Motion)
  - Sharpen
  - Color Balance
  - Level/Curve adjustment
- [ ] 实现滤镜预览
- [ ] 实现滤镜 stack

#### 7.3 批处理系统 (2-3天)
**文件**: `crates/retas-core/src/advanced/batch.rs`

- [ ] 完善批处理队列
- [ ] 实现批处理 UI

#### 7.4 打印系统 (1-2天)
**文件**: `crates/retas-core/src/advanced/print.rs`

- [ ] 实现打印预览
- [ ] 实现打印布局

---

## 实施建议

### 推荐开发顺序

1. **第一周**: Phase 1 (渲染系统) + Phase 2 (UI 文件操作)
2. **第二周**: Phase 2 (画布交互) + Phase 3 (Undo 系统)
3. **第三周**: Phase 5 (基础工具)
4. **第四周**: Phase 6 (文件格式) + Phase 4 (矢量完善)
5. **第五周起**: Phase 7 (高级功能) + 持续优化

### 技术要点

- **渲染**: 使用 WGPU 的 render pipeline，为每种 blend mode 创建 pipeline
- **工具系统**: 参考 CLASS_REFERENCE.md 的 CCTool* 类设计
- **文件格式**: 优先保证 .retas 原生格式的稳定，再扩展兼容性
- **UI**: 使用 Iced 的 subscription 实现播放循环

### 依赖添加

可能需要在 Cargo.toml 中添加：
```toml
# 字体渲染
fontdue = "0.9"

# 图像处理扩展
image = { version = "0.24", features = ["png", "jpeg", "tiff", "bmp"] }

# 视频编码
rav1e = "0.7"  # AV1 编码

# 异步运行时
tokio = { workspace = true }
```

---

## 完成标准

### 最小可用产品 (MVP)
- [x] ✅ 所有图层类型可渲染
- [x] ✅ 基础绘图工具可用 (铅笔、笔刷、橡皮)
- [x] ✅ 文件打开/保存功能
- [x] ✅ 撤销/重做功能
- [x] ✅ 时间轴基础功能

### 完整功能
- [ ] 所有工具实现
- [ ] 完整文件格式支持
- [ ] 滤镜系统
- [ ] 动画导出
- [ ] 批处理

---

*路线图创建日期: 2026-04-21*
*基于代码库分析和 CLASS_REFERENCE.md 制定*

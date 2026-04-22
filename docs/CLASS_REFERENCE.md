# RETAS STUDIO 类结构完整文档

## 概述

通过逆向分析从 RETAS STUDIO 6.6.0 提取的所有 C++ 类。

---

## 一、基础框架类 (RC 前缀)

来自 `rcwCmn.dll` - 核心工具库

### 数据类型
```
RCString          - 字符串
RCStringArray     - 字符串数组
RCNumber          - 数字包装
RCLength          - 长度单位

RCPoint           - 整数点
RCPointD          - 双精度点
RCPointF          - 浮点点
RCRect            - 矩形
RCRectD           - 双精度矩形
RCRectF           - 浮点矩形
RCSize            - 尺寸
RCSizeD           - 双精度尺寸
RCSizeF           - 浮点尺寸
```

### 颜色
```
RCRgbColor        - RGB颜色
RCHsvColor        - HSV颜色
RCComplexColor    - 复合颜色
```

### 文件系统
```
RCFile            - 文件操作
RCFilePath        - 文件路径
RCFindFile        - 文件查找
RCArchive         - 序列化基类
RCArchiveFile     - 文件序列化
RCArchiveMem      - 内存序列化
RCPreferenceFile  - 配置文件
```

### 图形
```
RCGdi             - GDI 绘图上下文
RCBitmap          - 位图
RCBitImage        - 位图图像
RCPict            - 图像
RCPixelMixer      - 像素混合器基类
RCPixelMixerAdd   - 加法混合
RCPixelMixerAlphaBlend - Alpha混合
RCPixelMixerAlphaCopy  - Alpha复制
RCPixelMixerBlend      - 普通混合
RCPixelMixerGaussian   - 高斯混合
RCPixelMixerGradation  - 渐变混合
RCPixelMixerWeightCopy - 权重复制

RCVOffscreen      - 离屏渲染
RCVOffscreenCache - 离屏缓存
RCVOffPixelMask   - 像素遮罩
RCVOffPixelMixer  - 像素混合器
```

### UI 控件
```
RCWindow          - 窗口基类
RCControl         - 控件基类
RCButton          - 按钮
RCMenu            - 菜单
RCMenuBar         - 菜单栏
RCScrollBar       - 滚动条
RCCursor          - 光标
RCFont            - 字体
RCRgn             - 区域
```

### 系统
```
RCThread          - 线程
RCCriticalSection - 临界区
RCRegKey          - 注册表键
RCTime            - 时间
RCException       - 异常
RCAlert           - 警告框
RCPointer         - 智能指针
RCHandle          - 句柄包装
```

---

## 二、应用基础类 (CC 前缀)

### 应用核心
```
CCBaseApp         - 应用基类
CCBaseWindow      - 窗口基类
CCBaseControl     - 控件基类
CCDocument        - 文档基类
CCDocWindow       - 文档窗口
CCAppCommand      - 应用命令
CCEventHandler    - 事件处理器
```

### UI 组件
```
CCCheckBox        - 复选框
CCDropDownBox     - 下拉框
CCStatic          - 静态控件
CCSliderButton    - 滑块按钮
CCBitmapButton    - 位图按钮
CCBitmapButtonRes - 位图按钮资源
CCBitmapScrollBarBase - 位图滚动条
CCSeparator       - 分隔符
CCPopupList       - 弹出列表
CCPopupSlider     - 弹出滑块
CCPopupWindow     - 弹出窗口
```

### 视图系统
```
CCView            - 视图基类
CCPlaneView       - 平面视图
CCElementView     - 元素视图
CCLayerView       - 图层视图
```

### 撤销/重做
```
CCCelDoItem       - 操作项基类
CCCelDoManager    - 操作管理器
CCDelayUndo       - 延迟撤销
```

---

## 三、图层系统

### 图层基类
```
CCLayer           - 图层基类
CCLayerArray      - 图层数组
CCAdjustLayer     - 调整图层
CCFloatingLayer   - 浮动图层
CCFrameLayer      - 帧图层
CCGridLayer       - 网格图层
CCGuideLayer      - 参考线图层
CCRulerLayer      - 尺图层
CCShapeLayer      - 形状图层
CCTextLayer       - 文字图层
CCVanishingPointLayer - 透视点图层
```

### 光栅图层
```
CCRasterLayer     - 光栅图层基类
CCRasNormalLayer  - 普通图层
CCRasDrawLayer    - 描线图层
CCRasDraftLayer   - 草稿图层
CCRasIndicateLayer - 指示图层
CCRasSelectLayer  - 选择图层
CCRasTempLineLayer - 临时线图层
CCRasPaintMonoLayer - 单色上色图层
CCRasPaintGradLayer - 渐变上色图层
```

### 矢量图层
```
CCVectorLayer     - 矢量图层基类
CCVectorLineLayer - 矢量线图层
CCVectorPaintLayer - 矢量上色图层
CCVectorSelectLayer - 矢量选择图层
CCVectorFaceLayer - 矢量正面图层
CCVectorTempLineLayer - 矢量临时线图层
CCVectorBorderPlane - 矢量边框平面
CCVectorVertLinePlane - 矢量垂直线平面
CCVectorVertPlane - 矢量垂直平面
```

### 平面 (Plane - 图层内的绘制区域)
```
CCPlane           - 平面基类
CCPlaneArray      - 平面数组
CCAdjustPlane     - 调整平面
CCFramePlane      - 帧平面
CCOriginFramePlane - 原始帧平面
CCCameraFramePlane - 摄像机帧平面
CCRasterPlane     - 光栅平面
CCRasDrawLinePlane - 描线平面
CCRasLineMonoPlane - 单色线平面
CCRasLineGradPlane - 渐变线平面
CCRasPaintPlane   - 上色平面
CCVectorLinePlane - 矢量线平面
CCVectorPaintPlane - 矢量上色平面
CCShapePlane      - 形状平面
CCTextPlane       - 文字平面
CCRulerPlane      - 尺平面
CCVanishingPointPlane - 透视点平面
```

---

## 四、工具系统

### 工具基类
```
CCTool            - 工具基类
CCToolParam       - 工具参数基类
CCToolManager     - 工具管理器
CCToolGroup       - 工具组
CCToolKeyItem     - 工具快捷键项
```

### 绘图工具
```
CCPenTool         - 钢笔工具
CCPenToolParam    - 钢笔参数
CCPencilTool      - 铅笔工具
CCPencilObject    - 铅笔对象
CCBrushTool       - 笔刷工具
CCBrushToolParam  - 笔刷参数
CCBrushBaseTool   - 笔刷基类
CCAirbrushTool    - 喷枪工具
CCAirBrushEffectTool - 喷枪效果工具
CCEraserTool      - 橡皮工具
CCEraserToolParam - 橡皮参数
```

### 填充工具
```
CCBucketTool      - 油漆桶工具
CCBucketToolParam - 油漆桶参数
CCConsecutiveFillTool - 连续填充工具
CCSpreadFillTool  - 扩散填充工具
CCCloseFillTool   - 闭合填充工具
CCGradationTool   - 渐变工具
```

### 选择工具
```
CCSelectTool      - 选择工具基类
CCSelectRectTool  - 矩形选择
CCSelectLassoTool - 套索选择
CCSelectWandTool  - 魔术棒选择
CCSelectTraceTool - 描边选择
CCSelectObjectTool - 对象选择
CCSelectArea      - 选择区域
CCSelectWand      - 魔术棒
```

### 变换工具
```
CCTransformTool   - 变换工具
CCRotateTool      - 旋转工具
CCZoomTool        - 缩放工具
CCHandTool        - 手型工具
CCLayerMoveTool   - 图层移动工具
CCFrameMoveTool   - 帧移动工具
```

### 特殊工具
```
CCLineTool        - 线工具
CCLineToolBase    - 线工具基类
CCPolylineTool    - 多段线工具
CCCurveTool       - 曲线工具
CCShapeTool       - 形状工具
CCTextTool        - 文字工具
CCSpuitTool       - 吸管工具
CCStampTool       - 图章工具
CCStampBaseTool   - 图章基类
CCFilterTool      - 滤镜工具
```

### 线条处理工具
```
CCLinePushTool    - 线推工具
CCLineSmoothTool  - 线平滑工具
CCLineVolumeTool  - 线粗细工具
CCJointLineTool   - 连接线工具
```

### 透视和参考
```
CCVanishingPointMoveTool - 透视点移动工具
CCLightMoveTool   - 灯光移动工具
CCLightMoveFromPointTool - 从点移动灯光
```

### 其他工具
```
CCBlurTool        - 模糊工具
CCDustTool        - 灰尘工具
CCTextureTool     - 纹理工具
CCColorPicker     - 颜色选择器
```

---

## 五、时间轴/摄影表系统

### 摄影表核心
```
CCScoreDocument   - 摄影表文档
CCScoreWindow     - 摄影表窗口
CCScoreView       - 摄影表视图
CCScoreLayerInfo  - 图层信息
CCScoreObject     - 摄影表对象
CCScorePencilObject - 铅笔对象
CCScoreArrowObject - 箭头对象
CCScoreFadeInObject - 淡入对象
CCScoreFadeOutObject - 淡出对象
CCScoreOverlapObject - 重叠对象
CCScoreTextObject - 文字对象
```

### 摄影表操作
```
CCScoreDoItem     - 摄影表操作项
CCScoreDoManager  - 摄影表操作管理器
CCScoreDoItemInsertFrame - 插入帧
CCScoreDoItemDeleteFrame - 删除帧
CCScoreDoItemInsertLayer - 插入图层
CCScoreDoItemEnableLayer - 启用图层
CCScoreDoItemRenameLayer - 重命名图层
CCScoreDoItemSwapLayer - 交换图层
CCScoreDoItemPencilObject - 铅笔对象操作
CCScoreDoItemDirection - 方向操作
CCScoreDoItemVoice - 音频操作
CCScoreDoItemScoreInfo - 摄影表信息
```

### 关键帧
```
CCScoreDoItemKeyFrame - 关键帧操作
CCScoreDoItemMotionPath - 运动路径操作
CCScoreDoItemKeyPtrInPath - 路径关键点
```

### CoreRETAS 摄影表扩展
```
CCCScoreDoItem    - CoreRETAS摄影表操作项
CCCScoreDoManager - CoreRETAS操作管理器
CCCScoreDoItemAddEffectLayer - 添加效果图层
CCCScoreDoItemCameraLayer - 摄像机图层操作
CCCScoreDoItemCelBankLayer - 赛璐珞库图层操作
CCCScoreDoItemCelBankNumber - 赛璐珞库编号
CCCScoreDoItemCelLayer - 赛璐珞图层操作
CCCScoreDoItemCelNumber - 赛璐珞编号
CCCScoreDoItemEffectLayer - 效果图层操作
CCCScoreDoItemTapLayer - Tap图层操作
```

---

## 六、赛璐珞系统

### 赛璐珞核心
```
CCCelDocument     - 赛璐珞文档
CCCelInfo         - 赛璐珞信息
CCCelBaseInfo     - 赛璐珞基础信息
CCCelFolder       - 赛璐珞文件夹
CCCelView         - 赛璐珞视图
CCCelWindow       - 赛璐珞窗口
```

### 赛璐珞操作
```
CCCelDoItem       - 赛璐珞操作项
CCCelDoManager    - 赛璐珞操作管理器
CCCelDoItemAddLayer - 添加图层
CCCelDoItemAddPlane - 添加平面
CCCelDoItemAdjustPlane - 调整平面
CCCelDoItemConvertLayer - 转换图层
CCCelDoItemConvertPlane - 转换平面
CCCelDoItemDocModify - 文档修改
CCCelDoItemElement - 元素操作
CCCelDoItemFloating - 浮动操作
CCCelDoItemFramePlane - 帧平面操作
CCCelDoItemGuide - 参考线操作
CCCelDoItemLayerMoveRas - 光栅图层移动
CCCelDoItemLayerMoveVec - 矢量图层移动
CCCelDoItemMulti - 多重操作
CCCelDoItemOrderLayer - 图层排序
CCCelDoItemOrderPlane - 平面排序
CCCelDoItemRasterPlane - 光栅平面操作
CCCelDoItemRuler - 尺操作
CCCelDoItemSelect - 选择操作
CCCelDoItemShape - 形状操作
CCCelDoItemTextPlane - 文字平面操作
CCCelDoItemVanishingPointPlane - 透视点平面操作
CCCelDoItemVecDraw - 矢量绘制操作
CCCelDoItemVecPaint - 矢量上色操作
```

### 赛璐珞库 (CoreRETAS)
```
CCCelBankInfo     - 赛璐珞库信息
CCCelBankPalette  - 赛璐珞库面板
CCCelCache        - 赛璐珞缓存
CCCelCacheInfo    - 赛璐珞缓存信息
```

---

## 七、渲染/合成系统 (CoreRETAS)

### 合成核心
```
CCComposite       - 合成器
CCCompositeRender - 合成渲染器
CCCompositeQueue  - 合成队列
CCCompositeScore  - 合成摄影表
CCCompositeView   - 合成视图
CCCompositeWindow - 合成窗口
```

### 合成图层
```
CCCompositeLayer  - 合成图层基类
CCCompositeLayerInfo - 合成图层信息
CCCompositeCameraLayer - 摄像机图层
CCCompositeCelLayer - 赛璐珞图层
CCCompositeCelBankLayer - 赛璐珞库图层
CCCompositeEffectLayer - 效果图层
CCCompositeTapLayer - Tap图层
CCCompositeSubCelLayer - 子赛璐珞图层
CCCompositeSoundLayer - 音频图层
```

### 合成线程
```
CCCompositeRenderThread - 渲染线程
CCCompositeLayerThread - 图层线程
CCCompositeCameraThread - 摄像机线程
```

### 合成缓存
```
CCCompositeCache  - 合成缓存
CCCompositeCacheInfo - 缓存信息
CCBaseCache       - 缓存基类
CCBaseCacheInfo   - 缓存信息基类
```

### 合成事件
```
CCCompositeEvent  - 合成事件基类
CCCompositeBlockEvent - 块事件
CCCompositeErrorEvent - 错误事件
CCCompositeFrameEvent - 帧事件
CCCompositeFrameReceiveEvent - 帧接收事件
CCCompositeInternalEvent - 内部事件
CCCompositeNotify - 合成通知
```

### 合成信息
```
CCCompositeFrameInfo - 帧信息
CCCompositeOutputInfo - 输出信息
CCCompositeRenderInfo - 渲染信息
```

### SWF导出
```
CCCompositeSwf    - SWF合成器
```

---

## 八、滤镜系统

### 滤镜基类
```
CCFilter          - 滤镜基类
CCFilterTool      - 滤镜工具
CCFilterToolParam - 滤镜工具参数
CCFilterDlgData   - 滤镜对话框数据
```

### 模糊滤镜
```
CCFilterBlurGauss - 高斯模糊
CCFilterBlurHard  - 硬模糊
CCFilterBlurSoft  - 软模糊
```

### 色彩滤镜
```
CCFilterColorBalance - 色彩平衡
CCFilterContrast     - 对比度
CCFilterContrastAuto - 自动对比度
CCFilterHue          - 色相
CCFilterLevel        - 色阶
CCFilterToneCurve    - 曲线
CCFilterInvert       - 反转
```

### 锐化滤镜
```
CCFilterSharpHard - 硬锐化
CCFilterSharpSoft - 软锐化
CCFilterUnSharpMask - USM锐化
```

### 其他滤镜
```
CCFilterDust      - 灰尘滤镜
```

---

## 九、批处理系统

### 批处理核心
```
CCBatchItem       - 批处理项
CCBatchList       - 批处理列表
CCBatchSet        - 批处理设置
CCBatchSetData    - 批处理设置数据
CCBatchPalette    - 批处理面板
CCBatchView       - 批处理视图
```

### 批处理操作
```
CCBatchAirBrush   - 喷枪批处理
CCBatchBlurGauss  - 高斯模糊批处理
CCBatchBlurHard   - 硬模糊批处理
CCBatchBlurSoft   - 软模糊批处理
CCBatchColorBalance - 色彩平衡批处理
CCBatchColorReplace - 颜色替换批处理
CCBatchConsecutiveFill - 连续填充批处理
CCBatchContrast   - 对比度批处理
CCBatchContrastAuto - 自动对比度批处理
CCBatchConvertLayer - 图层转换批处理
CCBatchConvertLayerToPaint - 转换为上色图层
CCBatchConvertLayerToRasterNormal - 转换为普通光栅图层
CCBatchDust       - 灰尘批处理
CCBatchHsv        - HSV批处理
CCBatchInvert     - 反转批处理
CCBatchLayerVisibility - 图层可见性批处理
CCBatchLevel      - 色阶批处理
CCBatchLineSmooth - 线平滑批处理
CCBatchLineVolume - 线粗细批处理
CCBatchReverseAndRotate - 反转旋转批处理
CCBatchSharpHard  - 硬锐化批处理
CCBatchSharpSoft  - 软锐化批处理
CCBatchSplit      - 分割批处理
CCBatchToneCurve  - 曲线批处理
CCBatchUnSharpMask - USM锐化批处理
CCBatchTrace      - 描线批处理 (TraceMan)
CCBatchTraceGray  - 灰度描线批处理
CCBatchTraceVector - 矢量描线批处理
```

### 批处理UI
```
CCBatchColumnCommon - 批处理列公共
CCBatchColumnView   - 批处理列视图
CCBatchFillPreviewDlgData - 填充预览对话框数据
```

---

## 十、导入导出系统

### 导出
```
CCExportFile      - 文件导出
CCExportFileParam - 文件导出参数
CCExportEpsFile   - EPS导出
CCExportSwfFile   - SWF导出
CCExportColorFile - 颜色文件导出
CCExportPreview   - 导出预览
CCCutExporter     - Cut导出器
```

### Cut管理
```
CCCutFolder       - Cut文件夹
CCCutFolderCache  - Cut文件夹缓存
CCCutDirection    - Cut方向
CCCutProcess      - Cut处理
CCCutProcessCheck - Cut处理检查
```

---

## 十一、面板/调色板系统

### 图层面板
```
CCLayerPalette    - 图层面板
CCLayerPaletteObject - 图层面板对象
```

### 工具面板
```
CCToolPalette     - 工具面板
CCToolPaletteObject - 工具面板对象
CCToolOptionPalette - 工具选项面板
CCToolOptionPaletteObject - 工具选项面板对象
```

### 颜色面板
```
CCColorPalette    - 颜色面板
CCColorPaletteObject - 颜色面板对象
CCColorBox        - 颜色框
CCColorButton     - 颜色按钮
CCColorSlider     - 颜色滑块
CCColorChartPalette - 色卡面板
CCColorChartPaletteObject - 色卡面板对象
CCColorLocator    - 颜色定位器
CCColorLocatorObject - 颜色定位器对象
```

### 文件预览面板
```
CCFilePreviewerPalette - 文件预览面板
CCFilePreviewerPaletteObject - 文件预览面板对象
```

### 透光台面板
```
CCLightTablePalette - 透光台面板
CCLightTablePaletteObject - 透光台面板对象
CCLightTable      - 透光台
CCLightTableManager - 透光台管理器
CCLightTableSet   - 透光台设置
```

### 批处理面板
```
CCBatchPalette    - 批处理面板
CCBatchPaletteObject - 批处理面板对象
```

### 子面板
```
CCSubPalette      - 子面板
```

---

## 十二、浏览系统

```
CCBrowseWindow    - 浏览窗口
CCBrowseWindowObject - 浏览窗口对象
CCBrowseView      - 浏览视图
CCBrowseItem      - 浏览项
CCBrowseListView  - 列表视图
CCBrowseCutView   - Cut视图
CCBrowseDirectionView - 方向视图
CCBrowseInfoView  - 信息视图
CCBrowseMemoView  - 备忘录视图
CCBrowseProcessView - 处理视图
CCBrowseTreeView  - 树形视图
```

---

## 十三、动画系统

### 动画纸
```
CCAnimationPaper  - 动画纸
CCAnimationPaperManager - 动画纸管理器
```

### 运动检查
```
CCMotionCheck     - 运动检查
CCWMotionCheckWindow - 运动检查窗口
CCWMotionCheckDlg - 运动检查对话框
```

---

## 十四、快捷键系统

```
CCAccelerator     - 加速器
CCAccelItem       - 加速器项
CCToolKeyItem     - 工具快捷键项
```

---

## 十五、剪贴板/拖放

```
CCClipBoard       - 剪贴板
CCDropServer      - 拖放服务器
CCLVDropSource    - 列表视图拖放源
CCLVDropTarget    - 列表视图拖放目标
CCLVDataObject    - 列表视图数据对象
CCLVEnumFORMATETC - 列表视图格式枚举
CCCQDropSource    - 复合队列拖放源
CCCQDropTarget    - 复合队列拖放目标
CCCQDataObject    - 复合队列数据对象
CCCQEnumFORMATETC - 复合队列格式枚举
```

---

## 十六、变换矩阵

```
CCTransform       - 变换
CCMatrixEngine    - 矩阵引擎
```

---

## 十七、打印系统

```
CCCelPrint        - 赛璐珞打印
CCScorePrint      - 摄影表打印
```

---

## 十八、对话框数据类 (DlgData后缀)

数百个对话框数据类用于存储UI状态，命名模式为：
- `CC[功能][动作]DlgData`
- 例如: `CCNewLayerDlgData`, `CCScoreInsertFrameDlgData`

---

## 十九、对话框窗口类 (CCW前缀)

```
CCWMainFrm        - 主框架窗口
CCWMotionCheckWindow - 运动检查窗口
CCWMotionCheckDlg - 运动检查对话框
CCWPrefDlg        - 首选项对话框
CCWPrefCelDlg     - 赛璐珞首选项
CCWPrefColorDlg   - 颜色首选项
CCWPrefFileDlg    - 文件首选项
CCWPrefMemoryDlg  - 内存首选项
CCWPrefMonitorDlg - 显示器首选项
CCWPrefPaperDlg   - 纸张首选项
CCWPrefPlugInDlg  - 插件首选项
CCWPrefScoreDlg   - 摄影表首选项
... (数百个对话框类)
```

---

## 二十、特殊系统

### 矢量化 (TraceMan)
```
CCVectorizeLine   - 矢量化线条
CCBatchTrace      - 批处理描线
CCBatchTraceGray  - 灰度描线
CCBatchTraceVector - 矢量描线
```

### 智能绘制
```
CCSmartDraw       - 智能绘制
CCTempLineOp      - 临时线操作
CCTempPenLineOp   - 临时钢笔线操作
CCTempRasterPenLineOp - 临时光栅钢笔线操作
CCTempSimpleLineOp - 临时简单线操作
CCThining         - 细化
```

### 音频
```
CCSoundInfoDlgData - 音频信息对话框数据
```

### 缓存
```
CCBaseCache       - 缓存基类
CCBaseCacheInfo   - 缓存信息基类
CCBaseCacheObject - 缓存对象基类
CCCacheManager    - 缓存管理器
CCCacheAutoRecordProcessingTime - 缓存自动记录处理时间
```

---

## 类数量统计

| 模块 | 类数量 |
|------|--------|
| 基础框架 (RC前缀) | ~80 |
| 应用核心 (CC前缀) | ~600+ |
| Stylos专用 | ~50 |
| PaintMan专用 | ~30 |
| CoreRETAS专用 | ~100 |
| TraceMan专用 | ~10 |
| **总计** | **~850+** |

---

*文档生成日期: 2026-04-20*
*来源: RETAS STUDIO 6.6.0 二进制逆向分析*

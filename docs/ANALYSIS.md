# RETAS STUDIO 逆向分析与 macOS 移植方案

## 软件概述

**RETAS STUDIO** 是由 CELSYS 开发的专业2D动画制作套件，版本 6.6.0。

### 组件构成

| 组件 | 用途 | 主程序大小 |
|------|------|-----------|
| Stylos | 描线和动画制作 | 7.6 MB |
| PaintMan | 上色工具 | 7.6 MB |
| TraceMan | 矢量描线 | 7.9 MB |
| CoreRETAS | 合成和渲染 | 7.5 MB |

## 技术架构分析

### 构建技术栈
- **框架**: MFC42 (Microsoft Foundation Classes 4.2)
- **编译器**: Visual Studio 7.10 (Visual Studio .NET 2003)
- **架构**: 32位 Windows PE (i386)
- **GUI**: Win32 GDI
- **网络**: WinINET, WinSock2

### 依赖的 Windows DLL

#### 系统库
- `KERNEL32.dll` - Windows 核心API
- `USER32.dll` - 窗口管理
- `GDI32.dll` - 图形设备接口
- `COMCTL32.dll` - 公共控件
- `COMDLG32.dll` - 公共对话框
- `ADVAPI32.dll` - 高级API
- `SHELL32.dll` - Shell接口
- `SHLWAPI.dll` - Shell轻量级API
- `OLE32.dll` - OLE/COM支持
- `WS2_32.dll` - WinSock 2
- `WININET.dll` - Internet API
- `iphlpapi.dll` - IP辅助API
- `MSIMG32.dll` - 图像处理
- `VERSION.dll` - 版本信息

#### 运行时库
- `MFC42.dll` - Microsoft Foundation Classes (需要安装)
- `mscoree.dll` - .NET 运行时引用

#### 自定义库
- `rcwCmn.dll` - RETAS公共库 (847 KB)
- `rcwRes.dll` - RETAS资源库 (2.1 MB)
- `rtwUtl.dll` - RETAS工具库 (203 KB)
- `zlib.dll` - 压缩库

#### 保护系统
- `HASPUT16.DLL` - HASP 加密狗保护驱动

### 插件系统

软件使用模块化插件架构：

#### 文件格式插件 (PlugIn/File/)
- AVI, Flash, QuickTime 视频格式
- BMP, JPEG, PNG, TIFF, TGA 图像格式
- Photoshop, SGI, FBX, XSI 兼容
- MXS 专有格式

#### 效果插件 (PlugIn/Effect/)
- Alpha, Blur, Glow, Shadow
- Motion Blur, Lens Flare
- Color Balance, Level, HSV
- 67种效果插件

### 许可机制

1. **序列号验证**: 存储在注册表 `SOFTWARE\CELSYS\RETAS STUDIO`
2. **HASP 加密狗**: 引用 `HASPUT16.DLL`
3. **share.sys**: 配置文件（中文资源）

## macOS 移植挑战

### 1. 架构差异
- **源**: x86 32位 Windows
- **目标**: ARM64 macOS (Apple Silicon)
- **需要**: Rosetta 2 或 Wine ARM64 兼容层

### 2. 图形系统
- WinGDI 是 Windows 特有的图形系统
- macOS 使用 Quartz/Core Graphics
- 需要 Wine 的 GDI 实现

### 3. MFC42 兼容性
- MFC 是 Windows 特有的 C++ 框架
- Wine 需要内置 MFC 实现
- 可能需要原生 MFC42.dll

### 4. HASP 加密狗
- 硬件保护锁驱动
- Wine 兼容性问题
- 需要特殊处理

## 移植方案

### 方案 A: Wine/CrossOver (推荐)

**优点**:
- 无需修改原始代码
- CrossOver 提供 Apple Silicon 支持
- 用户界面保持一致

**步骤**:
1. 安装 CrossOver 或 Wine
2. 安装 MFC42 运行时
3. 配置 Wine prefix
4. 处理 HASP 保护问题
5. 测试和调优

### 方案 B: 虚拟机

**优点**:
- 100% 兼容性
- 已有成功案例

**缺点**:
- 性能开销
- 需要购买虚拟化软件
- 需要Windows许可证

**选项**:
- Parallels Desktop (Apple Silicon)
- VMware Fusion
- UTM (免费)

### 方案 C: 完全重写

**优点**:
- 原生 macOS 体验
- 可利用现代技术

**缺点**:
- 工作量巨大
- 需要专业团队
- 时间成本高

## 下一步行动

1. 测试 Wine/CrossOver 兼容性
2. 研究 HASP 保护绕过方案
3. 收集更多技术信息
4. 制定详细实施计划

---
*分析日期: 2026-04-20*
*分析者: Sisyphus*

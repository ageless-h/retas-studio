# UTM 虚拟机安装指南

## 简介

UTM 是一个免费的虚拟化软件，可以在 Apple Silicon Mac 上运行 Windows。

## 安装步骤

### 1. 创建 Windows 11 ARM 虚拟机

1. 打开 UTM
2. 点击 "Create a New Virtual Machine"
3. 选择 "Virtualize" (使用 Apple Virtualization)
4. 选择 "Windows"
5. 按照向导完成设置

### 2. 获取 Windows 11 ARM

从 Microsoft 官网下载 Windows 11 ARM ISO:
https://www.microsoft.com/software-download/windowsinsiderpreviewARM64

### 3. 安装 Windows

1. 启动虚拟机
2. 按照安装向导安装 Windows
3. 安装完成后关闭虚拟机

### 4. 配置共享文件夹

1. 在 UTM 中选择虚拟机
2. 点击 "Settings" -> "Sharing"
3. 添加 RETAS STUDIO 目录

### 5. 安装 RETAS STUDIO

在 Windows 虚拟机中：
1. 打开共享文件夹
2. 复制 RETAS STUDIO 文件到 Windows
3. 运行应用程序

## 性能优化

### 内存设置
- 推荐分配 8GB 或更多内存
- 4核 CPU 或更多

### 显示设置
- 启用 Retina 显示
- 调整分辨率到最佳状态

### USB 设备
- 如有加密狗，配置 USB 透传
- Settings -> USB -> 添加设备

## 注意事项

1. Windows 11 ARM 可以运行 x86 应用（通过模拟）
2. 部分功能可能需要调整
3. 性能可能不如原生 Windows

## 故障排除

### 虚拟机无法启动
- 检查 macOS 安全设置
- 允许 UTM 控制电脑

### 网络问题
- 重置网络适配器
- 使用桥接模式

### 显示问题
- 更新 Windows 显示驱动
- 调整虚拟机分辨率

# RETAS STUDIO macOS 移植计划

## 项目概述

**目标**: 将 RETAS STUDIO 6.6.0 (Windows) 移植到 macOS Apple Silicon

**当前环境**: 
- macOS 26.4 (Tahoe)
- Apple Silicon (ARM64)
- 软件来源: Windows 32位 PE 可执行文件

---

## 研究发现摘要

### 软件技术栈
- **框架**: MFC42 (Microsoft Foundation Classes 4.2)
- **编译器**: Visual Studio 7.10 (VS .NET 2003)
- **架构**: 32位 x86 Windows
- **GUI**: Win32 GDI
- **保护**: HASP 加密狗 (HASPUT16.DLL)

### 已验证的兼容性信息

#### CrossOver/Wine 状态
- ✅ 有用户成功在 CrossOver 上运行 RETAS STUDIO
- ⚠️ 部分UI元素不可用（工具属性面板）
- ❌ Celsys 不官方支持 CrossOver 环境
- ❌ MFC42 不包含在 Wine 中，需要手动安装

#### HASP 保护问题
- ❌ Wine 不支持 HASP 硬件加密狗
- ❌ 无法直接在 Wine 下使用加密狗
- ⚠️ 需要特殊处理（VM + USB透传 或 模拟器）

---

## 移植方案

### 方案 1: Parallels Desktop (推荐 - 最高兼容性)

**优点**:
- 100% 兼容性保证
- 官方支持 Windows 11 ARM
- USB 设备透传（支持加密狗）
- DirectX 完整支持

**缺点**:
- 需要订阅费用 ($99.99/年)
- 需要 Windows 许可证
- 资源占用较高

**实施步骤**:
```
1. 安装 Parallels Desktop
2. 创建 Windows 11 ARM 虚拟机
3. 安装 RETAS STUDIO
4. 配置 USB 透传（如使用加密狗）
5. 测试所有模块
```

### 方案 2: CrossOver (中等成本)

**优点**:
- 无需 Windows 许可证
- 原生 macOS 集成
- 已有成功案例
- 14天免费试用

**缺点**:
- 部分功能可能不可用
- 需要处理 MFC42 依赖
- HASP 保护问题需要解决
- 无官方支持

**实施步骤**:
```
1. 安装 CrossOver (试用版)
2. 创建新容器
3. 安装依赖:
   - winetricks mfc42
   - winetricks vcrun6
   - winetricks vcrun6sp6
4. 安装 RETAS STUDIO
5. 处理 HASP 保护问题
6. 测试各模块功能
```

### 方案 3: VMware Fusion (免费)

**优点**:
- 个人使用免费
- 良好的 Apple Silicon 支持
- 完整 Windows 环境

**缺点**:
- 需要 Windows 许可证
- 配置较复杂
- 性能略低于 Parallels

### 方案 4: Wine + Wineskin (免费DIY)

**优点**:
- 完全免费
- 无需 Windows
- 学习机会

**缺点**:
- 技术难度高
- 需要大量调试
- 兼容性不保证

---

## HASP 保护解决方案

### 选项 A: 虚拟机 + USB 透传
```
物理加密狗 → USB透传 → Windows VM → RETAS STUDIO
```
- 最可靠、合法
- 需要保留加密狗

### 选项 B: 加密狗模拟器
- 需要 dump 加密狗数据
- 使用 Multikey 等模拟器
- 法律灰色地带

### 选项 C: 寻找破解版
- 注意: 需要确保来源可靠
- 法律风险

### 选项 D: 联系 CELSYS
- 请求无加密狗版本
- 成功率较低（软件已停更）

---

## 详细实施计划

### 阶段 1: 环境准备 (1-2天)

#### 任务 1.1: 安装虚拟化方案
- [ ] 下载 Parallels Desktop 试用版
- [ ] 或下载 VMware Fusion (免费)
- [ ] 或下载 CrossOver 试用版

#### 任务 1.2: 准备 Windows 环境
- [ ] 获取 Windows 11 ARM ISO
- [ ] 创建虚拟机
- [ ] 安装基本驱动

### 阶段 2: RETAS 安装测试 (2-3天)

#### 任务 2.1: 基础安装
- [ ] 复制 RETAS 文件到虚拟机
- [ ] 运行安装程序 (如有)
- [ ] 安装 MFC42 运行时

#### 任务 2.2: 许可配置
- [ ] 配置序列号
- [ ] 处理 HASP 保护

#### 任务 2.3: 功能测试
- [ ] 测试 Stylos
- [ ] 测试 PaintMan
- [ ] 测试 TraceMan
- [ ] 测试 CoreRETAS

### 阶段 3: 优化与文档 (1-2天)

#### 任务 3.1: 性能优化
- [ ] 调整虚拟机配置
- [ ] 测试图形性能

#### 任务 3.2: 文档整理
- [ ] 记录遇到的问题
- [ ] 编写使用指南
- [ ] 创建故障排除文档

---

## 推荐方案

**短期 (立即可用)**:
使用 **Parallels Desktop** + Windows 11 ARM
- 最可靠
- 支持加密狗
- 100% 兼容

**中期 (节省成本)**:
测试 **CrossOver** 方案
- 先用试用版验证
- 确认功能完整性
- 考虑购买

**长期 (完全移植)**:
考虑迁移到 **CLIP STUDIO PAINT**
- 原生 macOS 支持
- 官方迁移路径
- 持续更新

---

## 风险与注意事项

1. **法律风险**: 逆向工程需遵守当地法律
2. **数据安全**: 备份原始文件
3. **功能限制**: 可能存在功能缺失
4. **性能问题**: 虚拟化有性能损耗
5. **支持问题**: 无官方技术支持

---

## 相关资源

- [WineHQ](https://www.winehq.org/)
- [CrossOver](https://www.codeweavers.com/crossover)
- [Parallels Desktop](https://www.parallels.com/)
- [VMware Fusion](https://www.vmware.com/products/fusion.html)
- [RETAS STUDIO 系统要求](http://www.retasstudio.net/products/system/)

---

*文档版本: 1.0*
*创建日期: 2026-04-20*
*作者: Sisyphus*

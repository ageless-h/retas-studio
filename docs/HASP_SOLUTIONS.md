# HASP 保护处理方案

## 问题分析

RETAS STUDIO 使用 HASP 加密狗保护，检测到以下组件：

- `HASPUT16.DLL` - 16位 HASP API 库
- `HASPDOSDRV` - DOS 驱动引用
- `FCTProtect` 类 - 保护管理类

## Wine 兼容性问题

Wine **不支持** HASP 硬件加密狗，原因：

1. Wine 运行在用户空间，无法加载内核驱动
2. HASP 需要直接硬件访问
3. 没有开源的 HASP 驱动实现

## 解决方案

### 方案 1: 虚拟机 + USB 透传 (推荐)

**优点**: 合法、可靠
**缺点**: 需要物理加密狗

步骤：
1. 安装 Parallels Desktop 或 VMware Fusion
2. 创建 Windows 虚拟机
3. 配置 USB 设备透传
4. 安装 HASP 驱动
5. 运行 RETAS STUDIO

### 方案 2: 无加密狗版本

**寻找方案**:
- 联系 CELSYS 请求无保护版本
- 搜索 RETAS STUDIO 无加密狗版本

**注意**: 确保来源合法

### 方案 3: 加密狗模拟器

**技术方案**:
1. 使用 h5dmp 或 hasploger 读取加密狗数据
2. 创建注册表模拟文件
3. 使用 Multikey 或类似驱动

**法律警告**: 
- 仅用于个人合法使用的软件
- 不同地区法律不同，请咨询当地法律

### 方案 4: 二进制补丁

**原理**: 修改程序跳过保护检查

**风险**:
- 可能破坏程序功能
- 每个模块需要单独处理
- 需要逆向工程技能

## 具体实施

### 检测保护点

使用调试器查找保护检查代码：

```
1. 加载 Stylos.exe 到 x64dbg 或 OllyDbg
2. 搜索字符串 "HASPUT16" 
3. 查找调用 HASP API 的代码
4. 分析返回值检查逻辑
```

### 可能的补丁点

寻找以下模式：
- `test eax, eax` 后跟条件跳转
- `cmp eax, 0` 后跟 `jnz/jz`
- 调用 `hasp()` 或 `HaspCode()` 后的检查

## 注意事项

1. **备份原始文件** - 任何修改前先备份
2. **分模块处理** - 4个程序可能都需要处理
3. **测试功能** - 补丁后全面测试
4. **法律合规** - 确保合法使用

## 资源

- [HASPRUS/Tiranium](https://github.com/topics/hasp) - GitHub HASP 相关项目
- [Multikey](https://github.com/topics/multikey) - 加密狗模拟器
- [x64dbg](https://x64dbg.com/) - Windows 调试器

---
*本文档仅供教育和个人合法使用参考*

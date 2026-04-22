RETAS STUDIO Wine 配置说明
================================

Wine Prefix 设置
----------------
推荐使用 32 位 prefix（兼容性更好）:

export WINEPREFIX="$HOME/.wine_retas"
export WINEARCH=win32

必需依赖
--------
1. MFC42 运行时:
   winetricks mfc42

2. Visual C++ 6 运行时:
   winetricks vcrun6
   winetricks vcrun6sp6

3. 字体:
   winetricks corefonts
   winetricks fontsmooth-rgb

4. 其他:
   winetricks allfonts

DLL 覆盖设置
------------
运行以下命令设置 DLL 覆盖:

wine regedit <<EOF
Windows Registry Editor Version 5.00

[HKEY_CURRENT_USER\Software\Wine\DllOverrides]
"mfc42"="native"
"mfc42u"="native"
"rcwCmn"="native"
"rcwRes"="native"
"rtwUtl"="native"
EOF

注册表配置
----------
导入 RETAS 注册表:

wine regedit retas_registry.reg

已知问题
--------
1. HASP 加密狗无法在 Wine 下工作
2. 部分面板可能不显示
3. QuickTime 插件需要单独处理

性能优化
--------
1. 设置 Windows 版本为 Windows 7:
   winecfg
   
2. 启用虚拟桌面:
   winecfg -> 显示 -> 虚拟桌面

3. 禁用不必要的服务:
   wine regedit
   [HKEY_CURRENT_USER\Software\Wine\Winetricks]
   "mono"="disabled"

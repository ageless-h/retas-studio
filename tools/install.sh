#!/bin/bash
# RETAS STUDIO macOS 安装脚本
# 用于 CrossOver/Wine 环境

set -e

echo "=========================================="
echo "RETAS STUDIO macOS 安装向导"
echo "=========================================="
echo ""

# 检测运行环境
detect_environment() {
    echo "[1/5] 检测运行环境..."
    
    if command -v wine &> /dev/null; then
        echo "✓ Wine 已安装"
        WINE_CMD="wine"
    elif command -v /Applications/CrossOver.app/Contents/MacOS/CrossOver &> /dev/null; then
        echo "✓ CrossOver 已安装"
        echo "请使用 CrossOver 图形界面创建 Windows 容器"
        exit 0
    else
        echo "✗ 未检测到 Wine 或 CrossOver"
        echo ""
        echo "请先安装以下之一:"
        echo "  1. CrossOver (推荐): https://www.codeweavers.com/crossover"
        echo "  2. Wine: brew install --cask wine-stable"
        echo "  3. Parallels Desktop: https://www.parallels.com/"
        exit 1
    fi
}

# 安装 Wine 依赖
install_wine_deps() {
    echo ""
    echo "[2/5] 安装 Wine 依赖..."
    
    if command -v winetricks &> /dev/null; then
        echo "安装 MFC42..."
        winetricks mfc42 || echo "⚠ MFC42 安装可能失败，请手动处理"
        
        echo "安装 Visual C++ 6 运行时..."
        winetricks vcrun6 || true
        winetricks vcrun6sp6 || true
        
        echo "安装其他依赖..."
        winetricks corefonts || true
        winetricks fontsmooth-rgb || true
    else
        echo "✗ winetricks 未安装"
        echo "请运行: brew install winetricks"
        exit 1
    fi
}

# 复制 RETAS 文件
copy_retas_files() {
    echo ""
    echo "[3/5] 复制 RETAS STUDIO 文件..."
    
    SOURCE_DIR="/Users/huzhiheng/Documents/RETAS.STUDIO.6.6.0/RETAS 6.6.0简中/RETAS STUDIO 6.6.0 CHS"
    
    if [ ! -d "$SOURCE_DIR" ]; then
        echo "✗ 源目录不存在: $SOURCE_DIR"
        exit 1
    fi
    
    WINEPREFIX="${WINEPREFIX:-$HOME/.wine}"
    DEST_DIR="$WINEPREFIX/drive_c/Program Files/RETAS STUDIO"
    
    mkdir -p "$DEST_DIR"
    
    echo "复制文件到: $DEST_DIR"
    cp -r "$SOURCE_DIR"/* "$DEST_DIR/"
    
    echo "✓ 文件复制完成"
}

# 配置注册表
configure_registry() {
    echo ""
    echo "[4/5] 配置注册表..."
    
    # 创建注册表文件
    cat > /tmp/retas_registry.reg << 'EOF'
Windows Registry Editor Version 5.00

[HKEY_LOCAL_MACHINE\SOFTWARE\CELSYS]
[HKEY_LOCAL_MACHINE\SOFTWARE\CELSYS\RETAS STUDIO]
[HKEY_LOCAL_MACHINE\SOFTWARE\CELSYS\RETAS STUDIO\6.0]

[HKEY_CURRENT_USER\SOFTWARE\CELSYS]
[HKEY_CURRENT_USER\SOFTWARE\CELSYS\RETAS STUDIO]
EOF

    if command -v wine &> /dev/null; then
        wine regedit /tmp/retas_registry.reg
        echo "✓ 注册表配置完成"
    fi
    
    rm -f /tmp/retas_registry.reg
}

# 创建快捷方式
create_shortcuts() {
    echo ""
    echo "[5/5] 创建快捷方式..."
    
    WINEPREFIX="${WINEPREFIX:-$HOME/.wine}"
    RETAS_DIR="$WINEPREFIX/drive_c/Program Files/RETAS STUDIO"
    
    # 创建启动脚本
    cat > "$HOME/Desktop/RETAS_Stylos.command" << EOF
#!/bin/bash
cd "$RETAS_DIR/Stylos"
wine Stylos.exe
EOF
    chmod +x "$HOME/Desktop/RETAS_Stylos.command"
    
    cat > "$HOME/Desktop/RETAS_PaintMan.command" << EOF
#!/bin/bash
cd "$RETAS_DIR/PaintMan"
wine PaintMan.exe
EOF
    chmod +x "$HOME/Desktop/RETAS_PaintMan.command"
    
    cat > "$HOME/Desktop/RETAS_TraceMan.command" << EOF
#!/bin/bash
cd "$RETAS_DIR/TraceMan"
wine TraceMan.exe
EOF
    chmod +x "$HOME/Desktop/RETAS_TraceMan.command"
    
    cat > "$HOME/Desktop/RETAS_CoreRETAS.command" << EOF
#!/bin/bash
cd "$RETAS_DIR/CoreRETAS"
wine CoreRETAS.exe
EOF
    chmod +x "$HOME/Desktop/RETAS_CoreRETAS.command"
    
    echo "✓ 快捷方式已创建到桌面"
}

# 显示完成信息
show_completion() {
    echo ""
    echo "=========================================="
    echo "安装完成!"
    echo "=========================================="
    echo ""
    echo "⚠ 重要提示:"
    echo ""
    echo "1. HASP 加密狗问题:"
    echo "   Wine 不支持 HASP 硬件加密狗"
    echo "   如需使用加密狗，请考虑:"
    echo "   - 使用 Parallels Desktop + USB 透传"
    echo "   - 使用加密狗模拟器 (法律风险自负)"
    echo ""
    echo "2. 序列号:"
    echo "   请查看 retas序列号.txt 文件"
    echo ""
    echo "3. 已知问题:"
    echo "   - 部分UI面板可能无法显示"
    echo "   - 插件兼容性需要单独测试"
    echo ""
    echo "4. 测试建议:"
    echo "   先测试 Stylos 模块"
    echo "   逐步验证其他模块功能"
    echo ""
}

# 主程序
main() {
    detect_environment
    install_wine_deps
    copy_retas_files
    configure_registry
    create_shortcuts
    show_completion
}

main "$@"

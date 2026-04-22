#!/bin/bash

set -e

echo "=========================================="
echo "RETAS STUDIO 环境检测工具"
echo "=========================================="
echo ""

check_rosetta() {
    echo "[1/6] 检测 Rosetta 2..."
    if arch -x86_64 /bin/bash -c 'exit 0' 2>/dev/null; then
        echo "✓ Rosetta 2 已安装"
        return 0
    else
        echo "✗ Rosetta 2 未安装"
        echo "  运行: softwareupdate --install-rosetta --agree-to-license"
        return 1
    fi
}

check_arch() {
    echo ""
    echo "[2/6] 检测系统架构..."
    ARCH=$(arch)
    echo "  当前架构: $ARCH"
    if [ "$ARCH" = "arm64" ]; then
        echo "  Apple Silicon 检测到，需要 Rosetta 2 运行 32位 Windows 应用"
    else
        echo "  Intel Mac，可直接运行 Wine"
    fi
}

check_virtualization() {
    echo ""
    echo "[3/6] 检测虚拟化软件..."
    
    FOUND=0
    
    if [ -d "/Applications/Parallels Desktop.app" ]; then
        echo "✓ Parallels Desktop 已安装"
        FOUND=1
    fi
    
    if [ -d "/Applications/VMware Fusion.app" ] || [ -d "/Applications/VMware Fusion Tech Preview.app" ]; then
        echo "✓ VMware Fusion 已安装"
        FOUND=1
    fi
    
    if [ -d "/Applications/CrossOver.app" ]; then
        echo "✓ CrossOver 已安装"
        FOUND=1
    fi
    
    if [ -d "/Applications/Wine Stable.app" ] || [ -d "/Applications/Wine Devel.app" ]; then
        echo "✓ Wine 已安装"
        FOUND=1
    fi
    
    if [ -d "/Applications/UTM.app" ]; then
        echo "✓ UTM 已安装"
        FOUND=1
    fi
    
    if [ $FOUND -eq 0 ]; then
        echo "✗ 未检测到虚拟化软件"
        echo ""
        echo "推荐安装:"
        echo "  1. CrossOver (推荐):  brew install --cask crossover"
        echo "  2. VMware Fusion (免费): 从 VMware 官网下载"
        echo "  3. Wine:              brew install --cask wine-stable"
        echo "  4. UTM (免费):        brew install --cask utm"
    fi
}

check_wine() {
    echo ""
    echo "[4/6] 检测 Wine 环境..."
    
    if command -v wine &> /dev/null; then
        WINE_VER=$(wine --version 2>/dev/null || echo "未知")
        echo "✓ Wine 已安装: $WINE_VER"
        
        if command -v winetricks &> /dev/null; then
            WT_VER=$(winetricks --version 2>/dev/null | head -1 || echo "未知")
            echo "✓ winetricks 已安装: $WT_VER"
        else
            echo "✗ winetricks 未安装"
            echo "  运行: brew install winetricks"
        fi
    else
        echo "✗ Wine 未安装"
        echo "  运行: brew install --cask wine-stable"
    fi
}

check_retas_files() {
    echo ""
    echo "[5/6] 检测 RETAS STUDIO 文件..."
    
    RETAS_DIR="/Users/huzhiheng/Documents/RETAS.STUDIO.6.6.0/RETAS 6.6.0简中/RETAS STUDIO 6.6.0 CHS"
    
    if [ -d "$RETAS_DIR" ]; then
        echo "✓ RETAS STUDIO 目录存在"
        
        for module in Stylos PaintMan TraceMan CoreRETAS; do
            if [ -f "$RETAS_DIR/$module/$module.exe" ]; then
                echo "  ✓ $module.exe 存在"
            else
                echo "  ✗ $module.exe 缺失"
            fi
        done
    else
        echo "✗ RETAS STUDIO 目录不存在"
        echo "  预期位置: $RETAS_DIR"
    fi
}

check_serial() {
    echo ""
    echo "[6/6] 检测序列号..."
    
    SERIAL_FILE="/Users/huzhiheng/Documents/RETAS.STUDIO.6.6.0/retas序列号.txt"
    
    if [ -f "$SERIAL_FILE" ]; then
        echo "✓ 序列号文件存在"
        echo "  位置: $SERIAL_FILE"
    else
        echo "✗ 序列号文件不存在"
    fi
}

show_recommendation() {
    echo ""
    echo "=========================================="
    echo "推荐方案"
    echo "=========================================="
    echo ""
    echo "基于检测结果，推荐以下方案:"
    echo ""
    echo "【方案 A - 最简单】CrossOver"
    echo "  1. 安装: brew install --cask crossover"
    echo "  2. 下载 14 天试用版验证"
    echo "  3. 创建容器，安装 RETAS STUDIO"
    echo ""
    echo "【方案 B - 最可靠】VMware Fusion + Windows"
    echo "  1. 下载 VMware Fusion (个人免费)"
    echo "  2. 创建 Windows 11 ARM 虚拟机"
    echo "  3. 安装 RETAS STUDIO"
    echo ""
    echo "【方案 C - 免费DIY】Wine"
    echo "  1. 安装: brew install --cask wine-stable"
    echo "  2. 安装: brew install winetricks"
    echo "  3. 运行安装脚本"
    echo ""
}

main() {
    check_rosetta
    check_arch
    check_virtualization
    check_wine
    check_retas_files
    check_serial
    show_recommendation
}

main

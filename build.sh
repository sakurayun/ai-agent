#!/bin/bash

# 构建脚本 - 确保使用正确的 Rust nightly 编译器
# 使用方法: ./build.sh [release]

set -e

echo "🔧 AI Agent 构建脚本"
echo "===================="

# 检查 rustup 是否可用
if [ -x "$HOME/.cargo/bin/rustup" ]; then
    echo "✅ 找到 rustup: $HOME/.cargo/bin/rustup"
    CARGO="$HOME/.cargo/bin/cargo"
    RUSTUP="$HOME/.cargo/bin/rustup"
    
    # 显示当前工具链
    echo "📦 当前工具链:"
    $RUSTUP show active-toolchain
    
    # 确保 nightly 工具链已安装
    if ! $RUSTUP toolchain list | grep -q "nightly"; then
        echo "📥 安装 nightly 工具链..."
        $RUSTUP toolchain install nightly
    fi
else
    echo "⚠️  未找到 rustup，使用系统 cargo"
    CARGO="cargo"
fi

# 检查构建模式
if [ "$1" = "release" ]; then
    echo "🚀 构建 Release 版本..."
    $CARGO build --release
    echo "✅ Release 构建完成: target/release/my-gpui-app"
else
    echo "🔨 构建 Debug 版本..."
    $CARGO build
    echo "✅ Debug 构建完成: target/debug/my-gpui-app"
fi

echo ""
echo "📝 运行应用:"
if [ "$1" = "release" ]; then
    echo "   ./target/release/my-gpui-app"
else
    echo "   ./target/debug/my-gpui-app"
fi

echo ""
echo "📝 创建 macOS 应用包:"
echo "   cargo install cargo-bundle  # 首次需要安装"
echo "   $CARGO bundle --release"
echo "   open 'target/release/bundle/osx/AI Agent.app'"

#!/bin/bash

# 一键编译脚本 - 编译所有平台版本的 ai_git_pre_commit
# 支持: macOS Intel, macOS Apple Silicon, Windows x64, Linux x64 musl

set -e  # 遇到错误立即退出

# 配置
PROJECT_NAME="ai_git_pre_commit"
RELEASE_DIR="releases"

# 目标平台配置
targets=(
    "x86_64-apple-darwin"    # macOS Intel
    "aarch64-apple-darwin"   # macOS Apple Silicon
    "x86_64-pc-windows-gnu"  # Windows x64
    "x86_64-unknown-linux-musl" # Linux x64 musl
)

# 输出文件名映射逻辑在循环中处理

echo "🚀 开始编译 ${PROJECT_NAME} 所有平台版本"
echo "============================================"

# 创建 releases 目录
mkdir -p "${RELEASE_DIR}"

# 检查 rustup 是否安装
if ! command -v rustup >/dev/null 2>&1; then
    echo "❌ 错误: rustup 未安装，请先安装 Rust 工具链"
    exit 1
fi

# 添加必要的编译目标
echo "📦 添加必要的 Rust 编译目标..."
for target in "${targets[@]}"; do
    if ! rustup target list | grep -q "${target} (installed)"; then
        echo "添加目标: ${target}"
        rustup target add "${target}"
    else
        echo "目标已安装: ${target}"
    fi
done

echo ""
echo "🔨 开始编译各个平台版本..."
echo "============================================"

# 编译所有目标
for target in "${targets[@]}"; do
    # 根据 target 确定 output_name
    case "${target}" in
        "x86_64-apple-darwin")
            output_name="${PROJECT_NAME}-darwin-amd64"
            ;;
        "aarch64-apple-darwin")
            output_name="${PROJECT_NAME}-darwin-arm64"
            ;;
        "x86_64-pc-windows-gnu")
            output_name="${PROJECT_NAME}-windows-amd64.exe"
            ;;
        "x86_64-unknown-linux-musl")
            output_name="${PROJECT_NAME}-linux-amd64"
            ;;
        *)
            echo "❌ 未知目标: ${target}"
            continue
            ;;
    esac

    output_path="${RELEASE_DIR}/${output_name}"
    
    echo "编译: ${target} -> ${output_name}"
    
    # 编译
    if [[ "${target}" == *"windows"* ]]; then
        # Windows 目标需要特殊处理
        cargo build --release --target "${target}"
        cp "target/${target}/release/${PROJECT_NAME}.exe" "${output_path}"
    else
        # 其他平台
        cargo build --release --target "${target}"
        cp "target/${target}/release/${PROJECT_NAME}" "${output_path}"
    fi
    
    # 设置执行权限（非Windows平台）
    if [[ "${target}" != *"windows"* ]]; then
        chmod +x "${output_path}"
    fi
    
    # 显示文件信息
    file_size=$(du -h "${output_path}" | cut -f1)
    echo "✅ 完成: ${output_name} (${file_size})"
    echo ""
done

echo "🎉 所有平台版本编译完成！"
echo "============================================"
echo "输出文件位于: ${RELEASE_DIR}/"
echo ""

# 显示编译结果
ls -la "${RELEASE_DIR}/"
echo ""
echo "📋 编译结果汇总:"
echo "============================================"
for target in "${targets[@]}"; do
    # 根据 target 确定 output_name
    case "${target}" in
        "x86_64-apple-darwin")
            output_name="${PROJECT_NAME}-darwin-amd64"
            ;;
        "aarch64-apple-darwin")
            output_name="${PROJECT_NAME}-darwin-arm64"
            ;;
        "x86_64-pc-windows-gnu")
            output_name="${PROJECT_NAME}-windows-amd64.exe"
            ;;
        "x86_64-unknown-linux-musl")
            output_name="${PROJECT_NAME}-linux-amd64"
            ;;
    esac

    output_path="${RELEASE_DIR}/${output_name}"
    if [ -f "${output_path}" ]; then
        file_size=$(du -h "${output_path}" | cut -f1)
        echo "✓ ${output_name} (${file_size})"
    else
        echo "✗ ${output_name} (编译失败)"
    fi
done

echo ""
echo "💡 提示: 可以使用以下命令上传到服务器:"
echo "  scp ${RELEASE_DIR}/* user@server:/path/to/releases/"
echo ""
echo "✨ 一键编译完成！"
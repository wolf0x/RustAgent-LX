#!/bin/bash
# 测试 CLI 模式的输出显示

echo "=========================================="
echo "🧪 测试 Instant 模式输出"
echo "=========================================="
echo ""
echo "任务：列出当前目录的文件"
echo ""

timeout 10 ./target/release/RustAgent cli "列出当前目录的文件" 2>&1 | head -50

echo ""
echo "=========================================="
echo "🧪 测试 Expert 模式输出"
echo "=========================================="
echo ""
echo "任务：检查系统信息"
echo ""

timeout 30 ./target/release/RustAgent --mode expert cli "检查系统信息" 2>&1 | head -100

echo ""
echo "=========================================="
echo "✅ 测试完成"
echo "=========================================="

#!/bin/bash
# 简单的授权和推送脚本

set -e  # 遇到错误立即退出

echo "=========================================="
echo "🔐 RustAgent GitHub 授权和推送"
echo "=========================================="
echo ""

# 步骤 1: 登录 GitHub
echo "📋 步骤 1: 登录 GitHub CLI"
echo "----------------------------------------"
echo "即将启动浏览器进行 GitHub 授权..."
echo "请在浏览器中完成授权后回到此终端"
echo ""
read -p "按 Enter 键继续..."

gh auth login --hostname github.com --git-protocol ssh --web

echo ""
echo "✅ GitHub 登录成功！"
echo ""

# 步骤 2: 测试 SSH 连接
echo "📋 步骤 2: 验证 SSH 连接"
echo "----------------------------------------"
if ssh -T git@github.com 2>&1 | grep -q "successfully authenticated"; then
    echo "✅ SSH 连接正常！"
else
    echo "⚠️  SSH 连接需要确认，尝试继续..."
fi
echo ""

# 步骤 3: 推送代码
echo "📋 步骤 3: 推送代码到 GitHub"
echo "----------------------------------------"
echo "正在推送主分支..."
git push -u origin main

echo "正在推送 tag..."
git push origin v1.0.0

echo ""
echo "✅ 代码推送成功！"
echo ""

# 步骤 4: 创建 Release（可选）
echo "📋 步骤 4: 创建 GitHub Release"
echo "----------------------------------------"
read -p "是否创建 GitHub Release? (y/n): " create_release
if [ "$create_release" = "y" ] || [ "$create_release" = "Y" ]; then
    echo "正在创建 Release v1.0.0..."
    gh release create v1.0.0 \
      --title "v1.0.0 - First Stable Release" \
      --notes-file RELEASE_NOTES.md \
      RustAgent-linux-x86_64
    
    echo ""
    echo "✅ Release 创建成功！"
else
    echo "跳过 Release 创建"
fi

echo ""
echo "=========================================="
echo "🎉 完成！"
echo "=========================================="
echo ""
echo "您的项目已发布到："
echo "https://github.com/wolf0x/RustAgent-LX"
echo ""

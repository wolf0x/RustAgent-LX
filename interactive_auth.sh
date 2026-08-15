#!/bin/bash
# 交互式 GitHub 授权脚本

echo "=========================================="
echo "🔐 RustAgent GitHub 交互式授权"
echo "=========================================="
echo ""
echo "此脚本将引导您完成 GitHub 授权和代码推送"
echo ""
read -p "按 Enter 键开始..."

echo ""
echo "📋 步骤 1: 登录 GitHub CLI"
echo "=========================================="
echo ""
echo "即将打开浏览器进行 GitHub 登录..."
echo "请在浏览器中完成授权后，回到此终端"
echo ""

# 启动 gh auth login
gh auth login --hostname github.com --git-protocol ssh --web

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ GitHub 登录成功！"
else
    echo ""
    echo "❌ GitHub 登录失败"
    exit 1
fi

echo ""
echo "📋 步骤 2: 验证 SSH 连接"
echo "=========================================="
ssh -T git@github.com
if [ $? -eq 0 ]; then
    echo ""
    echo "✅ SSH 连接正常！"
else
    echo ""
    echo "⚠️  SSH 连接需要确认，继续..."
fi

echo ""
echo "📋 步骤 3: 推送代码到 GitHub"
echo "=========================================="
read -p "是否推送代码到 GitHub? (y/n): " confirm
if [ "$confirm" = "y" ]; then
    echo ""
    echo "正在推送主分支..."
    git push -u origin main
    
    echo ""
    echo "正在推送 tag v1.0.0..."
    git push origin v1.0.0
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ 代码推送成功！"
    else
        echo ""
        echo "❌ 代码推送失败"
        exit 1
    fi
else
    echo "跳过推送步骤"
fi

echo ""
echo "📋 步骤 4: 创建 GitHub Release"
echo "=========================================="
read -p "是否创建 GitHub Release? (y/n): " create_release
if [ "$create_release" = "y" ]; then
    echo ""
    echo "正在创建 Release v1.0.0..."
    gh release create v1.0.0 \
      --title "v1.0.0 - First Stable Release" \
      --notes-file RELEASE_NOTES.md \
      RustAgent-linux-x86_64
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ Release 创建成功！"
    else
        echo ""
        echo "❌ Release 创建失败"
        exit 1
    fi
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
echo "感谢使用 RustAgent！"

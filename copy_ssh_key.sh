#!/bin/bash
# 快速复制 SSH 公钥到剪贴板

echo "=== RustAgent SSH 公钥 ==="
echo ""
cat ~/.ssh/id_ed25519.pub
echo ""
echo "=========================="
echo ""
echo "请复制上面的公钥，然后访问："
echo "https://github.com/settings/keys"
echo ""
echo "点击 'New SSH key' 并粘贴"
echo ""

# 尝试复制到剪贴板
if command -v xclip &> /dev/null; then
    cat ~/.ssh/id_ed25519.pub | xclip -selection clipboard
    echo "✅ 公钥已复制到剪贴板！"
elif command -v xsel &> /dev/null; then
    cat ~/.ssh/id_ed25519.pub | xsel --clipboard
    echo "✅ 公钥已复制到剪贴板！"
else
    echo "💡 提示：安装 xclip 可以自动复制"
    echo "   sudo apt install xclip"
fi

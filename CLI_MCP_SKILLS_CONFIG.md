# CLI 模式配置完整指南

## 📋 配置文件加载顺序

所有配置文件都在 `~/.RustAgent/workspace/` 目录下，**CLI 和 Web 模式都会加载**：

1. **config.toml** - 主配置（可选）
2. **models.json** - 模型配置（必需）
3. **mcp_servers.json** - MCP 服务器（可选）
4. **cron_tasks.json** - 定时任务（可选，仅 Web 模式执行）
5. **AGENTS.md** - Agent 规则（可选，自动注入）
6. **SOUL.md** - 人格定义（可选，自动注入）
7. **TOOLS.md** - 工具约定（可选，自动注入，已嵌入跨平台版本）
8. **MEMORY.md** - 长期记忆（可选，自动注入）
9. **USER.md** - 用户偏好（可选，自动注入）
10. **HEARTBEAT.md** - 心跳检查清单（可选，仅 Web 模式执行）

## 🔧 MCP 配置

### 配置文件：mcp_servers.json

**位置**：`~/.RustAgent/workspace/mcp_servers.json`

**作用**：配置 MCP（Model Context Protocol）工具服务器

**加载时机**：启动时自动加载（CLI 和 Web 模式都会）

**格式**：
```json
{
  "servers": [
    {
      "name": "filesystem",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
      "env": {}
    },
    {
      "name": "github",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "your-token-here"
      }
    }
  ]
}
```

### CLI 模式下使用

```bash
# 1. 配置 MCP 服务器
cat > ~/.RustAgent/workspace/mcp_servers.json << 'EOF'
{
  "servers": [
    {
      "name": "filesystem",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/home/user/workspace"]
    }
  ]
}
EOF

# 2. 运行 CLI 任务
RustAgentX cli "列出 /home/user/workspace 的文件"

# MCP 工具会自动加载并可用
```

### 验证 MCP 工具加载

```bash
# 查看日志
RustAgentX cli "测试任务" 2>&1 | grep -E "MCP tool|Registered.*MCP"

# 应该看到类似输出：
# MCP tool: fs_read (Read file contents)
# Registered 1 MCP tool(s) total
```

## 🎨 SKILLS 配置

### Skills 目录结构

**位置**：`~/.RustAgent/workspace/skills/`

**结构**：
```
skills/
├── skill-name-1/
│   ├── SKILL.md          # 必需：技能定义
│   └── reference.md      # 可选：参考资料
├── skill-name-2/
│   └── SKILL.md
└── ...
```

### SKILL.md 格式

```markdown
---
name: skill-name
description: 技能描述
triggers:
  - trigger1
  - trigger2
---

# 技能内容

这里是技能的详细指导...
```

### CLI 模式下使用

```bash
# 1. 创建技能目录
mkdir -p ~/.RustAgent/workspace/skills/my-skill

# 2. 创建 SKILL.md
cat > ~/.RustAgent/workspace/skills/my-skill/SKILL.md << 'EOF'
---
name: my-skill
description: 我的自定义技能
triggers:
  - 自定义任务
  - 特殊操作
---

# 我的技能

当用户提到"自定义任务"时，执行以下操作：
1. 步骤一
2. 步骤二
3. 步骤三
EOF

# 3. 运行 CLI 任务
RustAgentX cli "执行自定义任务"

# 技能会自动匹配并应用
```

### 验证技能加载

```bash
# 查看日志
RustAgentX cli "测试任务" 2>&1 | grep -E "Loaded.*skills"

# 应该看到类似输出：
# Loaded 3 skills
```

## 📝 完整配置示例

### 最小配置（仅模型）

```bash
# 只需 models.json
mkdir -p ~/.RustAgent/workspace

cat > ~/.RustAgent/workspace/models.json << 'EOF'
[
  {
    "name": "deepseek-v3",
    "api_base": "https://api.deepseek.com",
    "api_key": "sk-your-key-here",
    "context_window": 128000,
    "max_tokens": 8192
  }
]
EOF

# 运行
RustAgentX cli "检查系统信息"
```

### 标准配置（模型 + MCP）

```bash
# 1. 模型配置
cat > ~/.RustAgent/workspace/models.json << 'EOF'
[
  {
    "name": "deepseek-v3",
    "api_base": "https://api.deepseek.com",
    "api_key": "sk-your-key",
    "context_window": 128000,
    "max_tokens": 8192
  }
]
EOF

# 2. MCP 配置
cat > ~/.RustAgent/workspace/mcp_servers.json << 'EOF'
{
  "servers": [
    {
      "name": "filesystem",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/home/user/workspace"]
    }
  ]
}
EOF

# 3. 运行
RustAgentX cli "列出工作区文件"
```

### 完整配置（模型 + MCP + Skills）

```bash
# 1. 模型配置（同上）

# 2. MCP 配置（同上）

# 3. 创建技能
mkdir -p ~/.RustAgent/workspace/skills/code-review

cat > ~/.RustAgent/workspace/skills/code-review/SKILL.md << 'EOF'
---
name: code-review
description: 代码审查技能
triggers:
  - 审查代码
  - code review
---

# 代码审查

当用户要求审查代码时：
1. 读取代码文件
2. 检查常见问题
3. 提供改进建议
EOF

# 4. 运行
RustAgentX cli "审查这段代码"
```

## 🔍 配置验证

### 检查所有配置是否加载

```bash
# 运行并查看详细日志
RustAgentX cli "测试任务" 2>&1 | grep -E "Loaded|Registered|Starting"

# 应该看到类似输出：
# Starting RustAgentX (pid 12345)
# Mode: Headless
# Loaded 1 model config from models.json
# Loaded 2 MCP server(s)
# MCP tool: fs_read (Read file contents)
# Registered 2 MCP tool(s) total
# Loaded 3 skills
```

### 检查特定配置

```bash
# 检查模型
cat ~/.RustAgent/workspace/models.json

# 检查 MCP
cat ~/.RustAgent/workspace/mcp_servers.json

# 检查技能
ls ~/.RustAgent/workspace/skills/
```

## ⚠️ 常见问题

### Q1: MCP 工具没有加载

**检查**：
```bash
# 1. 检查配置文件是否存在
ls ~/.RustAgent/workspace/mcp_servers.json

# 2. 检查 JSON 格式是否正确
cat ~/.RustAgent/workspace/mcp_servers.json | jq .

# 3. 检查日志中的错误
RustAgentX cli "测试" 2>&1 | grep -i "mcp.*error"
```

**解决**：
- 确保 JSON 格式正确
- 确保 MCP 服务器命令可用（如 `npx`）
- 检查网络连接

### Q2: 技能没有匹配

**检查**：
```bash
# 1. 检查技能目录结构
ls ~/.RustAgent/workspace/skills/

# 2. 检查 SKILL.md 格式
cat ~/.RustAgent/workspace/skills/my-skill/SKILL.md

# 3. 检查 triggers 是否正确定义
grep "triggers:" ~/.RustAgent/workspace/skills/*/SKILL.md
```

**解决**：
- 确保 SKILL.md 有正确的 YAML frontmatter
- 确保 triggers 列表包含相关关键词
- 检查任务描述是否匹配 triggers

### Q3: CLI 模式需要所有配置吗？

**不需要**！

- **必需**：`models.json`（至少一个模型）
- **可选**：其他所有配置文件

最小配置只需 `models.json` 即可运行。

## 🚀 快速开始

### 一键配置脚本

```bash
#!/bin/bash
# 创建完整配置

WORKSPACE=~/.RustAgent/workspace
mkdir -p $WORKSPACE/skills

# 1. 模型配置
cat > $WORKSPACE/models.json << 'EOF'
[{"name":"deepseek-v3","api_base":"https://api.deepseek.com","api_key":"sk-your-key","context_window":128000,"max_tokens":8192}]
EOF

# 2. MCP 配置（可选）
cat > $WORKSPACE/mcp_servers.json << 'EOF'
{"servers":[{"name":"filesystem","command":"npx","args":["-y","@modelcontextprotocol/server-filesystem","/tmp"]}]}
EOF

# 3. 示例技能
mkdir -p $WORKSPACE/skills/example
cat > $WORKSPACE/skills/example/SKILL.md << 'EOF'
---
name: example
description: 示例技能
triggers:
  - 示例任务
---

# 示例技能

这是一个示例...
EOF

echo "✅ 配置完成！"
echo "运行：RustAgentX cli \"你的任务\""
```

---

**配置完成！现在可以在 CLI 模式下使用 MCP 和 Skills 了！** 🎉

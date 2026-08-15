# RustAgentX 配置指南

## 📁 配置文件概览

RustAgentX 使用以下配置文件，全部位于 `~/.RustAgent/workspace/` 目录：

| 文件名 | 用途 | 是否必需 | 说明 |
|--------|------|----------|------|
| `config.toml` | 主配置文件 | ✅ | 服务器、Agent、模型配置 |
| `models.json` | 模型配置 | ✅ | LLM 模型定义和 API 密钥 |
| `mcp_servers.json` | MCP 服务器配置 | ❌ | MCP 工具服务器连接 |
| `cron_tasks.json` | 定时任务 | ❌ | CRON 任务定义 |
| `AGENTS.md` | Agent 行为规则 | ❌ | 自动注入到系统提示词 |
| `SOUL.md` | 人格与边界 | ❌ | 自动注入到系统提示词 |
| `TOOLS.md` | 工具使用约定 | ❌ | 自动注入到系统提示词 |
| `MEMORY.md` | 长期记忆 | ❌ | 自动注入到系统提示词 |
| `USER.md` | 用户偏好 | ❌ | 自动注入到系统提示词 |

## 🎯 CLI 模式配置

### 最小配置（快速开始）

对于 CLI 模式，只需配置 `models.json`：

```bash
# 1. 创建配置文件
mkdir -p ~/.RustAgent/workspace

# 2. 配置模型（替换 your-api-key-here）
cat > ~/.RustAgent/workspace/models.json << 'EOFMODELS'
[
  {
    "name": "deepseek-v3",
    "api_base": "https://api.deepseek.com",
    "api_key": "your-api-key-here",
    "context_window": 128000,
    "max_tokens": 8192
  }
]
EOFMODELS

# 3. 运行 CLI 任务
RustAgentX cli "检查系统信息"
```

### 完整配置（推荐）

**config.toml**：
```toml
[server]
host = "127.0.0.1"
port = 7788

[agent]
working_dir = "."
max_iterations = 100
mode = "instant"
timezone_offset = 8
```

**models.json**：同上

## 🔧 配置文件详解

### 1. config.toml

主配置文件，控制服务器和 Agent 行为。

### 2. models.json

定义 LLM 模型和 API 配置。API Key 会自动加密存储。

### 3. mcp_servers.json

配置 MCP 工具服务器。CLI 模式下会自动加载。

### 4. cron_tasks.json

定义定时任务。Web 模式下自动执行，CLI 模式下仅加载不执行。

### 5. 工作区 Markdown 文件

AGENTS.md、SOUL.md、TOOLS.md、MEMORY.md、USER.md 会自动注入到系统提示词。

## 🚀 快速开始

```bash
# 1. 创建目录
mkdir -p ~/.RustAgent/workspace

# 2. 配置模型
cat > ~/.RustAgent/workspace/models.json << 'EOF'
[{"name":"deepseek-v3","api_base":"https://api.deepseek.com","api_key":"your-key","context_window":128000,"max_tokens":8192}]
EOF

# 3. 运行
RustAgentX cli "你的任务"
```

---

**配置完成！现在可以开始使用 RustAgentX 了！** 🚀

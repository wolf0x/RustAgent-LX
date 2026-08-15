# PowerShell 调用问题排查与修复

## 🐛 问题描述

**现象**：在 Linux 环境下运行 RustAgent CLI 模式执行磁盘检测时，agent 第一时间调用了 PowerShell 而不是 bash/sh。

## 🔍 问题根源

### 根本原因：TOOLS.md 包含 Windows 专用规则

**TOOLS.md 的作用**：
- TOOLS.md 是工作区配置文件，定义了本地工具使用约定
- **会被自动注入到系统提示词中**（见 `src/agent/llm_agent.rs:559`）
- Agent 会根据 TOOLS.md 的指示选择命令执行方式

**问题代码位置**：
```rust
// src/agent/llm_agent.rs:556-562
let config_files = [
    ("AGENTS.md", "Agent Behavior & Rules"),
    ("SOUL.md", "Personality, Tone & Boundaries"),
    ("TOOLS.md", "Local Tool Usage Conventions"),  // ← 这里！
    ("MEMORY.md", "Curated Long-Term Memory"),
    ("USER.md", "User Communication Preferences"),
];
```

**原始 TOOLS.md 内容**（Windows 专用）：
```markdown
## Windows 执行规则

### 核心规则

1. **所有可执行文件调用必须显式添加 `.exe` 扩展名**
   ✅ 正确：`python.exe script.py`、`git.exe status`
   
2. **PowerShell 脚本调用统一使用 `powershell.exe -File <脚本路径>`**
   或使用 `pwsh.exe`（若安装 PowerShell Core）
   
3. **CMD 内部命令（如 `dir`、`copy`、`del`）无需后缀**
```

**影响**：
- Agent 在 Linux 下也遵循了 TOOLS.md 的 Windows 规则
- 尝试调用 PowerShell 而不是 bash
- 使用 Windows 路径格式而不是 Linux 路径

## ✅ 解决方案

### 1. 更新工作区 TOOLS.md

**位置**：`~/.RustAgent/workspace/TOOLS.md`

**修改内容**：
- ✅ 移除 Windows 专用规则
- ✅ 添加跨平台执行规则
- ✅ 包含 Linux 和 Windows 路径示例
- ✅ 添加平台检测指导

**新的 TOOLS.md**：
```markdown
TOOLS.md — 本地环境配置（跨平台）

## 通用执行规则

### 核心规则

1. **优先使用跨平台命令**
   - ✅ 正确：`python script.py`、`git status`、`ls -la`
   - ❌ 避免：平台特定命令（如 `dir`、`copy`、Windows 路径）

2. **使用完整路径（如需要）**
   - Linux: `/usr/bin/python3`、`/usr/local/bin/node`
   - Windows: `C:\Python3\python.exe`（仅在 Windows 环境）

3. **脚本调用**
   - Shell 脚本: `bash script.sh` 或 `./script.sh`
   - Python 脚本: `python3 script.py` 或 `python script.py`

## 平台检测

Agent 应该根据运行环境选择合适的命令：

- **Linux**: 使用 `bash`、`sh`、`ls`、`cat`、`grep` 等
- **Windows**: 使用 `powershell`、`cmd`、`dir`、`type` 等
- **跨平台**: 优先使用 Python 脚本或跨平台工具
```

### 2. 更新项目模板 TOOLS.md

**位置**：`AI_IT_AGENT/TOOLS.md`

**作用**：确保首次运行时生成的工作区包含正确的跨平台配置

**修改**：与工作区 TOOLS.md 保持一致

### 3. 备份原始文件

```bash
# 工作区
mv ~/.RustAgent/workspace/TOOLS.md ~/.RustAgent/workspace/TOOLS.md.windows.bak

# 项目
mv TOOLS.md TOOLS.md.windows.bak
```

## 📊 修复效果

### 修复前
```bash
$ RustAgent cli "检查磁盘空间"

# Agent 尝试调用（错误）：
powershell.exe -Command "Get-WmiObject Win32_LogicalDisk | Select-Object DeviceID, Size, FreeSpace"
# 或
powershell -c "Get-PSDrive -PSProvider FileSystem"
```

### 修复后
```bash
$ RustAgent cli "检查磁盘空间"

# Agent 正确调用：
🔧 Calling tool: shell_exec
   Args: {
     "command": "df -h"
   }

✅ Tool result: shell_exec
{
  "exit_code": 0,
  "stdout": "Filesystem      Size  Used Avail Use% Mounted on\n..."
}
```

## 🔧 验证方法

### 1. 检查 TOOLS.md 内容

```bash
cat ~/.RustAgent/workspace/TOOLS.md | head -20
```

**应该看到**：
- ✅ "跨平台" 或 "cross-platform"
- ✅ "Linux 环境" 和 "Windows 环境" 分开说明
- ✅ 没有强制要求 `.exe` 扩展名
- ✅ 没有强制使用 PowerShell

### 2. 测试 CLI 执行

```bash
RustAgent cli "检查磁盘空间" 2>&1 | grep -E "Calling tool|command"
```

**应该看到**：
```
🔧 Calling tool: shell_exec
   Args: {
     "command": "df -h"
   }
```

**不应该看到**：
```
❌ powershell.exe -Command ...
❌ Get-WmiObject ...
```

### 3. 检查系统提示词

在日志中查看注入的系统提示词：

```bash
grep -A 20 "TOOLS.md" ~/.RustAgent/workspace/logs/rustagent-*.log | head -30
```

## 📝 相关文件

| 文件 | 位置 | 作用 |
|------|------|------|
| 工作区 TOOLS.md | `~/.RustAgent/workspace/TOOLS.md` | 运行时注入到系统提示词 |
| 项目 TOOLS.md | `AI_IT_AGENT/TOOLS.md` | 工作区模板，首次运行时复制 |
| 注入逻辑 | `src/agent/llm_agent.rs:556-582` | 读取并注入 TOOLS.md |

## 🎯 最佳实践

### 对于用户

1. **定期检查 TOOLS.md**
   - 确保符合当前运行平台
   - 移除平台特定的硬编码规则

2. **使用跨平台命令**
   - 优先使用 Python 脚本
   - 使用 bash/sh 而不是 PowerShell
   - 使用正斜杠路径而不是反斜杠

3. **平台检测**
   - 在脚本中添加平台检查
   - 根据 `uname -s` 选择不同命令

### 对于开发者

1. **避免硬编码平台规则**
   - 不要在 TOOLS.md 中强制特定平台命令
   - 提供平台选择的指导而不是限制

2. **代码审查**
   - 检查系统提示词注入的内容
   - 确保跨平台兼容性

3. **文档化**
   - 清晰说明 TOOLS.md 的作用
   - 提供跨平台配置示例

## 🚀 立即修复

### 快速修复命令

```bash
# 1. 备份原始文件
mv ~/.RustAgent/workspace/TOOLS.md ~/.RustAgent/workspace/TOOLS.md.windows.bak

# 2. 创建跨平台版本
cat > ~/.RustAgent/workspace/TOOLS.md << 'EOF'
TOOLS.md — 本地环境配置（跨平台）

## 通用执行规则

### 核心规则

1. **优先使用跨平台命令**
   - ✅ 正确：`python script.py`、`git status`、`ls -la`
   - ❌ 避免：平台特定命令（如 `dir`、`copy`、Windows 路径）

2. **使用完整路径（如需要）**
   - Linux: `/usr/bin/python3`、`/usr/local/bin/node`
   - Windows: `C:\Python3\python.exe`（仅在 Windows 环境）

3. **脚本调用**
   - Shell 脚本: `bash script.sh` 或 `./script.sh`
   - Python 脚本: `python3 script.py` 或 `python script.py`

## 平台检测

Agent 应该根据运行环境选择合适的命令：

- **Linux**: 使用 `bash`、`sh`、`ls`、`cat`、`grep` 等
- **Windows**: 使用 `powershell`、`cmd`、`dir`、`type` 等
- **跨平台**: 优先使用 Python 脚本或跨平台工具

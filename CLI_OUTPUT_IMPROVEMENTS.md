# CLI 模式输出改进

## 🐛 修复的问题

### 问题 1: Instant 模式缺少思考过程输出

**之前的行为**：
- ❌ 只显示最终的 `TextDelta`（agent 回答）
- ❌ 没有显示 `Thinking` 事件（思考过程）
- ❌ 没有显示 `ToolCall` 事件（工具调用）
- ❌ 没有显示 `ToolResult` 事件（工具结果）

**现在的改进**：
✅ 显示思考过程（💭 Thinking...）
✅ 显示工具调用（🔧 Calling tool: xxx）
✅ 显示工具参数（Args: {...}）
✅ 显示工具结果（✅ Tool result: xxx）
✅ 显示执行进度（⏳ [10s] tool_name - message）
✅ 显示错误信息（❌ [ERROR] message）

### 问题 2: Expert 模式的 Manager/Executor/Auditor 输出

**之前的行为**：
- ❌ Manager 的计划输出没有显示在终端
- ❌ Executor 的执行过程没有显示
- ❌ Auditor 的审计结果没有显示

**现在的改进**：
✅ 所有 `AgentEvent::text` 事件都会正确显示
✅ Manager 的计划、状态更新可见
✅ Executor 的工具调用和结果可见
✅ Auditor 的审计反馈可见

## 📊 输出示例

### Instant 模式示例

```bash
$ RustAgent cli "列出当前目录的文件"

💭 Thinking...
用户想要查看当前目录的文件列表。我需要使用 shell_exec 工具来执行 ls 命令。

🔧 Calling tool: shell_exec
   Args: {
     "command": "ls -la"
   }

✅ Tool result: shell_exec
{
  "exit_code": 0,
  "stdout": "total 1234\ndrwxr-xr-x  5 user user   4096 Aug 15 16:00 .\n...",
  "stderr": ""
}

当前目录的文件列表如下：
[agent 的最终回答]
```

### Expert 模式示例

```bash
$ RustAgent --mode expert cli "检查系统信息"

💭 Thinking...
这是一个系统检查任务，需要收集多项系统信息。

🔧 Calling tool: shell_exec
   Args: {
     "command": "uname -a"
   }

✅ Tool result: shell_exec
{
  "exit_code": 0,
  "stdout": "Linux hostname 5.15.0-generic #1 SMP x86_64 GNU/Linux",
  "stderr": ""
}

[Manager] 已收集系统内核信息，继续检查其他系统信息...

🔧 Calling tool: shell_exec
   Args: {
     "command": "df -h"
   }

✅ Tool result: shell_exec
{
  "exit_code": 0,
  "stdout": "Filesystem      Size  Used Avail Use% Mounted on\n...",
  "stderr": ""
}

[Executor] 磁盘空间检查完成

[Auditor] 已验证所有系统信息收集任务
```

## 🎨 输出格式说明

| 图标 | 事件类型 | 说明 |
|------|---------|------|
| 💭 | Thinking | Agent 的思考过程 |
| 🔧 | ToolCall | 工具调用 |
| ✅ | ToolResult | 工具执行结果 |
| ⏳ | Progress | 长时间运行工具的进度 |
| ❌ | Error | 错误信息 |
| 📝 | TextDelta | Agent 的最终回答（stdout） |

## 🔧 技术实现

### 修改的文件

**`src/main.rs`** - Headless 模式事件处理

```rust
match &event {
    agent::AgentEvent::Thinking { content, .. } => {
        eprintln!("\n💭 Thinking...\n{}", content);
    }
    agent::AgentEvent::TextDelta { content, .. } => {
        print!("{}", content);
        // flush stdout
    }
    agent::AgentEvent::ToolCall { name, args, .. } => {
        eprintln!("\n🔧 Calling tool: {}", name);
        if !args.is_null() && !args.as_object().map(|m| m.is_empty()).unwrap_or(true) {
            eprintln!("   Args: {}", serde_json::to_string_pretty(args).unwrap_or_default());
        }
    }
    agent::AgentEvent::ToolResult { name, result, .. } => {
        let result_str = serde_json::to_string_pretty(result).unwrap_or_default();
        let display_str = if result_str.len() > 500 {
            format!("{}... (truncated, {} chars total)", &result_str[..500], result_str.len())
        } else {
            result_str
        };
        eprintln!("\n✅ Tool result: {}\n{}", name, display_str);
    }
    agent::AgentEvent::Progress { tool_name, message, elapsed_secs, .. } => {
        eprintln!("⏳ [{}s] {} - {}", elapsed_secs, tool_name, message);
    }
    agent::AgentEvent::Error { message, .. } => {
        eprintln!("\n❌ [ERROR] {}", message);
        exit_code = 1;
    }
    _ => {}
}
```

### 输出流分离

- **stdout**: 仅输出 `TextDelta`（agent 的最终回答）
- **stderr**: 输出所有其他事件（thinking、tool calls、results、errors）

这样设计的好处：
1. 可以通过管道将 agent 回答保存到文件，同时保留所有调试信息
2. 符合 Unix 哲学：数据流走 stdout，状态信息走 stderr
3. 便于自动化脚本解析 agent 输出

## 📝 使用建议

### 查看完整执行过程

```bash
# 查看所有输出（包括 stderr）
RustAgent cli "your task" 2>&1

# 或显式重定向
RustAgent cli "your task" 2>&1 | less
```

### 仅保存 agent 回答

```bash
# 只保存 stdout（agent 回答）到文件
RustAgent cli "your task" > answer.txt

# stderr（执行过程）仍显示在终端
```

### 完全静默执行

```bash
# 只保存 agent 回答，丢弃所有其他输出
RustAgent cli "your task" > answer.txt 2>/dev/null
```

### 记录完整日志

```bash
# 保存所有输出到文件
RustAgent cli "your task" > full_log.txt 2>&1

# 或同时查看和保存
RustAgent cli "your task" 2>&1 | tee full_log.txt
```

## 🧪 测试脚本

运行测试脚本验证输出：

```bash
./test_cli_output.sh
```

该脚本会测试：
1. Instant 模式的完整输出
2. Expert 模式的完整输出
3. 各种事件类型的显示

## 🎯 总结

现在 CLI 模式提供：
- ✅ 完整的执行过程可见性
- ✅ 思考过程透明
- ✅ 工具调用和结果清晰展示
- ✅ Expert 模式的所有角色输出可见
- ✅ 符合 Unix 哲学的输出流设计
- ✅ 便于自动化和脚本集成

**所有模式都提供完整的执行过程信息，只是输出目标不同！**

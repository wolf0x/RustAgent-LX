**[English](README.en.md)** | **[中文](#)**

# RustAgent

跨平台通用 AI 智能体——支持 Web Dashboard 和 CLI 两种运行模式，具备 WebSocket 网关、多模型支持、31+ 内置工具、权限管控、持久记忆与任务调度能力。单二进制部署，开箱即用。

## 🎯 项目定位

RustAgent 是一个**跨平台通用 AI 智能体**，从最初的 Windows IR（事件响应）系统演进而来，现在支持 Linux 和 Windows 双平台。

**两种运行模式**：
- **Web Dashboard 模式**：通过浏览器访问的交互式 Dashboard，支持实时对话、工具调用可视化、设置管理等
- **CLI Headless 模式**：命令行自动化执行，适合脚本集成、定时任务、LongHorizon-Harness 等外部框架调用

**核心能力**：
- 🔧 **31+ 内置工具**：文件操作、Shell 执行、浏览器自动化、恶意软件分析、日志分析、SSH 远程执行等
- 🧠 **智能记忆系统**：SQLite + FTS5 全文搜索 + 知识蒸馏，积累调查经验
- 🔒 **安全权限系统**：分类门控 + 意图策略引擎（Block/Audit/Pass）
- 🌐 **多模型支持**：OpenAI-compatible API，支持 DeepSeek、GPT-4、Qwen 等
- ⏰ **任务调度**：内置 CRON 调度器，支持定期巡检和自动化监控
- 📊 **Profile 系统**：共享/隔离 workspace，多环境管理

## 🚀 快速开始

### 安装

```bash
# 从源码编译
git clone https://github.com/wolf0x/RustAgent-LX.git
cd RustAgent-LX
cargo build --release

# 二进制产物
./target/release/RustAgent  # Linux
.\target\release\RustAgent.exe  # Windows
```

### 运行模式

#### Web Dashboard 模式

```bash
# 启动 Web Dashboard（默认端口 7788）
RustAgent web

# 指定端口
RustAgent web --port 8080

# 使用特定 profile
RustAgent --profile web

# 访问 Dashboard
# 浏览器打开 http://localhost:7788
# 使用启动时显示的密码登录
```

#### CLI Headless 模式

```bash
# 执行任务并退出
RustAgent cli "检查磁盘空间并报告使用情况"

# 使用特定 profile
RustAgent --profile headless "分析系统日志"

# 从文件读取任务（适合自动化）
RustAgent --prompt-file task.md

# 执行模式切换
RustAgent --mode instant cli "快速任务"  # 默认，单轮快速
RustAgent --mode expert cli "复杂分析"   # 多轮深度分析

# 权限控制
RustAgent --auto-approve cli "自动批准所有权限"
RustAgent --read-only cli "只读模式"
```

### Profile 系统

```bash
# 默认：所有 profiles 共享 workspace
RustAgent web                    # 使用 ~/.RustAgent/workspace/
RustAgent cli "task"             # 使用 ~/.RustAgent/workspace/

# 隔离模式：每个 profile 独立 workspace
RustAgent --profile myproject --isolated cli "task"
# 使用 ~/.RustAgent/workspace/profiles/myproject/
```

## 🏗️ 核心架构

```
┌─────────────────────────────────────────────────┐
│          Web Dashboard (SPA)                     │
│    (Chat / Skills / MCP / CRON / Settings)      │
└──────────────────────┬──────────────────────────┘
                       │ WebSocket / HTTP
┌──────────────────────┴──────────────────────────┐
│              Axum 0.8 Server                     │
│         (REST API + WS Gateway + SSE)           │
├─────────────────────────────────────────────────┤
│  Runner → LlmAgent (Agentic Loop)               │
│    ├── Agent trait → EventStream (9 种事件)      │
│    ├── CJK-aware token budget 历史裁剪           │
│    ├── Thinking / ToolCall / ToolResult 输出     │
│    └── Truncated JSON repair                    │
├─────────────────────────────────────────────────┤
│  CLI Mode (Headless)                            │
│    ├── stdin/stdout 交互                        │
│    ├── 完整执行过程可见（stderr）                │
│    └── Agent 回答输出（stdout）                  │
├─────────────────────────────────────────────────┤
│  Tool Layer                                      │
│    ├── 31+ Built-in Tools                        │
│    ├── MCP Client (stdio + SSE)                 │
│    ├── Skill Manager (weighted scoring)          │
│    └── External Tools (workspace/tools/)         │
├─────────────────────────────────────────────────┤
│  Infrastructure                                  │
│    ├── Memory (SQLite + FTS5)                    │
│    ├── Permission (category gates + bypass)      │
│    ├── Intent Policy (Block / Audit / Pass)      │
│    ├── Scheduler (CRON + interval)               │
│    ├── Checkpoint (crash recovery)               │
│    ├── Crypto (AES-256-GCM)                      │
│    └── Knowledge Distillation                    │
└─────────────────────────────────────────────────┘
```

## 🛠️ 工具系统

### 文件操作（5 个）
- **FileRead** / **FileWrite** / **FileDelete** / **FileModify** / **FileList**
- 支持大文件分块读取（>300MB 只读前 1MB）
- 自动权限检查

### Shell 执行
- **ShellExecTool**：bash/sh（Linux）或 PowerShell/CMD（Windows）
- **意图策略引擎**：语义级命令分析
  - **Block**：`rm -rf /`、`dd of=/dev/sda`、`mkfs` 等不可逆操作
  - **Audit**：`rm`、`kill`、`systemctl stop` 等高危操作（记录日志）
  - **Pass**：`ps`、`ls`、`netstat` 等只读命令

### 浏览器自动化（2 个）
- **browser_cdp**：chromiumoxide CDP 隔离浏览器（无登录态，安全搜索）
- **browser_skill**：BSK 扩展控制用户浏览器（需要登录态的操作）

### 恶意软件分析（3 个）
- **malware_scan**：Boreal YARA 规则扫描（支持自定义规则集）
- **malware_deep**：PE 深度分析（goblin 解析 + iced-x86 反汇编）
- **malware_analysis**：综合恶意软件分析

### 日志分析（3 个）
- **ir_log_parse**：通用日志解析（自动识别格式，安全模式匹配）
- **ir_pcap_analyze**：PCAP 流量分析（协议分布、流跟踪、DNS/HTTP 提取）
- **ir_weblog_scan**：Web 日志扫描（SQLi/XSS/RCE 检测）

### Linux 应急响应（13 个）
| 工具 | 能力 |
|------|------|
| linux_ir_process | 进程枚举、资源占用、命令行分析 |
| linux_ir_network | TCP/UDP 连接、监听端口、关联进程 |
| linux_ir_persistence | 持久化位置扫描（cron、systemd、SSH） |
| linux_ir_rootkit | Rootkit 检测（常见特征匹配） |
| linux_ir_file | 文件系统异常检测 |
| linux_ir_web | Web 应用日志分析 |
| linux_ir_mining | 挖矿程序检测 |
| linux_ir_lateral | 横向移动痕迹检测 |
| linux_ir_auth | 认证日志分析 |
| linux_ir_backdoor | 后门检测 |
| linux_ir_bruteforce | 暴力破解检测 |
| linux_ir_integrity | 文件完整性检查 |
| linux_ir_config | 系统配置审计 |

### SSH 远程执行（2 个）
- **linux_ssh**：SSH 远程命令执行（russh 实现）
- **ir_linux**：Linux 远程应急响应

### Web 工具（2 个）
- **web_fetch**：HTTP/HTTPS 页面抓取
- **browser_open**：打开默认浏览器（xdg-open/open）

### 生产力工具（4 个）
- **TodoWrite**：任务规划与进度跟踪
- **CronManage**：CRON 任务管理
- **MemoryMd**：MEMORY.md 读写（长期记忆）
- **SysRemind**：系统提醒（通知到 Web Dashboard）

### MCP 动态工具
- 通过 MCP 协议接入外部工具服务器
- 运行时动态注册到 ToolRegistry
- 支持 stdio + SSE 双传输协议

### 外部工具发现
- `workspace/tools/` 目录下的可执行文件自动发现并注册
- 支持 bash、python3、perl 等脚本

## 🔒 安全特性

### 权限系统
- **五级权限分类**：read / write / delete / modify / execute
- **异步用户授权**：Web Dashboard 通过 WebSocket 推送授权请求
- **跨类别绕过检测**：防止通过 shell_exec 绕过 file_delete 权限

### 命令意图策略
- **语义解析**：提取 verb + targets，替代传统字符串黑名单
- **三级判定**：
  - **Block**：绝对禁止（磁盘格式化、安全日志清除等）
  - **Audit**：审计放行（文件删除、进程终止等，记录日志）
  - **Pass**：静默放行（只读查询等）
- **Linux 专用规则**：
  - Block：`rm -rf /`、`dd of=/dev/sda`、`mkfs`、fork 炸弹等
  - Audit：`rm`、`kill`、`systemctl stop`、挂载操作等
  - Pass：`ps`、`ls`、`netstat`、`cat` 等

### 其他安全特性
- **API 密钥加密**：AES-256-GCM，密钥从 machine-id 派生
- **密码认证 Dashboard**：`.password` 文件保护
- **CDP 浏览器隔离**：chromiumoxide 独立实例，无用户登录态
- **权限拒绝强反馈**：拒绝时向 LLM 返回强措辞错误消息

## 🧠 记忆系统

### SQLite + FTS5 对话记忆
- 4 层 Schema 版本演进
- CJK bigram 分词（中日韩文优化）
- BM25 排序的全文搜索
- 对话历史自动清理（3 天/50 条）

### 知识蒸馏
- 会话结束时自动触发
- LLM 提取结构化知识条目
- 写入 `workspace/knowledge/` 下的 5 个分类文件
- Append-only 设计，只增不改

### 文件记忆（MEMORY.md）
- LLM 主动维护的个人笔记
- 自动注入 System Prompt
- 与 SQLite 记忆互补

## ⏰ 调度系统

- **CRON 表达式**：标准 5 字段（分 时 日 月 周）
- **间隔语法**：`every 5m`、`every 2h` 等自然语言风格
- **JSON 持久化**：任务定义存储于 `cron_tasks.json`
- **30 秒轮询**：调度器每 30 秒检查到期任务
- **心跳机制**：从 `HEARTBEAT.md` 读取周期性健康检查清单

## 📊 CLI 输出

### 输出流分离
- **stdout**：仅输出 agent 的最终回答（`TextDelta`）
- **stderr**：输出所有执行过程（thinking、tool calls、results、errors）

### 事件类型
| 图标 | 事件 | 说明 |
|------|------|------|
| 💭 | Thinking | Agent 的思考过程 |
| 🔧 | ToolCall | 工具调用和参数 |
| ✅ | ToolResult | 工具执行结果 |
| ⏳ | Progress | 长时间运行工具的进度 |
| ❌ | Error | 错误信息 |
| 📝 | TextDelta | Agent 的最终回答 |

### 使用示例

```bash
# 查看完整执行过程
RustAgent cli "检查磁盘空间" 2>&1

# 只保存 agent 回答
RustAgent cli "检查磁盘空间" > answer.txt

# 保存完整日志
RustAgent --mode expert cli "分析系统" > full_log.txt 2>&1
```

## 🔗 LongHorizon-Harness 集成

RustAgent 支持作为 LongHorizon-Harness 的 agent 后端：

```bash
# Harness 调用命令模板
RustAgent --profile headless \
  --prompt-file {prompt_path} \
  --timeout {seconds} \
  --auto-approve
```

- `--prompt-file`：从文件读取任务
- `--timeout`：执行超时控制
- `--auto-approve`：自动批准所有权限
- `--trajectory`：JSONL 轨迹输出

Python adapter 见：`adapters/rustagent_adapter.py`

## 📁 配置

### 工作目录

**Linux**：`~/.RustAgent/workspace/`  
**Windows**：`%USERPROFILE%\.RustAgent\workspace\`

### 目录结构

```
workspace/
├── config.toml          # 主配置
├── models.json          # 模型配置（API Key 加密存储）
├── mcp_servers.json     # MCP 服务器配置
├── cron_tasks.json      # 定时任务定义
├── .password            # Dashboard 访问密码（仅 Web 模式）
├── memory/
│   └── memory.db        # SQLite 记忆数据库
├── knowledge/           # 知识蒸馏输出
├── skills/              # 技能目录
├── tools/               # 外部工具目录
├── logs/                # 运行日志（rustagent-YYYY-MM-DD.N.log）
├── static/              # Dashboard 静态资源
└── output/              # 工具输出
```

### 配置文件示例

**config.toml**：
```toml
[server]
host = "127.0.0.1"
port = 7788

[agent]
working_dir = "."
max_iterations = 100
mode = "instant"  # 或 "expert"
timezone_offset = 8
```

**models.json**：
```json
[
  {
    "name": "deepseek-v4-flash",
    "api_base": "https://api.deepseek.com",
    "api_key": "your-key-here",
    "context_window": 128000,
    "max_tokens": 16384
  }
]
```

## 🛠️ 技术栈

| 组件 | 技术选型 |
|------|----------|
| 运行时 | Tokio (full features) |
| HTTP/WS | Axum 0.8 |
| LLM 协议 | OpenAI-compatible streaming |
| 数据库 | SQLite (rusqlite bundled) + FTS5 |
| MCP | rmcp v1.8.0 (stdio + SSE) |
| 浏览器 | chromiumoxide (CDP) |
| 加密 | aes-gcm (AES-256-GCM) |
| YARA | boreal (规则扫描) |
| PE 解析 | goblin + iced-x86 |
| SSH | russh (SSH 客户端) |
| 序列化 | serde + serde_json + serde_yaml + toml |
| 日志分析 | regex + pcap-parser |
| 日志 | tracing + tracing-subscriber |
| CLI | clap (derive) |

## 📦 构建

```bash
# 编译 release 版本（~30MB，LTO + strip）
cargo build --release

# 运行测试
cargo test

# 代码检查
cargo check
```

Release profile：`opt-level = 3`、`lto = true`、`strip = true`

## 📚 文档

- [CLI_OUTPUT_IMPROVEMENTS.md](CLI_OUTPUT_IMPROVEMENTS.md) - CLI 输出改进详情
- [RELEASE_NOTES.md](RELEASE_NOTES.md) - v1.0.0 发布说明
- [HARNESS_INTEGRATION.md](HARNESS_INTEGRATION.md) - LongHorizon-Harness 集成指南

## 🙏 致谢

原始项目：[AI_IT_AGENT](https://github.com/wolf0x/AI_IT_AGENT) by wolf0x

本项目从 Windows IR 专用工具演进为跨平台通用智能体，保留了核心的 Agent 架构和工具系统设计。

## 📄 License

MIT

---

**RustAgent - 跨平台通用 AI 智能体，从 Windows IR 到通用自动化的演进之旅** 🚀

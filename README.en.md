**[English](#)** | **[中文](README.md)**

# RustAgent

Cross-platform general-purpose AI agent with Web Dashboard and CLI modes, WebSocket gateway, multi-model support, 31+ built-in tools, permission control, persistent memory, and task scheduling. Single-binary deployment, ready out of the box.

## 🎯 Positioning

RustAgent is a **cross-platform general-purpose AI agent** that evolved from a Windows IR (Incident Response) system to support both Linux and Windows platforms.

**Two Operating Modes**:
- **Web Dashboard Mode**: Interactive browser-based dashboard with real-time chat, tool call visualization, settings management
- **CLI Headless Mode**: Command-line automation for script integration, scheduled tasks, and external frameworks like LongHorizon-Harness

**Core Capabilities**:
- 🔧 **31+ Built-in Tools**: File operations, shell execution, browser automation, malware analysis, log analysis, SSH remote execution
- 🧠 **Intelligent Memory System**: SQLite + FTS5 full-text search + knowledge distillation
- 🔒 **Security Permission System**: Category gates + Intent Policy engine (Block/Audit/Pass)
- 🌐 **Multi-model Support**: OpenAI-compatible API, supporting DeepSeek, GPT-4, Qwen, etc.
- ⏰ **Task Scheduling**: Built-in CRON scheduler for periodic monitoring and automation
- 📊 **Profile System**: Shared/isolated workspace for multi-environment management

## 🚀 Quick Start

### Installation

```bash
# Build from source
git clone https://github.com/wolf0x/RustAgent-LX.git
cd RustAgent-LX
cargo build --release

# Binary output
./target/release/RustAgent  # Linux
.\target\release\RustAgent.exe  # Windows
```

### Running Modes

#### Web Dashboard Mode

```bash
# Start Web Dashboard (default port 7788)
RustAgent web

# Specify port
RustAgent web --port 8080

# Use specific profile
RustAgent --profile web

# Access Dashboard
# Open browser at http://localhost:7788
# Login with password shown at startup
```

#### CLI Headless Mode

```bash
# Execute task and exit
RustAgent cli "Check disk space and report usage"

# Use specific profile
RustAgent --profile headless "Analyze system logs"

# Read task from file (for automation)
RustAgent --prompt-file task.md

# Execution mode switching
RustAgent --mode instant cli "Quick task"  # Default, single-pass
RustAgent --mode expert cli "Complex analysis"   # Multi-round depth

# Permission control
RustAgent --auto-approve cli "Auto-approve all permissions"
RustAgent --read-only cli "Read-only mode"
```

### Profile System

```bash
# Default: all profiles share workspace
RustAgent web                    # Uses ~/.RustAgent/workspace/
RustAgent cli "task"             # Uses ~/.RustAgent/workspace/

# Isolated mode: each profile has independent workspace
RustAgent --profile myproject --isolated cli "task"
# Uses ~/.RustAgent/workspace/profiles/myproject/
```

## 🏗️ Core Architecture

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
│    ├── Agent trait → EventStream (9 event types)│
│    ├── CJK-aware token budget history trimming  │
│    ├── Thinking / ToolCall / ToolResult output  │
│    └── Truncated JSON repair                    │
├─────────────────────────────────────────────────┤
│  CLI Mode (Headless)                            │
│    ├── stdin/stdout interaction                 │
│    ├── Complete execution process (stderr)      │
│    └── Agent answer output (stdout)             │
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

## 🛠️ Tool System

### File Operations (5 tools)
- **FileRead** / **FileWrite** / **FileDelete** / **FileModify** / **FileList**
- Large file chunked reading (>300MB reads only first 1MB)
- Automatic permission checking

### Shell Execution
- **ShellExecTool**: bash/sh (Linux) or PowerShell/CMD (Windows)
- **Intent Policy Engine**: Semantic-level command analysis
  - **Block**: `rm -rf /`, `dd of=/dev/sda`, `mkfs`, and other irreversible operations
  - **Audit**: `rm`, `kill`, `systemctl stop`, and other high-risk operations (logged)
  - **Pass**: `ps`, `ls`, `netstat`, and other read-only commands

### Browser Automation (2 tools)
- **browser_cdp**: chromiumoxide CDP isolated browser (no login state, secure search)
- **browser_skill**: BSK extension controls user browser (operations requiring login state)

### Malware Analysis (3 tools)
- **malware_scan**: Boreal YARA rule scanning (custom rule sets supported)
- **malware_deep**: PE deep analysis (goblin parsing + iced-x86 disassembly)
- **malware_analysis**: Comprehensive malware analysis

### Log Analysis (3 tools)
- **ir_log_parse**: Universal log parsing (auto-identify format, security pattern matching)
- **ir_pcap_analyze**: PCAP traffic analysis (protocol distribution, flow tracking, DNS/HTTP extraction)
- **ir_weblog_scan**: Web log scanning (SQLi/XSS/RCE detection)

### Linux Incident Response (13 tools)
| Tool | Capability |
|------|------------|
| linux_ir_process | Process enumeration, resource usage, command line analysis |
| linux_ir_network | TCP/UDP connections, listening ports, associated processes |
| linux_ir_persistence | Persistence location scanning (cron, systemd, SSH) |
| linux_ir_rootkit | Rootkit detection (common signature matching) |
| linux_ir_file | File system anomaly detection |
| linux_ir_web | Web application log analysis |
| linux_ir_mining | Mining program detection |
| linux_ir_lateral | Lateral movement trace detection |
| linux_ir_auth | Authentication log analysis |
| linux_ir_backdoor | Backdoor detection |
| linux_ir_bruteforce | Brute force detection |
| linux_ir_integrity | File integrity checking |
| linux_ir_config | System configuration auditing |

### SSH Remote Execution (2 tools)
- **linux_ssh**: SSH remote command execution (russh implementation)
- **ir_linux**: Linux remote incident response

### Web Tools (2 tools)
- **web_fetch**: HTTP/HTTPS page fetching
- **browser_open**: Open default browser (xdg-open/open)

### Productivity Tools (4 tools)
- **TodoWrite**: Task planning and progress tracking
- **CronManage**: CRON task management
- **MemoryMd**: MEMORY.md read/write (long-term memory)
- **SysRemind**: System reminders (notifications to Web Dashboard)

### MCP Dynamic Tools
- Connect to external tool servers via MCP protocol
- Runtime dynamic registration to ToolRegistry
- Supports stdio + SSE dual transport

### External Tool Discovery
- Auto-discover and register executables in `workspace/tools/` directory
- Supports bash, python3, perl, and other scripts

## 🔒 Security Features

### Permission System
- **Five-level permission categories**: read / write / delete / modify / execute
- **Asynchronous user authorization**: Web Dashboard pushes authorization requests via WebSocket
- **Cross-category bypass detection**: Prevents bypassing file_delete permissions via shell_exec

### Command Intent Policy
- **Semantic parsing**: Extract verb + targets, replacing traditional string blacklists
- **Three-level judgment**:
  - **Block**: Absolutely prohibited (disk formatting, security log clearing, etc.)
  - **Audit**: Audit pass (file deletion, process termination, etc., logged)
  - **Pass**: Silent pass (read-only queries, etc.)
- **Linux-specific rules**:
  - Block: `rm -rf /`, `dd of=/dev/sda`, `mkfs`, fork bombs, etc.
  - Audit: `rm`, `kill`, `systemctl stop`, mount operations, etc.
  - Pass: `ps`, `ls`, `netstat`, `cat`, etc.

### Other Security Features
- **API key encryption**: AES-256-GCM, key derived from machine-id
- **Password-protected Dashboard**: `.password` file protection
- **CDP browser isolation**: chromiumoxide independent instance, no user login state
- **Permission denial strong feedback**: Returns strong error messages to LLM on denial

## 🧠 Memory System

### SQLite + FTS5 Conversation Memory
- 4-layer schema evolution
- CJK bigram tokenization (optimized for Chinese/Japanese/Korean)
- BM25-ranked full-text search
- Automatic conversation history cleanup (3 days/50 entries)

### Knowledge Distillation
- Auto-triggered at session end
- LLM extracts structured knowledge entries
- Writes to 5 category files under `workspace/knowledge/`
- Append-only design, write-only, no modifications

### File Memory (MEMORY.md)
- LLM-maintained personal notes
- Auto-injected into System Prompt
- Complements SQLite memory

## ⏰ Scheduling System

- **CRON expressions**: Standard 5-field (minute hour day month weekday)
- **Interval syntax**: `every 5m`, `every 2h`, and other natural language styles
- **JSON persistence**: Task definitions stored in `cron_tasks.json`
- **30-second polling**: Scheduler checks due tasks every 30 seconds
- **Heartbeat mechanism**: Reads periodic health check checklist from `HEARTBEAT.md`

## 📊 CLI Output

### Output Stream Separation
- **stdout**: Only agent's final answer (`TextDelta`)
- **stderr**: All execution process (thinking, tool calls, results, errors)

### Event Types
| Icon | Event | Description |
|------|-------|-------------|
| 💭 | Thinking | Agent's reasoning process |
| 🔧 | ToolCall | Tool calls and arguments |
| ✅ | ToolResult | Tool execution results |
| ⏳ | Progress | Progress for long-running tools |
| ❌ | Error | Error messages |
| 📝 | TextDelta | Agent's final answer |

### Usage Examples

```bash
# View complete execution process
RustAgent cli "Check disk space" 2>&1

# Save only agent's answer
RustAgent cli "Check disk space" > answer.txt

# Save complete log
RustAgent --mode expert cli "Analyze system" > full_log.txt 2>&1
```

## 🔗 LongHorizon-Harness Integration

RustAgent supports running as an agent backend for LongHorizon-Harness:

```bash
# Harness command template
RustAgent --profile headless \
  --prompt-file {prompt_path} \
  --timeout {seconds} \
  --auto-approve
```

- `--prompt-file`: Read task from file
- `--timeout`: Execution timeout control
- `--auto-approve`: Auto-approve all permissions
- `--trajectory`: JSONL trajectory output

Python adapter: `adapters/rustagent_adapter.py`

## 📁 Configuration

### Working Directory

**Linux**: `~/.RustAgent/workspace/`  
**Windows**: `%USERPROFILE%\.RustAgent\workspace\`

### Directory Structure

```
workspace/
├── config.toml          # Main configuration
├── models.json          # Model configuration (API Key encrypted)
├── mcp_servers.json     # MCP server configuration
├── cron_tasks.json      # Scheduled task definitions
├── .password            # Dashboard access password (Web mode only)
├── memory/
│   └── memory.db        # SQLite memory database
├── knowledge/           # Knowledge distillation output
├── skills/              # Skill directory
├── tools/               # External tool directory
├── logs/                # Runtime logs (rustagent-YYYY-MM-DD.N.log)
├── static/              # Dashboard static assets
└── output/              # Tool outputs
```

### Configuration File Examples

**config.toml**:
```toml
[server]
host = "127.0.0.1"
port = 7788

[agent]
working_dir = "."
max_iterations = 100
mode = "instant"  # or "expert"
timezone_offset = 8
```

**models.json**:
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

## 🛠️ Tech Stack

| Component | Technology |
|-----------|------------|
| Runtime | Tokio (full features) |
| HTTP/WS | Axum 0.8 |
| LLM Protocol | OpenAI-compatible streaming |
| Database | SQLite (rusqlite bundled) + FTS5 |
| MCP | rmcp v1.8.0 (stdio + SSE) |
| Browser | chromiumoxide (CDP) |
| Encryption | aes-gcm (AES-256-GCM) |
| YARA | boreal (rule scanning) |
| PE Parser | goblin + iced-x86 |
| SSH | russh (SSH client) |
| Serialization | serde + serde_json + serde_yaml + toml |
| Log Analysis | regex + pcap-parser |
| Logging | tracing + tracing-subscriber |
| CLI | clap (derive) |

## 📦 Building

```bash
# Build release version (~30MB, LTO + strip)
cargo build --release

# Run tests
cargo test

# Code check
cargo check
```

Release profile: `opt-level = 3`, `lto = true`, `strip = true`

## 📚 Documentation

- [CLI_OUTPUT_IMPROVEMENTS.md](CLI_OUTPUT_IMPROVEMENTS.md) - CLI output improvements details
- [RELEASE_NOTES.md](RELEASE_NOTES.md) - v1.0.0 release notes
- [HARNESS_INTEGRATION.md](HARNESS_INTEGRATION.md) - LongHorizon-Harness integration guide

## 🙏 Acknowledgments

Original project: [AI_IT_AGENT](https://github.com/wolf0x/AI_IT_AGENT) by wolf0x

This project evolved from a Windows IR-specific tool to a cross-platform general-purpose agent, preserving the core Agent architecture and tool system design.

## 📄 License

MIT

---

**RustAgent - Cross-platform general-purpose AI agent, evolving from Windows IR to general-purpose automation** 🚀

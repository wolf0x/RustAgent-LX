# RustAgent v1.0.0 Release Notes

## 🎉 First Stable Release as Cross-Platform General-Purpose Agent

This release marks the transformation from a Windows-specific IR (Incident Response) agent to a cross-platform general-purpose AI agent.

## ✨ Major Features

### 🔄 Cross-Platform Support
- **Linux**: Full support with bash/sh, xdg-open, and Linux-specific tools
- **Windows**: Maintained compatibility (via conditional compilation)
- **30MB optimized binary**: Stripped and LTO-optimized release build

### 🖥️ Web Dashboard Mode
```bash
RustAgent web                    # Start web dashboard on localhost:7788
RustAgent --profile web          # Use 'web' profile configuration
RustAgent --port 8080 web        # Custom port
```
- Password-protected access (auto-generated on startup)
- Interactive chat interface
- Real-time agent execution monitoring
- Settings management UI

### ⚡ CLI Headless Mode
```bash
RustAgent cli "your task"        # Execute task and exit
RustAgent --profile headless "task"  # Use 'headless' profile
RustAgent --prompt-file task.md  # Read task from file (for automation)
```
- Non-interactive execution for automation
- Clean stdout output (agent responses only)
- System logs written to file only (`workspace/logs/`)
- Exit codes: 0 (success), 1 (error)

### 📁 Profile System
```bash
# Shared workspace (default)
RustAgent --profile web          # All profiles share ~/.RustAgent/workspace/
RustAgent --profile cli "task"   # Shared config, models.json, skills, etc.

# Isolated workspace
RustAgent --profile myproject --isolated cli "task"
# Uses ~/.RustAgent/workspace/profiles/myproject/ independently
```

### 🎯 Execution Modes
```bash
RustAgent --mode instant cli "task"  # Fast single-pass (default)
RustAgent --mode expert cli "task"   # Thorough multi-round analysis
```

### 🔗 LongHorizon-Harness Integration
```bash
# For external harness automation
RustAgent --profile headless \
  --prompt-file {prompt_path} \
  --timeout {seconds} \
  --auto-approve
```
- `--prompt-file`: Read task from file
- `--timeout`: Execution time limit
- `--auto-approve`: Auto-approve all permissions
- `--trajectory`: JSONL output for harness dashboard

## 🛠️ Technical Changes

### Removed Windows-Specific Components
- **21 Windows IR tools deleted**: `ir_scan`, `ir_process`, `ir_account`, etc.
- **2 directories removed**: `computer_use/`, `forensics/`
- **Dependencies removed**: `windows` crate, `winresource`, `evtx`, `prefetch-core`

### Rewritten for Linux
- **shell_exec.rs**: PowerShell → bash/sh
- **browser_open.rs**: cmd.exe → xdg-open
- **permission.rs**: Windows IntentPolicy → LinuxIntentPolicy
- **external_exec.rs**: PowerShell/cmd → bash/python3

### New Features
- **CLI parsing**: Added `clap` for argument parsing
- **Profile system**: Shared/isolated workspace support
- **Log separation**: System logs (file) vs agent output (console) in CLI mode
- **Permission gating**: Intent-based policy with block/audit rules

## 📊 Testing & Quality

- ✅ **54 tests passing**: All unit tests pass
- ✅ **Linux permission validation**: Block rules for dangerous commands (`rm -rf /`, `dd of=/dev/sda`, etc.)
- ✅ **Cross-platform path resolution**: `HOME` (Linux) / `USERPROFILE` (Windows)
- ✅ **Config migration**: Removed Windows hard-coded paths

## 📦 Installation

### From Source
```bash
git clone https://github.com/YOUR_USERNAME/RustAgent-LX.git
cd RustAgent-LX
cargo build --release
./target/release/RustAgent --help
```

### Binary Release
- **Linux x86_64**: `RustAgent-linux-x86_64` (30MB)
- **Windows x86_64**: `RustAgent-windows-x86_64.exe` (coming soon)

## 🚀 Quick Start

```bash
# Web mode - interactive dashboard
./RustAgent web

# CLI mode - execute task
./RustAgent cli "check disk space and report usage"

# With custom profile
./RustAgent --profile production cli "analyze system logs"

# For automation (e.g., cron jobs)
echo "monitor system health" | ./RustAgent --profile headless
```

## 📝 Configuration

Config file: `~/.RustAgent/workspace/config.toml`
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

Model config: `~/.RustAgent/workspace/models.json`
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

## 🐛 Known Issues

- Windows support not yet tested in this release
- Some browser automation features may require additional setup

## 📚 Documentation

- [README.md](README.md) - Project overview
- [CLI_USAGE.md](CLI_USAGE.md) - CLI usage guide (coming soon)
- [HARNESS_INTEGRATION.md](HARNESS_INTEGRATION.md) - LongHorizon-Harness setup (coming soon)

## 🙏 Acknowledgments

Original project: [AI_IT_AGENT](https://github.com/wolf0x/AI_IT_AGENT) by wolf0x

---

**Full Changelog**: https://github.com/YOUR_USERNAME/RustAgent-LX/commits/v1.0.0

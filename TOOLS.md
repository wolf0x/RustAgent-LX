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

4. **路径含空格时必须加引号**
   - Linux: `"/usr/local/my program/bin/tool"`
   - Windows: `"C:\Program Files\tool.exe"`

### 本地路径（按实际环境修改）

**Linux 环境**：
- Python: `/usr/bin/python3`
- Node: `/usr/bin/node`
- Git: `/usr/bin/git`
- 工作区: `~/.RustAgent/workspace`
- 产出物: `~/.RustAgent/workspace/output`

**Windows 环境**（仅在 Windows 运行时使用）：
- Git: `"C:\Program Files\Git\bin\git.exe"`
- Python: `"C:\Users\%USERNAME%\AppData\Local\Programs\Python\Python312\python.exe"`
- Node: `"C:\Program Files\nodejs\node.exe"`
- 工作区: `%USERPROFILE%\.RustAgent\workspace`
- 产出物: `%USERPROFILE%\.RustAgent\workspace\output`

### 产出物规范

**所有文件产出必须写入 `output/` 目录**，使用描述性文件名：

- 报告: `output/analysis_report_2026-08-15.html`
- 截图: `output/screenshot_2026-08-15_143022.png`
- 分析结果: `output/scan_results_2026-08-15.json`

**引用产出物时，使用工具返回的完整路径**，不要简化为文件名。

## 平台检测

Agent 应该根据运行环境选择合适的命令：

- **Linux**: 使用 `bash`、`sh`、`ls`、`cat`、`grep` 等
- **Windows**: 使用 `powershell`、`cmd`、`dir`、`type` 等
- **跨平台**: 优先使用 Python 脚本或跨平台工具

## 本地环境备注

_在此添加你的本地环境特定配置：_

- SSH 主机: （待配置）
- 特殊工具: （待配置）
- 其他: （待配置）

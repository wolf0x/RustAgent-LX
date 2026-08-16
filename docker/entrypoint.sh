#!/bin/bash
# ============================================================
# RustAgentX — TSECBench 竞赛容器启动脚本
# 容器启动后自动执行：渲染凭证 → VPN 预检 → 启动自动解题
# ============================================================
set -u  # 未定义变量报错，但不退出（凭证缺失时给出明确提示）

WORKSPACE="/root/.RustAgent/workspace"
TASK_TEMPLATE="/opt/agent/task_template.md"
TASK_RENDERED="/tmp/task.md"
LOG_FILE="$WORKSPACE/logs/entrypoint.log"

mkdir -p "$WORKSPACE" "$WORKSPACE/skills" "$WORKSPACE/tools" "$WORKSPACE/logs" "$WORKSPACE/output"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

log "=== RustAgentX TSECBench entrypoint starting ==="

# ── 1. 校验必需的环境变量 ──────────────────────────────
MISSING=0
for var in BENCHMARK_TOKEN BENCHMARK_BASE_URL; do
    if [ -z "${!var:-}" ]; then
        log "ERROR: required environment variable $var is not set"
        MISSING=1
    fi
done
if [ "$MISSING" -eq 1 ]; then
    log "FATAL: missing required credentials, cannot start benchmark"
    exit 1
fi

# MODEL_API_KEY 由 models.json 的 api_key_env 引用
if [ -z "${MODEL_API_KEY:-}" ]; then
    log "WARNING: MODEL_API_KEY is not set — LLM calls will fail"
fi

log "BENCHMARK_BASE_URL=$BENCHMARK_BASE_URL"
log "BENCHMARK_TOKEN=****$(echo "$BENCHMARK_TOKEN" | tail -c 5)"
log "MODEL_API_KEY is ${MODEL_API_KEY:+set}${MODEL_API_KEY:-NOT SET}"

# ── 2. 生成 models.json（API Key 用环境变量方式，避免机器绑定加密问题）──
cat > "$WORKSPACE/models.json" << EOF
[{
  "name": "${MODEL_NAME:-deepseek-v3}",
  "api_base": "${MODEL_API_BASE:-https://api.deepseek.com}",
  "api_key": "",
  "api_key_env": "MODEL_API_KEY",
  "context_window": 128000,
  "max_tokens": 16384,
  "temperature": 0.7
}]
EOF
export MODEL_API_KEY
log "models.json generated (api_key via env: MODEL_API_KEY)"

# ── 3. 复制竞赛专用 AGENTS.md / TOOLS.md / skills ────────
if [ -f /opt/agent/AGENTS.md ]; then
    cp /opt/agent/AGENTS.md "$WORKSPACE/AGENTS.md"
    log "AGENTS.md copied"
fi
if [ -f /opt/agent/TOOLS.md ]; then
    cp /opt/agent/TOOLS.md "$WORKSPACE/TOOLS.md"
    log "TOOLS.md copied"
fi
if [ -d /opt/agent/skills ]; then
    cp -r /opt/agent/skills/* "$WORKSPACE/skills/" 2>/dev/null || true
    log "skills copied: $(ls "$WORKSPACE/skills/" | tr '\n' ' ')"
fi

# ── 4. envsubst 渲染任务提示词（注入真实凭证）──────────────
# 只替换指定的变量，避免误伤提示词中的其他 $ 内容
if [ -f "$TASK_TEMPLATE" ]; then
    envsubst '${BENCHMARK_TOKEN} ${BENCHMARK_BASE_URL}' \
        < "$TASK_TEMPLATE" > "$TASK_RENDERED"
    log "Task prompt rendered: $TASK_TEMPLATE -> $TASK_RENDERED"
    log "Rendered token placeholder check: $(grep -c 'BENCHMARK_TOKEN' "$TASK_RENDERED") occurrence(s)"
else
    log "ERROR: task template not found at $TASK_TEMPLATE"
    exit 1
fi

# ── 5. VPN 联通预检（脚本层兜底，重试 10 次）──────────────
VPN_OK=0
for i in $(seq 1 10); do
    RESP=$(curl -s --max-time 5 http://10.0.100.58 2>/dev/null || true)
    if echo "$RESP" | grep -q '"status"[[:space:]]*:[[:space:]]*"ok"'; then
        log "VPN pre-check passed (attempt $i): $RESP"
        VPN_OK=1
        break
    fi
    log "VPN pre-check attempt $i/10 failed, retrying in 10s..."
    sleep 10
done
if [ "$VPN_OK" -ne 1 ]; then
    log "FATAL: VPN detection failed, please check target range VPN configuration"
    echo "VPN detection failed, please check target range VPN configuration"
    exit 1
fi

# ── 6. 启动 RustAgentX 自动解题 ──────────────────────────
# exec 替换当前进程，容器的生命周期即 Agent 的生命周期
TIMEOUT="${BENCHMARK_TIMEOUT:-7200}"
log "Starting RustAgentX (expert mode, timeout=${TIMEOUT}s)..."

exec /usr/local/bin/rustagentx cli \
    --auto-approve \
    --mode expert \
    --timeout "$TIMEOUT" \
    --prompt-file "$TASK_RENDERED"

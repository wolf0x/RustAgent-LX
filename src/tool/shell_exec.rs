use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use super::Tool;
use crate::context::ToolContext;
use crate::error::AgentResult;
use crate::policy::{LinuxIntentPolicy, LinuxIntentVerdict};

pub struct ShellExecTool;

#[async_trait]
impl Tool for ShellExecTool {
    fn name(&self) -> &str { "shell_exec" }
    fn description(&self) -> &str {
        "Execute a shell command using bash. Returns stdout, stderr, and exit code.\n\n\
         IMPORTANT: DO NOT use this tool for SSH commands to remote Linux/Unix hosts. Use the 'linux_ssh' tool instead."
    }
    fn is_builtin(&self) -> bool { true }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Command to execute" },
                "shell": { "type": "string", "description": "Shell to use: 'bash' (default) or 'sh'", "enum": ["bash", "sh"] },
                "timeout_secs": { "type": "integer", "description": "Timeout in seconds (default 30)" }
            },
            "required": ["command"]
        })
    }
    async fn execute(&self, args: Value, _ctx: &ToolContext) -> AgentResult<Value> {
        let command = args["command"].as_str().ok_or_else(|| "Missing 'command'".to_string())?;
        let shell = args["shell"].as_str().unwrap_or("bash");
        let timeout = args["timeout_secs"].as_u64().unwrap_or(30);

        // ── Intent Policy evaluation ──
        // - Block: catastrophic irreversible ops → hard reject regardless of permissions
        // - Audit: high-risk but legitimate → log and proceed
        // - Pass: normal → silent
        let policy = LinuxIntentPolicy::new();
        match policy.evaluate(command) {
            LinuxIntentVerdict::Block { reason } => {
                return Err(format!(
                    "BLOCKED (safety interlock): {}. \
                     This operation is irreversible and cannot be executed through RustAgent. \
                     If you truly need this, execute it manually outside the agent.",
                    reason
                ).into());
            }
            LinuxIntentVerdict::Audit { reason } => {
                tracing::warn!(
                    "[AUDIT] shell_exec high-risk: {} | shell={} | command={}",
                    reason, shell, command
                );
            }
            LinuxIntentVerdict::Pass => { /* silent */ }
        }

        let shell_bin = match shell {
            "sh" => "sh",
            _ => "bash",
        };

        let mut cmd = Command::new(shell_bin);
        cmd.args(["-c", command]);
        cmd.kill_on_drop(true);

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout),
            cmd.output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);
                Ok(json!({
                    "stdout": stdout,
                    "stderr": stderr,
                    "exit_code": exit_code
                }))
            }
            Ok(Err(e)) => Err(format!("Failed to execute: {}", e).into()),
            Err(_) => Err(format!("Command timed out after {}s", timeout).into()),
        }
    }
}

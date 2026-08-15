use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;

use super::Tool;
use crate::context::ToolContext;
use crate::error::AgentResult;

/// If `path` is an absolute path located under `workspace_dir`, returns its
/// workspace-relative forward-slash path (e.g. /home/user/output/a.html -> "output/a.html").
/// Returns None otherwise.
fn absolute_path_under_workspace(path: &str, workspace_dir: &str) -> Option<String> {
    if workspace_dir.is_empty() {
        return None;
    }
    let p = std::path::Path::new(path);
    if !p.is_absolute() {
        return None;
    }
    let ws = std::path::Path::new(workspace_dir);
    let rel = p.strip_prefix(ws).ok()?;
    if rel.as_os_str().is_empty() {
        return None;
    }
    Some(rel.to_string_lossy().to_string())
}

pub struct BrowserOpenTool;

#[async_trait]
impl Tool for BrowserOpenTool {
    fn name(&self) -> &str { "browser_open" }
    fn description(&self) -> &str {
        "Open a URL in the default web browser. If no URL scheme is provided, 'https://' is prepended."
    }
    fn is_builtin(&self) -> bool { true }
    fn is_read_only(&self) -> bool { true }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL to open in browser" }
            },
            "required": ["url"]
        })
    }
    async fn execute(&self, args: Value, ctx: &ToolContext) -> AgentResult<Value> {
        let mut url = args["url"].as_str().ok_or_else(|| "Missing 'url'".to_string())?.to_string();

        // Reject URLs containing shell metacharacters to prevent command injection
        let dangerous_chars = ['&', '|', '>', '<', '^', '`', ';'];
        if url.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(format!("URL contains dangerous characters: {:?}. Only http/https URLs with standard characters are allowed.", dangerous_chars).into());
        }

        if url.trim().is_empty() {
            return Err("'url' is empty".to_string().into());
        }

        let lower = url.to_ascii_lowercase();

        // Smart URL detection and conversion
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://") {
            // Already a proper URL, use as-is
        } else if let Some(rel) = absolute_path_under_workspace(&url, &ctx.workspace_dir) {
            url = format!("http://localhost:7788/workspace/{}", rel);
        } else if lower.starts_with("output/") || lower.starts_with("workspace/") {
            let workspace_path = if lower.starts_with("output/") {
                format!("workspace/{}", url)
            } else {
                url
            };
            url = format!("http://localhost:7788/{}", workspace_path);
        } else {
            // Assume it's a domain or URL without scheme
            url = format!("https://{}", url);
        }

        // Use xdg-open on Linux
        let result = Command::new("xdg-open")
            .arg(&url)
            .spawn();

        match result {
            Ok(_) => Ok(json!({ "status": "opened", "url": url })),
            Err(e) => Err(format!("Failed to open browser: {}", e).into()),
        }
    }
}

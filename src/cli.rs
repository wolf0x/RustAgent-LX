//! CLI argument parsing and mode resolution.
//!
//! Mode selection:
//!   RustAgentX web [OPTIONS]              → Web dashboard mode
//!   RustAgentX cli [OPTIONS] <TASK>       → Headless CLI mode
//!   RustAgentX --profile web [OPTIONS]    → Same as `web`
//!   RustAgentX --profile headless <TASK>  → Same as `cli`
//!   RustAgentX --profile cli <TASK>       → Same as `cli`
//!
//! Without arguments, prints help.
//!
//! Profiles:
//!   --profile <name> selects workspace/profiles/<name>/ as the workspace directory.

use clap::Parser;

/// RustAgentX — Cross-platform general-purpose AI agent
#[derive(Parser, Debug)]
#[command(name = "RustAgentX", version, about)]
pub struct Cli {
    /// Mode: "web" (dashboard) or "cli" (headless automation).
    /// Remaining args after "cli" are the task text.
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,

    /// Profile name (uses workspace/profiles/<name>/ directory).
    /// Implies mode: "web" → web mode, "headless"/"cli" → headless mode.
    #[arg(long)]
    pub profile: Option<String>,

    /// Read task from a file (for external harness integration)
    #[arg(long)]
    pub prompt_file: Option<String>,

    /// Run mode: "instant" (fast) or "expert" (thorough, multi-round)
    #[arg(long)]
    pub mode: Option<String>,

    /// Use isolated workspace for this profile (default: shared workspace)
    #[arg(long)]
    pub isolated: bool,

    /// Path to config.toml file (overrides profile/workspace config)
    #[arg(long)]
    pub config: Option<String>,

    /// Override workspace directory
    #[arg(long)]
    pub workspace: Option<String>,

    /// Server host address (web mode)
    #[arg(long)]
    pub host: Option<String>,

    /// Server port (web mode)
    #[arg(long)]
    pub port: Option<u16>,

    /// Max agent iterations per conversation turn
    #[arg(long)]
    pub max_iterations: Option<usize>,

    /// Tool execution timeout in seconds
    #[arg(long)]
    pub tool_timeout: Option<usize>,

    /// Primary model name
    #[arg(long)]
    pub model: Option<String>,

    /// Fallback model name
    #[arg(long)]
    pub fallback_model: Option<String>,

    /// Context window threshold percentage (0-100)
    #[arg(long)]
    pub context_threshold: Option<usize>,

    /// Rabbit hole detection threshold
    #[arg(long)]
    pub rabbit_hole: Option<usize>,

    /// Max tool retries
    #[arg(long)]
    pub max_retries: Option<usize>,

    /// Enable parallel IR tool execution
    #[arg(long)]
    pub parallel_ir: Option<bool>,

    /// Timezone offset in hours (e.g., 8 for UTC+8)
    #[arg(long)]
    pub timezone: Option<i8>,

    /// Log level filter (e.g., "info", "debug", "trace")
    #[arg(long)]
    pub log_level: Option<String>,

    /// Auto-approve all permission requests (headless mode)
    #[arg(long)]
    pub auto_approve: bool,

    /// Read-only mode: only allow read-category tools (for auditor roles)
    #[arg(long)]
    pub read_only: bool,

    /// Working directory for task execution (defaults to CWD)
    #[arg(long)]
    pub workdir: Option<String>,

    /// Timeout for the entire headless execution in seconds
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Write trajectory JSONL to this path (for external harness integration)
    #[arg(long)]
    pub trajectory: Option<String>,
}

/// Resolved startup configuration after parsing CLI args
#[derive(Debug, Clone)]
pub struct ResolvedCli {
    /// The resolved mode (web/headless)
    pub mode: RunMode,
    /// The task to execute (headless mode only)
    pub task: Option<String>,
    /// Profile name (if specified)
    pub profile: Option<String>,
    /// Path to config file (if explicitly specified)
    pub config_path: Option<String>,
    /// Override workspace directory (if explicitly specified)
    pub workspace_override: Option<String>,
    /// Working directory for task execution
    pub workdir: Option<String>,
    /// CLI overrides for config values
    pub overrides: ConfigOverrides,
    /// Permission policy for headless mode
    pub permission_policy: PermissionPolicy,
    /// Execution timeout in seconds
    pub execution_timeout: Option<u64>,
    /// Trajectory output path
    pub trajectory_path: Option<String>,
    /// Log level filter
    pub log_level: Option<String>,
    /// Run mode: "instant" or "expert"
    pub run_mode: Option<String>,
    /// Whether to use isolated workspace for this profile
    pub isolated: bool,
}

/// Resolved run mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunMode {
    /// Web dashboard mode with Axum server
    Web,
    /// Headless command-line mode for automation
    Headless,
}

/// Permission policy for headless mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PermissionPolicy {
    /// Normal interactive permission flow (default for web mode)
    Interactive,
    /// Auto-approve all operations
    AutoApprove,
    /// Only allow read-category tools
    ReadOnly,
}

/// CLI overrides for configuration values
#[derive(Debug, Clone, Default)]
pub struct ConfigOverrides {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub max_iterations: Option<usize>,
    pub tool_timeout_secs: Option<usize>,
    pub primary_model: Option<String>,
    pub fallback_model: Option<String>,
    pub context_window_threshold: Option<usize>,
    pub rabbit_hole_threshold: Option<usize>,
    pub max_tool_retries: Option<usize>,
    pub parallel_ir_tools: Option<bool>,
    pub timezone_offset: Option<i8>,
}

impl Cli {
    /// Parse CLI arguments and resolve into a structured configuration.
    /// Returns None if no valid mode is specified (should show help).
    pub fn resolve(self) -> Option<ResolvedCli> {
        let mut args_iter = self.args.iter().peekable();
        let mut mode: Option<RunMode> = None;
        let mut task_parts: Vec<String> = Vec::new();
        let mut profile = self.profile.clone();

        // Determine mode from positional args or profile name
        if let Some(first) = args_iter.peek() {
            match first.as_str() {
                "web" => {
                    mode = Some(RunMode::Web);
                    args_iter.next(); // consume "web"
                    // Set default profile to "web" if not explicitly set
                    if profile.is_none() {
                        profile = Some("web".to_string());
                    }
                }
                "cli" => {
                    mode = Some(RunMode::Headless);
                    args_iter.next(); // consume "cli"
                    // Remaining args are the task
                    task_parts = args_iter.map(|s| s.clone()).collect();
                    // Set default profile to "cli" if not explicitly set
                    if profile.is_none() {
                        profile = Some("cli".to_string());
                    }
                }
                _ => {
                    // First arg is not a mode keyword — treat all args as task text
                    // Mode will be determined by --profile
                    task_parts = self.args.clone();
                }
            }
        }

        // If mode not determined by positional arg, infer from --profile
        if mode.is_none() {
            if let Some(ref p) = profile {
                match p.as_str() {
                    "web" => mode = Some(RunMode::Web),
                    "headless" | "cli" => mode = Some(RunMode::Headless),
                    _ => {
                        // Custom profile: if task is present → headless, else → web
                        if !task_parts.is_empty() || self.prompt_file.is_some() {
                            mode = Some(RunMode::Headless);
                        } else {
                            mode = Some(RunMode::Web);
                        }
                    }
                }
            }
        }

        // No mode determined → no valid invocation
        let mode = mode?;

        // Resolve task text: --prompt-file > positional task args
        let task = if let Some(ref prompt_file) = self.prompt_file {
            match std::fs::read_to_string(prompt_file) {
                Ok(content) if !content.trim().is_empty() => Some(content.trim().to_string()),
                Ok(_) => {
                    eprintln!("Warning: prompt file '{}' is empty", prompt_file);
                    None
                }
                Err(e) => {
                    eprintln!("Error: cannot read prompt file '{}': {}", prompt_file, e);
                    None
                }
            }
        } else if !task_parts.is_empty() {
            Some(task_parts.join(" "))
        } else {
            None
        };

        // Headless mode requires a task
        if mode == RunMode::Headless && task.is_none() {
            eprintln!("Error: cli/headless mode requires a task.");
            eprintln!("Usage: RustAgentX cli \"your task here\"");
            eprintln!("       RustAgentX --profile headless \"your task here\"");
            return None;
        }

        // Determine permission policy
        let permission_policy = if self.read_only {
            PermissionPolicy::ReadOnly
        } else if self.auto_approve || mode == RunMode::Headless {
            PermissionPolicy::AutoApprove
        } else {
            PermissionPolicy::Interactive
        };

        let overrides = ConfigOverrides {
            host: self.host,
            port: self.port,
            max_iterations: self.max_iterations,
            tool_timeout_secs: self.tool_timeout,
            primary_model: self.model,
            fallback_model: self.fallback_model,
            context_window_threshold: self.context_threshold,
            rabbit_hole_threshold: self.rabbit_hole,
            max_tool_retries: self.max_retries,
            parallel_ir_tools: self.parallel_ir,
            timezone_offset: self.timezone,
        };

        Some(ResolvedCli {
            mode,
            task,
            profile,
            config_path: self.config,
            workspace_override: self.workspace,
            workdir: self.workdir,
            overrides,
            permission_policy,
            execution_timeout: self.timeout,
            trajectory_path: self.trajectory,
            log_level: self.log_level,
            run_mode: self.mode,
            isolated: self.isolated,
        })
    }
}

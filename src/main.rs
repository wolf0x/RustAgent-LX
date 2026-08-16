#[allow(dead_code)]
mod agent;
#[allow(dead_code)]
mod callbacks;
mod checkpoint;
mod cli;
mod config;
mod crypto;
mod distill;
#[allow(dead_code)]
mod context;
#[allow(dead_code)]
mod error;
mod event_log;
mod external_tools;
mod heartbeat;
mod log;
#[allow(dead_code)]
mod managed;
mod memory;
mod model;
mod model_store;
mod permission;
mod policy;
#[allow(dead_code)]
mod runner;
mod scheduler;
mod server;
#[allow(dead_code)]
mod session;
mod skill;
#[allow(dead_code)]
mod tool;
mod web;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::agent::LlmAgent;
use crate::checkpoint::TaskCheckpointer;
use crate::config::Config;
use crate::external_tools::ExternalToolsManager;
use crate::log::ConversationLogger;
use crate::memory::MemoryStore;
use crate::model::openai::OpenAiProvider;
use crate::runner::Runner;
use crate::permission::{PermissionResolver, default_permissions};
use crate::scheduler::Scheduler;
use crate::heartbeat::Heartbeat;
use crate::server::AppState;
use crate::skill::SkillManager;
use crate::tool::mcp_client::McpClientManager;
use crate::tool::ToolRegistry;

/// Workspace template files embedded into the binary at build time.
/// Extracted to workspace on first run only — existing files are never overwritten.
const EMBEDDED_FILES: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/embedded_files.rs"));

/// Shared per-process log state: the workspace logs directory, file prefix, and
/// the currently open per-day file together with the day it belongs to.
/// Shared per-process log state: the workspace logs directory, file prefix, and
/// the currently open per-run file (one file per program launch,
/// `rustagent-YYYY-MM-DD.N.log`) plus a stable dated alias
/// (`rustagent-YYYY-MM-DD.log`) that always holds only the current run.
struct DailyLogShared {
    log_dir: std::path::PathBuf,
    prefix: String,
    state: std::sync::Mutex<DailyLogState>,
}
struct DailyLogState {
    day: String,
    file: Option<std::fs::File>,
    alias: Option<std::fs::File>,
    run: u32,
}
impl DailyLogShared {
    fn new(log_dir: std::path::PathBuf, prefix: String) -> Self {
        Self {
            log_dir,
            prefix,
            state: std::sync::Mutex::new(DailyLogState { day: String::new(), file: None, alias: None, run: 0 }),
        }
    }
    fn today() -> String {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    }
    /// Highest existing run index for the day, plus one (i.e. the next per-run file number).
    fn next_run_index(dir: &std::path::Path, prefix: &str, day: &str) -> u32 {
        let stem = format!("{}-{}.", prefix, day);
        let mut max: u32 = 0;
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if let Some(rest) = name.strip_prefix(&stem) {
                    if let Some(num) = rest.strip_suffix(".log") {
                        if let Ok(n) = num.parse::<u32>() {
                            if n > max { max = n; }
                        }
                    }
                }
            }
        }
        max + 1
    }
    fn run_path(&self, day: &str, n: u32) -> std::path::PathBuf {
        self.log_dir.join(format!("{}-{}.{}.log", self.prefix, day, n))
    }
    fn alias_path(&self, day: &str) -> std::path::PathBuf {
        self.log_dir.join(format!("{}-{}.log", self.prefix, day))
    }
    /// Path of the current run's log file (valid once the first log line is flushed).
    fn current_run_path(&self, today: &str) -> std::path::PathBuf {
        let run = self.state.lock()
            .map(|g| g.run)
            .unwrap_or_else(|e| e.into_inner().run);
        self.run_path(today, run)
    }

    /// Open a fresh per-run file (and its stable dated alias) for the day. Each
    /// call picks the next run index, so a re-launched process never appends to
    /// an earlier run's file. The stable alias is truncated so it holds only the
    /// current run (helpers that read the dated name still see this run).
    fn rotate(&self, st: &mut DailyLogState, today: &str) {
        if st.day == today && st.file.is_some() {
            return;
        }
        let n = Self::next_run_index(&self.log_dir, &self.prefix, today);
        let run_file = std::fs::OpenOptions::new()
            .create(true).write(true).truncate(true)
            .open(self.run_path(today, n)).ok();
        let alias_file = std::fs::OpenOptions::new()
            .create(true).write(true).truncate(true)
            .open(self.alias_path(today)).ok();
        st.day = today.to_string();
        st.file = run_file;
        st.alias = alias_file;
        st.run = n;
    }
}

/// A tracing writer that mirrors every formatted line to a log file.
/// Whether it also writes to stdout depends on the `show_on_console` flag:
/// - CLI mode: false (only agent TextDelta events are printed to stdout)
/// - Web mode: true (all system logs are echoed to console for debugging)
struct TeeLogWriter {
    shared: std::sync::Arc<DailyLogShared>,
    show_on_console: bool,
}
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for TeeLogWriter {
    type Writer = TeeLogSink;
    fn make_writer(&'a self) -> Self::Writer {
        TeeLogSink { 
            shared: self.shared.clone(), 
            show_on_console: self.show_on_console, 
        }
    }
}

struct TeeLogSink {
    shared: std::sync::Arc<DailyLogShared>,
    show_on_console: bool,
}
impl std::io::Write for TeeLogSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Recover from a poisoned lock instead of panicking.
        let mut st = match self.shared.state.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        // Only echo to stdout if show_on_console is enabled (web mode).
        // In CLI mode, agent text output comes from AgentEvent::TextDelta events,
        // not from tracing logs.
        if self.show_on_console {
            let _ = std::io::Write::write_all(&mut std::io::stdout(), buf);
        }
        let today = DailyLogShared::today();
        self.shared.rotate(&mut st, &today);
        if let Some(f) = st.file.as_mut() {
            let _ = std::io::Write::write_all(f, buf);
        }
        if let Some(a) = st.alias.as_mut() {
            let _ = std::io::Write::write_all(a, buf);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let mut st = match self.shared.state.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        if self.show_on_console {
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
        if let Some(f) = st.file.as_mut() {
            let _ = std::io::Write::flush(f);
        }
        if let Some(a) = st.alias.as_mut() {
            let _ = std::io::Write::flush(a);
        }
        Ok(())
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---- Parse CLI arguments ----
    use clap::Parser;
    let cli_args = cli::Cli::parse();
    let resolved = match cli_args.resolve() {
        Some(r) => r,
        None => {
            // No valid mode specified — print help
            use clap::CommandFactory;
            cli::Cli::command().print_help().ok();
            println!();
            return Ok(());
        }
    };

    // ---- Resolve exe + workspace dir ----
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Base workspace directory (cross-platform)
    let base_workspace = if let Ok(home) = std::env::var("HOME") {
        format!("{}/.RustAgent/workspace", home)
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        format!("{}\\.RustAgent\\workspace", userprofile)
    } else {
        exe_dir.join(".workspace").to_string_lossy().to_string()
    };

    // Resolve workspace:
    // --workspace override > --isolated profile > shared base workspace
    // By default, all profiles share the same base workspace.
    // Only with --isolated flag does a profile get its own directory.
    let workspace_dir = if let Some(ref ws) = resolved.workspace_override {
        ws.clone()
    } else if resolved.isolated {
        // Isolated mode: profile gets its own workspace
        if let Some(ref profile) = resolved.profile {
            format!("{}/profiles/{}", base_workspace, profile)
        } else {
            base_workspace
        }
    } else {
        // Shared mode: all profiles use the base workspace
        base_workspace
    };

    if let Err(e) = std::fs::create_dir_all(&workspace_dir) {
        tracing::warn!("Failed to create workspace directory {}: {}", workspace_dir, e);
    }

    // Initialize logging: mirror to log file AND optionally to console.
    // CLI mode: system logs (info/error/debug) only written to file, agent output
    // comes from AgentEvent::TextDelta events printed directly to stdout.
    // Web mode: all logs echoed to console for interactive debugging.
    let logs_dir = std::path::Path::new(&workspace_dir).join("logs");
    let _ = std::fs::create_dir_all(&logs_dir);
    let log_shared = std::sync::Arc::new(DailyLogShared::new(logs_dir.clone(), "rustagent".to_string()));
    let default_filter = resolved.log_level.clone().unwrap_or_else(|| "info".to_string());
    let show_on_console = matches!(resolved.mode, cli::RunMode::Web);
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&format!("{},chromiumoxide::handler=error", default_filter))),
        )
        .with_writer(TeeLogWriter { 
            shared: log_shared.clone(), 
            show_on_console, 
        })
        .with_ansi(false)
        .init();

    info!("Starting RustAgentX (pid {})", std::process::id());
    info!("Mode: {:?}", resolved.mode);
    if let Some(ref profile) = resolved.profile {
        info!("Profile: {}", profile);
    }
    info!("Executable directory: {}", exe_dir.display());
    info!("Workspace directory: {}", workspace_dir);
    info!(
        "Runtime log file: {}",
        log_shared.current_run_path(&DailyLogShared::today()).display()
    );
    let ws_subdirs = ["memory", "tools", "skills", "logs", "static", "output", "knowledge", "rules", "profiles"];
    for sub in &ws_subdirs {
        let p = std::path::Path::new(&workspace_dir).join(sub);
        let _ = std::fs::create_dir_all(&p);
    }

    // Load config: explicit --config path > workspace config.toml > default
    let mut config = if let Some(ref config_path) = resolved.config_path {
        // Load from explicit config file path
        let content = std::fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read config file '{}': {}", config_path, e))?;
        let cfg: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file '{}': {}", config_path, e))?;
        info!("Config loaded from explicit path: {}", config_path);
        cfg
    } else {
        let cfg = Config::load(&workspace_dir)?;
        info!("Config loaded from workspace");
        cfg
    };

    // Apply CLI overrides (CLI takes precedence over config file)
    config.apply_cli_overrides(&resolved.overrides);

    // Apply run mode override (--mode instant|expert)
    if let Some(ref run_mode) = resolved.run_mode {
        config.agent.mode = run_mode.clone();
    }
    info!("Run mode: {}", config.agent.mode);

    // ── Detect user's Given Name from Windows ──
    // Detect on every startup using whoami crate (no persistence to config).
    let user_given_name = crate::config::detect_user_given_name();
    info!("Detected user_given_name: {}", user_given_name);

    // Build tool registry (built-in tools)
    // The notification broadcast channel is created early so tools that need to
    // push messages to WebSocket clients (e.g. sys_remind) can hold a sender.
    let (notify_tx, _) = tokio::sync::broadcast::channel::<String>(100);

    // ── Random password (web mode only) ──────
    // A fresh random 6-digit password is generated for web dashboard access.
    // Headless/CLI mode does not require password authentication.
    let password = if resolved.mode == cli::RunMode::Web {
        let mut bytes = [0u8; 3];
        getrandom::fill(&mut bytes).expect("getrandom");
        let num = ((bytes[0] as u32) << 16 | (bytes[1] as u32) << 8 | bytes[2] as u32) % 1000000;
        let password = format!("{:06}", num);
        let pwd_file = std::path::Path::new(&workspace_dir).join(".password");
        if let Err(e) = std::fs::write(&pwd_file, &password) {
            tracing::warn!("Failed to save password: {}", e);
        }
        password
    } else {
        String::new() // Headless mode: no password needed
    };

    // ── Extract embedded workspace files (first-run only) ────
    // AGENTS.md, SOUL.md, TOOLS.md, USER.md are compiled into the binary.
    // On first run they are written to workspace; existing files are never overwritten.
    for &(name, content) in EMBEDDED_FILES {
        let path = std::path::Path::new(&workspace_dir).join(name);
        if !path.exists() {
            if let Err(e) = std::fs::write(&path, content) {
                tracing::warn!("Failed to extract {}: {}", name, e);
            } else {
                info!("Extracted {} to workspace", name);
            }
        }
    }

    // Migrate existing config files from exe_dir → workspace (first-run upgrade)
    let migrations = [
        ("models.json", "models.json"),
        ("cron_tasks.json", "cron_tasks.json"),
        ("mcp_servers.json", "mcp_servers.json"),
        ("memory.db", "memory/memory.db"),
    ];
    for (src_name, dst_rel) in &migrations {
        let src = exe_dir.join(src_name);
        let dst = std::path::Path::new(&workspace_dir).join(dst_rel);
        if src.exists() && !dst.exists() {
            if let Err(e) = std::fs::copy(&src, &dst) {
                tracing::warn!("Failed to migrate {} → {}: {}", src.display(), dst.display(), e);
            } else {
                info!("Migrated {} → {}", src.display(), dst.display());
            }
        }
    }
    // Migrate Tools/ → tools/ (case change for consistency)
    {
        let old_tools = exe_dir.join("Tools");
        let new_tools = std::path::Path::new(&workspace_dir).join("tools");
        if old_tools.exists() && !new_tools.exists() {
            let _ = std::fs::rename(&old_tools, &new_tools);
        }
    }
    // Migrate skills/ → workspace/skills/
    {
        let old_skills = exe_dir.join("skills");
        let new_skills = std::path::Path::new(&workspace_dir).join("skills");
        if old_skills.exists() && !new_skills.exists() {
            let _ = std::fs::rename(&old_skills, &new_skills);
        }
    }
    // Migrate logs/ → workspace/logs/
    {
        let old_logs = exe_dir.join("logs");
        let new_logs = std::path::Path::new(&workspace_dir).join("logs");
        if old_logs.exists() && !new_logs.exists() {
            let _ = std::fs::rename(&old_logs, &new_logs);
        }
    }
    // Migrate static/ → workspace/static/
    {
        let old_static = exe_dir.join("static");
        let new_static = std::path::Path::new(&workspace_dir).join("static");
        if old_static.exists() && !new_static.exists() {
            let _ = std::fs::rename(&old_static, &new_static);
        }
    }

    let working_dir = if resolved.mode == cli::RunMode::Headless {
        // Headless mode: use --workdir or CWD as the task working directory
        resolved.workdir.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        })
    } else if config.agent.working_dir == "." {
        workspace_dir.clone()
    } else {
        config.agent.working_dir.clone()
    };
    let mut registry = ToolRegistry::build_default(&working_dir, Some(notify_tx.clone()));
    info!("Built-in tools: {:?}", registry.tool_names());

    // Connect MCP servers (persist to workspace)
    let mcp_persist_path = std::path::Path::new(&workspace_dir).join("mcp_servers.json");
    let mut mcp_manager = McpClientManager::with_persist_path(mcp_persist_path);

    // Load persisted MCP server configs (from mcp_servers.json, auth tokens auto-decrypted)
    let persisted = mcp_manager.load_configs();
    if !persisted.is_empty() {
        info!("Loaded {} persisted MCP server(s)", persisted.len());
        mcp_manager.connect(&persisted).await;
    }

    // Register all MCP tools into the tool registry
    let mcp_tools = mcp_manager.get_tools();
    for tool in &mcp_tools {
        info!("MCP tool: {} ({})", tool.name(), tool.description());
        registry.register(tool.clone());
    }
    if !mcp_tools.is_empty() {
        info!("Registered {} MCP tool(s) total", mcp_tools.len());
    }

    // Load skills (resolve skills dir from workspace)
    let skills_dir = std::path::Path::new(&workspace_dir).join("skills");
    let skill_manager = Arc::new(SkillManager::new_with_notify(
        skills_dir.to_str().unwrap_or("skills"),
        Some(notify_tx.clone()),
    ));
    let skills = skill_manager.list();
    info!("Loaded {} skills", skills.len());

    // Add skill meta-tools
    let meta_tools = skill_manager.build_meta_tools();
    for mt in &meta_tools {
        registry.register(mt.clone());
    }

    // Build LLM provider (implements Llm trait)
    // Load persisted model configs (from models.json in workspace, api_keys auto-decrypted)
    let model_store_path = std::path::Path::new(&workspace_dir).join("models.json");
    let initial_models = model_store::load_configs(&model_store_path);
    if !initial_models.is_empty() {
        info!("Loaded {} model config(s) from models.json", initial_models.len());
    }
    let model_names: Vec<String> = initial_models.iter().map(|m| m.name.clone()).collect();
    let shared_models = Arc::new(tokio::sync::RwLock::new(initial_models));
    let provider = Arc::new(OpenAiProvider::new_with_shared(shared_models.clone()));
    let provider_for_state = provider.clone();
    info!("Models available: {:?}", model_names);

    // Build logger (resolve log dir from workspace)
    let log_dir = std::path::Path::new(&workspace_dir).join("logs");
    let logger = Arc::new(ConversationLogger::new(log_dir.to_str().unwrap_or("logs")));

    // Build memory store (resolve DB path from workspace/memory/)
    let db_path = std::path::Path::new(&workspace_dir).join("memory").join("memory.db");
    let memory_store = Arc::new(
        MemoryStore::new(db_path.to_str().unwrap_or("memory.db"))
            .expect("Failed to initialize memory store")
    );
    info!("Memory store ready: {}", db_path.display());

    // Clean up stale checkpoints (older than 24 hours) on startup
    let _ = memory_store.cleanup_stale_checkpoints(24);


    // Build task checkpointer for crash recovery (断点续跑)
    let checkpointer = Arc::new(TaskCheckpointer::new(memory_store.clone()));

    // Build external tools manager (resolve tools dir from workspace)
    let tools_dir = std::path::Path::new(&workspace_dir).join("tools");
    let external_tools = Arc::new(Mutex::new(ExternalToolsManager::new(tools_dir.clone())));
    info!("External tools dir: {}", tools_dir.display());

    // Register external tools into registry (LLM-visible at startup)
    {
        let mgr = external_tools.lock().await;
        let handles = mgr.get_tool_handles();
        if !handles.is_empty() {
            registry.sync_external_tools(&handles);
            info!("Registered {} external tool(s): {:?}",
                handles.len(),
                handles.iter().map(|(n, _, _, _)| n.as_str()).collect::<Vec<_>>()
            );
        }
    }

    // Wrap registry in Arc<RwLock> for dynamic MCP tool registration
    let shared_tools = Arc::new(tokio::sync::RwLock::new(registry));

    // Create browser session early so it can be shared between agent (cleanup) and tool (use)
    let browser_session = crate::tool::browser_cdp::BrowserSession::new(workspace_dir.clone());

    // Build agent using builder pattern (ADK-RUST style)
    let agent = LlmAgent::builder()
        .name("RustAgentX")
        .description("Cross-platform AI agent")
        .provider(provider)
        .tools(shared_tools.clone())
        .skill_manager(skill_manager.clone())
        .max_iterations(config.agent.max_iterations)
        .working_dir(&working_dir)
        .workspace_dir(&workspace_dir)
        .parallel_ir_tools(config.agent.parallel_ir_tools)
        .user_given_name(&user_given_name)
        .cleanup_session(browser_session.clone())
        .build()
        .map_err(|e| format!("Failed to build agent: {}", e))?;
    let agent: Arc<dyn agent::Agent> = Arc::new(agent);

    // Build runner using builder pattern (ADK-RUST style)
    let runner = Runner::builder()
        .agent(agent)
        .logger(logger.clone())
        .checkpointer(checkpointer)
        .app_name("RustAgentX")
        .build()
        .map_err(|e| format!("Failed to build runner: {}", e))?;
    let runner = Arc::new(runner);

    // Build permission state
    let (permission_resolver, permission_pending) = PermissionResolver::new();
    let permissions = Arc::new(Mutex::new(default_permissions()));
    // Clone for headless mode (before moving into AppState)
    let permissions_for_headless = permissions.clone();
    let permission_pending_for_headless = permission_pending.clone();

    // Build scheduler (resolve cron path from workspace)
    let cron_path = std::path::Path::new(&workspace_dir).join("cron_tasks.json");
    let scheduler = Arc::new(Mutex::new(Scheduler::new(
        cron_path.to_str().unwrap_or("cron_tasks.json"),
        runner.clone(),
        shared_models.clone(),
        permissions.clone(),
        permission_pending.clone(),
        config.agent.max_iterations,
        config.agent.rabbit_hole_threshold,
        128000,  // default context window for CRON tasks
        config.agent.context_window_threshold,
        config.agent.tool_timeout_secs as u64,
        notify_tx.clone(),
    )));

    // Spawn scheduler background loop
    let scheduler_loop = scheduler.clone();
    tokio::spawn(async move {
        Scheduler::run_loop(scheduler_loop).await;
    });

    // Spawn heartbeat background loop
    let heartbeat = Heartbeat::new(
        runner.clone(),
        shared_models.clone(),
        permissions.clone(),
        permission_pending.clone(),
        config.agent.max_iterations,
        config.agent.rabbit_hole_threshold,
        128000,
        config.agent.context_window_threshold,
        config.agent.tool_timeout_secs as u64,
        notify_tx.clone(),
        workspace_dir.clone(),
    );
    tokio::spawn(async move {
        heartbeat.run_loop().await;
    });
    info!("Heartbeat background loop spawned");

    // Register CRON management tool (needs scheduler, which depends on runner)
    // Register memory_md tool (file-based daily logs + long-term memory)
    // Register todo_update tool (lightweight task planning/tracking)
    // Register browser_cdp tool (uses same session as agent cleanup)
    {
        let mut reg = shared_tools.write().await;
        reg.register(Arc::new(crate::tool::cron_manage::CronManageTool::new(scheduler.clone())));
        reg.register(Arc::new(crate::tool::memory_md::MemoryMdTool::new(workspace_dir.clone())));
        reg.register(Arc::new(crate::tool::todo_update::TodoUpdateTool::new(workspace_dir.clone())));
        reg.register(Arc::new(crate::tool::browser_cdp::BrowserCdpTool::new(browser_session)));
    }
    info!("Registered cron_manage + memory_md + todo_update + browser_cdp tools");

    // Computer Use flag (kept for config compatibility, but tools are not available on Linux)
    let computer_use_enabled = Arc::new(std::sync::atomic::AtomicBool::new(false));
    
    // Human intervention simulation switch (default: false)
    let human_intervention_enabled = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Build app state (clone Arcs so the headless branch can also use them)
    let state = Arc::new(AppState {
        runner: runner.clone(),
        skill_manager: skill_manager.clone(),
        mcp_manager: Arc::new(Mutex::new(mcp_manager)),
        tools: shared_tools.clone(),
        logger,
        memory_store: memory_store.clone(),
        external_tools,
        password: password.clone(),
        model_configs: shared_models.clone(),
        model_store_path: model_store_path.to_str().unwrap_or("models.json").to_string(),
        max_iterations: Arc::new(AtomicUsize::new(config.agent.max_iterations)),
        rabbit_hole_threshold: Arc::new(AtomicUsize::new(config.agent.rabbit_hole_threshold)),
        context_window_threshold: Arc::new(AtomicUsize::new(config.agent.context_window_threshold)),
        tool_timeout_secs: Arc::new(AtomicUsize::new(config.agent.tool_timeout_secs)),
        max_tool_retries: Arc::new(AtomicUsize::new(config.agent.max_tool_retries)),
        expert_max_iterations: Arc::new(AtomicUsize::new(config.agent.expert_max_iterations)),
        expert_tool_timeout_secs: Arc::new(AtomicUsize::new(config.agent.expert_tool_timeout_secs)),
        expert_max_tool_retries: Arc::new(AtomicUsize::new(config.agent.expert_max_tool_retries)),
        expert_max_managed_rounds: Arc::new(AtomicUsize::new(config.agent.expert_max_managed_rounds)),
        sessions: Mutex::new(std::collections::HashMap::new()),
        permissions,
        permission_resolver,
        permission_pending,
        expert_tasks: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        scheduler,
        notify_tx,
        workspace_dir: workspace_dir.clone(),
        provider: provider_for_state.clone(),
        computer_use_enabled: computer_use_enabled.clone(),
        human_intervention_enabled: human_intervention_enabled.clone(),
        primary_model: Arc::new(std::sync::RwLock::new(config.agent.primary_model.clone())),
        fallback_model: Arc::new(std::sync::RwLock::new(config.agent.fallback_model.clone())),
        timezone_offset: Arc::new(std::sync::RwLock::new(config.agent.timezone_offset)),
    });

    // ── Mode branch ──
    match resolved.mode {
        cli::RunMode::Headless => {
            // Headless mode: execute task from positional argument
            let task = resolved.task.clone().unwrap_or_default();
            if task.is_empty() {
                eprintln!("Usage: RustAgentX [OPTIONS] <TASK>");
                eprintln!("Example: RustAgentX --profile headless \"运行测试套件并报告失败的测试\"");
                return Err("No task provided.".into());
            }

            info!("=== RustAgentX Headless Mode ===");
            info!("Task: {}", task);
            if let Some(ref profile) = resolved.profile {
                info!("Profile: {}", profile);
            }

            // Apply permission policy based on CLI flags
            if resolved.permission_policy == cli::PermissionPolicy::AutoApprove {
                // Auto-approve all permissions
                let mut perms = permissions_for_headless.lock().await;
                perms.insert("read".to_string(), true);
                perms.insert("write".to_string(), true);
                perms.insert("delete".to_string(), true);
                perms.insert("modify".to_string(), true);
                perms.insert("execute".to_string(), true);
            } else if resolved.permission_policy == cli::PermissionPolicy::ReadOnly {
                // Only allow read operations
                let mut perms = permissions_for_headless.lock().await;
                perms.insert("read".to_string(), true);
                perms.insert("write".to_string(), false);
                perms.insert("delete".to_string(), false);
                perms.insert("modify".to_string(), false);
                perms.insert("execute".to_string(), false);
            }

            // Build a session and run the agent
            let session_id = format!("headless-{}", uuid::Uuid::new_v4());
            let model_name = config.agent.primary_model.clone().unwrap_or_default();

            // Expert mode routes to ManagedRunner (Manager-Executor-Auditor loop);
            // instant mode uses the plain runner. This mirrors the Web mode dispatch
            // in server.rs so CLI expert output (Manager Plan / Rounds / Findings)
            // matches the Dashboard behavior.
            let event_stream = if config.agent.mode == "expert" {
                info!("Expert mode: dispatching to ManagedRunner");
                let managed_runner = managed::ManagedRunner::new(
                    runner.clone(),
                    provider_for_state.clone(),
                    model_name.clone(),
                    config.agent.expert_max_managed_rounds,
                    memory_store.clone(),
                    shared_tools.clone(),
                    working_dir.clone(),
                    workspace_dir.clone(),
                    config.agent.expert_max_iterations,
                    config.agent.rabbit_hole_threshold,
                    128000,  // default context window
                    config.agent.context_window_threshold,
                    config.agent.fallback_model.clone(),
                    config.agent.expert_tool_timeout_secs as u64,
                    config.agent.expert_max_tool_retries,
                    skill_manager.clone(),
                    computer_use_enabled.clone(),
                    human_intervention_enabled.clone(),
                );
                let task_cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
                managed_runner.run(
                    &task,
                    &session_id,
                    &model_name,
                    &task,  // scope = the task itself
                    permissions_for_headless.clone(),
                    permission_pending_for_headless.clone(),
                    task_cancel,
                    None,   // no handoff (fresh CLI run)
                    true,   // force new run
                ).await.map_err(|e| format!("Managed run failed: {}", e))?
            } else {
                runner.run(
                    &task,
                    &session_id,
                    &model_name,
                    config.agent.max_iterations,
                    vec![],  // no history
                    permissions_for_headless.clone(),
                    permission_pending_for_headless.clone(),
                    None,    // no preauth profile
                    config.agent.fallback_model.clone(),
                    config.agent.rabbit_hole_threshold,
                    128000,  // default context window
                    config.agent.context_window_threshold,
                    config.agent.tool_timeout_secs as u64,
                    config.agent.max_tool_retries,
                    vec![],  // no images
                    None,    // no checkpoint
                    None,    // no resume
                ).await.map_err(|e| format!("Agent run failed: {}", e))?
            };

            // Consume the event stream and print text output
            use futures::StreamExt;
            let mut stream = event_stream;
            let mut trajectory_file = resolved.trajectory_path.as_ref().and_then(|path| {
                std::fs::File::create(path).ok()
            });
            let mut exit_code: i32 = 0;
            // Track whether we need a newline separator before the next TextDelta.
            // After tool events (logged only), the agent's reply should start on a
            // fresh line instead of running together with prior output.
            let mut need_leading_newline = false;

            while let Some(event_result) = stream.next().await {
                match event_result {
                    Ok(event) => {
                        // Write trajectory JSONL if enabled
                        if let Some(ref mut traj_file) = trajectory_file {
                            use std::io::Write;
                            let json_line = serde_json::to_string(&event).unwrap_or_default();
                            let _ = writeln!(traj_file, "{}", json_line);
                        }

                        match &event {
                            agent::AgentEvent::TextDelta { content, .. } => {
                                // TextDelta carries the agent's answer AND Expert-mode
                                // managed outputs (Manager Plan / Executor / Verification /
                                // Auditing) — always shown on terminal.
                                // Ensure a blank line separates this text block from the
                                // previous tool activity (which was logged, not printed).
                                if need_leading_newline && !content.starts_with('\n') {
                                    println!();
                                }
                                need_leading_newline = false;
                                print!("{}", content);
                                use std::io::Write;
                                let _ = std::io::stdout().flush();
                            }
                            agent::AgentEvent::ToolCall { name, args, .. } => {
                                // Tool calls: log only (not shown on terminal).
                                // tracing writes to workspace/logs/ in CLI mode.
                                // Mark that the next TextDelta should start on a new line.
                                need_leading_newline = true;
                                if !args.is_null() && !args.as_object().map(|m| m.is_empty()).unwrap_or(true) {
                                    info!("[cli] Tool call: {} | args: {}", name, serde_json::to_string(args).unwrap_or_default());
                                } else {
                                    info!("[cli] Tool call: {}", name);
                                }
                            }
                            agent::AgentEvent::ToolResult { name, result, .. } => {
                                // Tool results: log only (not shown on terminal).
                                // Char-based truncation avoids panicking on multi-byte
                                // UTF-8 boundaries (e.g. Chinese text).
                                need_leading_newline = true;
                                let result_str = serde_json::to_string(result).unwrap_or_default();
                                let log_str = if result_str.chars().count() > 500 {
                                    let truncated: String = result_str.chars().take(500).collect();
                                    format!("{}... (truncated, {} chars total)", truncated, result_str.chars().count())
                                } else {
                                    result_str
                                };
                                info!("[cli] Tool result: {} | {}", name, log_str);
                            }
                            agent::AgentEvent::Progress { tool_name, message, elapsed_secs, .. } => {
                                // Progress: log only (not shown on terminal).
                                info!("[cli] Progress: [{}s] {} - {}", elapsed_secs, tool_name, message);
                            }
                            agent::AgentEvent::Error { message, .. } => {
                                eprintln!("\n❌ [ERROR] {}", message);
                                exit_code = 1;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        eprintln!("\n[ERROR] Stream error: {}", e);
                        exit_code = 1;
                        break;
                    }
                }
            }
            println!(); // Final newline
            info!("Headless task completed (exit_code={})", exit_code);
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
            Ok(())
        }
        cli::RunMode::Web => {
            // Web mode: start the Axum server with dashboard
            let app = server::create_router(state);
            let addr = format!("{}:{}", config.server.host, config.server.port);

            info!("=== RustAgentX is running ===");
            info!("Local:   http://localhost:{}", config.server.port);
            info!("Network: http://{}:{}", get_local_ip(), config.server.port);
            info!("Password: {}", password);

            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await?;

            Ok(())
        }
    }
}

fn get_local_ip() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "0.0.0.0".to_string())
}

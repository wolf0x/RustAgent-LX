# Cron/Heartbeat 功能检查报告

## 📊 当前状态

### Cron 调度器

**实现位置**：`src/scheduler.rs`

**功能**：
- ✅ 从 `cron_tasks.json` 加载定时任务
- ✅ 支持间隔语法（`every 5m`, `every 1h`）和 CRON 表达式
- ✅ 每 30 秒检查到期任务
- ✅ 通过 Agent 执行任务
- ✅ 结果通过 WebSocket 通知

**初始化位置**：`src/main.rs:562-580`

**问题**：
- ❌ **CLI 模式下也会启动后台任务**（第 577-580 行）
- ❌ CLI 模式是短期运行的，后台任务会在任务完成后被强制终止
- ❌ 没有根据运行模式条件性启动

### Heartbeat 系统

**实现位置**：`src/heartbeat.rs`

**功能**：
- ✅ 每 30 分钟读取 `HEARTBEAT.md`
- ✅ 通过 Agent 执行健康检查
- ✅ 只在发现问题时发送通知
- ✅ 可配置检查间隔

**初始化位置**：`src/main.rs:583-599`

**问题**：
- ❌ **CLI 模式下也会启动**（第 596-598 行）
- ❌ 同样的问题：CLI 模式下无意义

## 🔍 代码分析

### 当前初始化逻辑（main.rs:560-599）

```rust
// 无论 Web 还是 CLI 模式，都会执行
let scheduler = Arc::new(Mutex::new(Scheduler::new(...)));
tokio::spawn(async move {
    Scheduler::run_loop(scheduler_loop).await;
});

let heartbeat = Heartbeat::new(...);
tokio::spawn(async move {
    heartbeat.run_loop().await;
});
```

### 问题影响

**Web 模式**：
- ✅ 后台任务正常运行
- ✅ 定时任务按时执行
- ✅ 心跳检查定期进行

**CLI 模式**：
- ⚠️ 后台任务会启动
- ⚠️ 但 CLI 任务完成后立即退出
- ⚠️ 定时任务和心跳检查无法真正执行
- ⚠️ 浪费资源初始化不需要的组件

## 💡 建议修复

### 方案 1：条件性启动（推荐）

只在 Web 模式下启动后台任务：

```rust
// 在模式分支后初始化
match resolved.mode {
    cli::RunMode::Web => {
        // 初始化并启动后台任务
        let scheduler = Arc::new(Mutex::new(Scheduler::new(...)));
        tokio::spawn(async move {
            Scheduler::run_loop(scheduler).await;
        });
        
        let heartbeat = Heartbeat::new(...);
        tokio::spawn(async move {
            heartbeat.run_loop().await;
        });
        
        // ... Web 模式代码
    }
    cli::RunMode::Headless => {
        // CLI 模式：不启动后台任务
        // ... CLI 模式代码
    }
}
```

**优点**：
- ✅ 节省 CLI 模式资源
- ✅ 逻辑清晰
- ✅ 避免无意义的后台任务

**缺点**：
- ⚠️ 需要重构代码结构
- ⚠️ Scheduler 需要在 AppState 中可用（用于 cron_manage 工具）

### 方案 2：保持现状（当前）

**优点**：
- ✅ 代码简单
- ✅ 两种模式行为一致

**缺点**：
- ❌ CLI 模式下浪费资源
- ❌ 后台任务实际上无法执行

## 📝 配置文件说明

### cron_tasks.json

**位置**：`~/.RustAgent/workspace/cron_tasks.json`

**格式**：
```json
{
  "tasks": [
    {
      "id": "task-1",
      "name": "每小时检查",
      "schedule": "every 1h",
      "message": "检查系统状态",
      "enabled": true,
      "last_run": "2026-08-15T10:00:00Z",
      "next_run": "2026-08-15T11:00:00Z",
      "interval_secs": 3600
    }
  ]
}
```

**使用方式**：
- **Web 模式**：调度器自动执行
- **CLI 模式**：文件会被加载，但不会自动执行
- **管理**：通过 `cron_manage` 工具增删改查

### HEARTBEAT.md

**位置**：`~/.RustAgent/workspace/HEARTBEAT.md`

**格式**：Markdown 清单

**示例**：
```markdown
# 健康检查清单

- [ ] 检查磁盘空间是否超过 80%
- [ ] 检查是否有异常进程
- [ ] 检查关键服务是否运行
- [ ] 检查网络连接状态
```

**使用方式**：
- **Web 模式**：每 30 分钟自动检查
- **CLI 模式**：不会自动检查
- **自定义**：编辑文件内容即可

## ✅ 当前功能状态

| 功能 | Web 模式 | CLI 模式 | 说明 |
|------|---------|---------|------|
| Cron 调度器 | ✅ 正常 | ⚠️ 初始化但不执行 | CLI 任务完成后退出 |
| Heartbeat | ✅ 正常 | ⚠️ 初始化但不执行 | CLI 任务完成后退出 |
| cron_manage 工具 | ✅ 可用 | ✅ 可用 | 可以管理任务定义 |
| 定时任务执行 | ✅ 自动 | ❌ 不自动 | 需要 Web 模式 |
| 心跳检查 | ✅ 自动 | ❌ 不自动 | 需要 Web 模式 |

## 🎯 建议

### 短期（当前版本）

保持现状，因为：
1. CLI 模式下后台任务虽然启动但不会造成问题
2. 代码重构有风险
3. 用户主要关注功能是否可用

### 长期（下一版本）

实施条件性启动：
1. 重构初始化逻辑
2. 根据模式条件性启动后台任务
3. 优化资源使用

## 📚 相关文档

- `CONFIGURATION_GUIDE.md` - 配置指南
- `src/scheduler.rs` - 调度器实现
- `src/heartbeat.rs` - 心跳系统实现

---

**结论**：Cron/Heartbeat 功能实现完整，在 Web 模式下正常工作。CLI 模式下虽然会初始化，但由于任务完成后立即退出，实际上无法执行。当前版本可以接受，建议下一版本优化。

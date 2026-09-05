use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ai::deepseek::{DeepSeekClient, Message};
use crate::ai::tools::{ToolRegistry, ToolCall, ToolResult, ToolSchema, detect_runtimes};

/// ─── Agent Loop 配置 ───

#[derive(Debug, Clone)]
pub struct LoopConfig {
    /// 最大工具调用迭代次数
    pub max_iterations: usize,
    /// 是否在每步后注入"先读后改"提醒（Claude 模式）
    pub inject_read_before_edit_reminder: bool,
    /// 是否在每 N 步注入"目标完成度"提醒（GPT 模式）
    pub inject_progress_reminder_every: Option<usize>,
    /// 是否在开始前要求 Grep/Glob 概览（Gemini 模式）
    pub require_initial_scan: bool,
    /// 是否强制分解为子任务（Qwen 模式）
    pub require_task_decomposition: bool,
    /// 是否每步要求"先输出推理"（Kimi 模式）
    pub require_thinking_prefix: bool,
}

impl LoopConfig {
    /// 根据 Persona 决定 Loop 行为
    pub fn for_mode(mode: &str) -> Self {
        match mode {
            "deep-anth" => Self {
                max_iterations: 30,
                inject_read_before_edit_reminder: true,
                inject_progress_reminder_every: None,
                require_initial_scan: false,
                require_task_decomposition: false,
                require_thinking_prefix: false,
            },
            "deep-oai" => Self {
                max_iterations: 20,
                inject_read_before_edit_reminder: false,
                inject_progress_reminder_every: Some(5),
                require_initial_scan: false,
                require_task_decomposition: false,
                require_thinking_prefix: false,
            },
            "deep-gem" => Self {
                max_iterations: 15,
                inject_read_before_edit_reminder: false,
                inject_progress_reminder_every: None,
                require_initial_scan: true,
                require_task_decomposition: false,
                require_thinking_prefix: false,
            },
            "deep-qwen" => Self {
                max_iterations: 25,
                inject_read_before_edit_reminder: false,
                inject_progress_reminder_every: None,
                require_initial_scan: false,
                require_task_decomposition: true,
                require_thinking_prefix: false,
            },
            "deep-kimi" => Self {
                max_iterations: 30,
                inject_read_before_edit_reminder: false,
                inject_progress_reminder_every: None,
                require_initial_scan: false,
                require_task_decomposition: false,
                require_thinking_prefix: true,
            },
            _ => Self {
                max_iterations: 20,
                inject_read_before_edit_reminder: false,
                inject_progress_reminder_every: None,
                require_initial_scan: false,
                require_task_decomposition: false,
                require_thinking_prefix: false,
            },
        }
    }
}

/// ─── Agent 事件（用于向前端流式推送）───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEventKind {
    /// Agent 开始
    Started { mode: String, max_iterations: usize },
    /// 助手文本增量
    AssistantText { content: String },
    /// 工具调用请求（arguments 为 JSON 字符串，避免 Value 序列化问题）
    ToolCallRequested { id: String, name: String, arguments: String },
    /// 工具执行完成
    ToolCallExecuted { id: String, name: String, success: bool, output: String },
    /// 迭代计数
    Iteration { current: usize, max: usize },
    /// 循环结束
    Done { content: String, total_iterations: usize, total_tool_calls: usize },
    /// 错误
    Error { message: String },
    /// 文件系统变化（write/edit/delete 等成功后触发，前端应刷新文件树）
    FileChanged { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub kind: AgentEventKind,
    pub ts: u64,
}

impl AgentEvent {
    pub fn new(kind: AgentEventKind) -> Self {
        Self {
            kind,
            ts: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }
}

/// ─── Agent Loop 主入口 ───

pub struct AgentLoopInput {
    pub mode: String,
    pub user_message: String,
    pub history: Vec<Message>,
    pub context_paths: Vec<String>,
    pub working_dir: PathBuf,
    pub deepseek: std::sync::Arc<DeepSeekClient>,
    /// 命令层组装好的原生 System Prompt（模式基础提示 + 原装工作流源码 + 上下文文件）
    pub system_prompt: String,
}

pub struct AgentLoopOutput {
    pub final_content: String,
    pub total_iterations: usize,
    pub total_tool_calls: usize,
    pub events: Vec<AgentEvent>,
}

/// ─── 运行 Agent Loop ───

pub async fn run_agent_loop<F>(
    input: AgentLoopInput,
    mut on_event: F,
) -> Result<AgentLoopOutput, String>
where
    F: FnMut(AgentEvent) + Send,
{
    let config = LoopConfig::for_mode(&input.mode);
    let tools = ToolRegistry::new(input.working_dir.clone());
    let tool_schemas: Vec<ToolSchema> = ToolRegistry::schemas();

    // 记录执行前已存在的临时脚本，避免误删用户文件
    let existing_temp_files = snapshot_temp_py_files(&input.working_dir);

    let mut events: Vec<AgentEvent> = Vec::new();
    let emit = |ev: AgentEvent, sink: &mut Vec<AgentEvent>, cb: &mut dyn FnMut(AgentEvent)| {
        sink.push(ev.clone());
        cb(ev);
    };

    emit(
        AgentEvent::new(AgentEventKind::Started {
            mode: input.mode.clone(),
            max_iterations: config.max_iterations,
        }),
        &mut events,
        &mut on_event,
    );

    // 1. 使用命令层组装好的原生 System Prompt（模式基础提示 + 原装工作流源码节选 +
    //    上下文文件内容）
    let mut system_prompt = input.system_prompt.clone();

    // 注入模式特有的引擎循环规则
    system_prompt.push_str(&mode_loop_directives(&input.mode, &config));

    // 2. 初始化消息历史
    let mut messages: Vec<Message> = Vec::new();
    messages.push(Message {
        role: "user".into(),
        content: input.user_message.clone(),
        tool_calls: None,
        tool_call_id: None,
        name: None,
        r#type: "user".into(),
    });
    // 历史消息也加进去（如果 history 非空）
    for h in &input.history {
        let mut m = h.clone();
        // 兼容老格式：补全新字段
        if m.tool_calls.is_none() { m.tool_calls = None; }
        if m.tool_call_id.is_none() { m.tool_call_id = None; }
        if m.name.is_none() { m.name = None; }
        if m.r#type.is_empty() { m.r#type = m.role.clone(); }
        messages.push(m);
    }

    let mut final_content = String::new();
    let mut total_tool_calls = 0;

    // 3. 主循环
    for iter in 0..config.max_iterations {
        emit(
            AgentEvent::new(AgentEventKind::Iteration {
                current: iter + 1,
                max: config.max_iterations,
            }),
            &mut events,
            &mut on_event,
        );

        // Kimi 模式：每步注入"先思考"提醒
        if config.require_thinking_prefix && iter > 0 {
            messages.push(Message {
                role: "user".into(),
                content: "[Reminder] Before your next action, briefly explain your reasoning (1-2 sentences).".into(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
                r#type: "user".into(),
            });
        }

        // GPT 模式：每 N 步注入目标检查
        if let Some(every) = config.inject_progress_reminder_every {
            if iter > 0 && iter % every == 0 {
                messages.push(Message {
                    role: "user".into(),
                    content: format!(
                        "[Progress Check] You've completed {} iterations. Review:\n\
                         - What's the original goal?\n\
                         - What have you completed?\n\
                         - What's the next concrete step?\n\
                         If you've completed the goal, output the final answer and STOP.",
                        iter
                    ),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                    r#type: "user".into(),
                });
            }
        }

        // 调用 DeepSeek
        let resp = input.deepseek
            .chat_with_tools(&system_prompt, &messages, Some(&tool_schemas))
            .await;

        let response = match resp {
            Ok(r) => r,
            Err(e) => {
                cleanup_temp_py_files(&input.working_dir, &existing_temp_files);
                let ev = AgentEvent::new(AgentEventKind::Error { message: e.clone() });
                events.push(ev.clone());
                on_event(ev);
                return Err(e);
            }
        };

        let choice = match response.choices.first() {
            Some(c) => c,
            None => {
                cleanup_temp_py_files(&input.working_dir, &existing_temp_files);
                let ev = AgentEvent::new(AgentEventKind::Error {
                    message: "No choices in response".into(),
                });
                events.push(ev.clone());
                on_event(ev);
                return Err("No choices in response".into());
            }
        };

        let assistant_msg = &choice.message;
        final_content = assistant_msg.content.clone();

        // 推送助手文本
        if !assistant_msg.content.is_empty() {
            let ev = AgentEvent::new(AgentEventKind::AssistantText {
                content: assistant_msg.content.clone(),
            });
            events.push(ev.clone());
            on_event(ev);
        }

        // 检查是否需要工具调用
        let tool_calls = assistant_msg.tool_calls.clone().unwrap_or_default();

        // 把助手消息加入历史，确保 type 字段非空
        let mut assistant_msg = assistant_msg.clone();
        if assistant_msg.r#type.is_empty() { assistant_msg.r#type = assistant_msg.role.clone(); }
        messages.push(assistant_msg);
        if tool_calls.is_empty() {
            // 没有工具调用 = 任务完成
            cleanup_temp_py_files(&input.working_dir, &existing_temp_files);
            let ev = AgentEvent::new(AgentEventKind::Done {
                content: final_content.clone(),
                total_iterations: iter + 1,
                total_tool_calls,
            });
            events.push(ev.clone());
            on_event(ev);
            return Ok(AgentLoopOutput {
                final_content,
                total_iterations: iter + 1,
                total_tool_calls,
                events,
            });
        }

        // 执行工具调用
        for call in &tool_calls {
            total_tool_calls += 1;

            // 解析 arguments（可能是字符串）
            let call_with_parsed_args = normalize_tool_call(call);

            // 通知前端（arguments 转为 JSON 字符串，避免 Value 序列化问题）
            let ev = AgentEvent::new(AgentEventKind::ToolCallRequested {
                id: call_with_parsed_args.id.clone(),
                name: call_with_parsed_args.function.name.clone(),
                arguments: call_with_parsed_args.function.arguments.to_string(),
            });
            events.push(ev.clone());
            on_event(ev);

            // Claude 模式：Edit/Write 前必须 Read
            if config.inject_read_before_edit_reminder {
                let name = &call_with_parsed_args.function.name;
                if (name == "edit" || name == "write")
                    && !has_recent_read(&messages, &call_with_parsed_args.function.arguments, name == "write")
                {
                    let warning = format!(
                        "[System] You must call `read` on the target file BEFORE `{}`. \
                         Edit/Write without Read is a HARD VIOLATION of the read-before-edit protocol. \
                         Please read the file first.",
                        name
                    );
                    let warning_str = warning.clone();
                    messages.push(tool_result_message(
                        &call_with_parsed_args.id,
                        &call_with_parsed_args.function.name,
                        &ToolResult { success: false, output: warning, data: None },
                    ));
                    let ev = AgentEvent::new(AgentEventKind::ToolCallExecuted {
                        id: call_with_parsed_args.id.clone(),
                        name: call_with_parsed_args.function.name.clone(),
                        success: false,
                        output: warning_str,
                    });
                    events.push(ev.clone());
                    on_event(ev);
                    continue;
                }
            }

            // 真正执行
            let result = tools.execute(&call_with_parsed_args).await;

            let ev = AgentEvent::new(AgentEventKind::ToolCallExecuted {
                id: call_with_parsed_args.id.clone(),
                name: call_with_parsed_args.function.name.clone(),
                success: result.success,
                output: truncate_for_display(&result.output, 2000),
            });
            events.push(ev.clone());
            on_event(ev);

            // 写入类工具成功后通知前端刷新文件树
            // （write/edit 必触发；bash/delete 在 result.success 时也触发，
            //  因为脚本可能生成文件，delete 必然改了文件树）
            if result.success {
                let name = call_with_parsed_args.function.name.as_str();
                if matches!(name, "write" | "edit" | "delete" | "bash") {
                    let ev = AgentEvent::new(AgentEventKind::FileChanged {
                        reason: name.to_string(),
                    });
                    events.push(ev.clone());
                    on_event(ev);
                }
            }

            // 把工具结果加入消息
            messages.push(tool_result_message(
                &call_with_parsed_args.id,
                &call_with_parsed_args.function.name,
                &result,
            ));
        }
    }

    // 达到最大迭代
    cleanup_temp_py_files(&input.working_dir, &existing_temp_files);
    let ev = AgentEvent::new(AgentEventKind::Done {
        content: final_content.clone(),
        total_iterations: config.max_iterations,
        total_tool_calls,
    });
    events.push(ev.clone());
    on_event(ev);

    Ok(AgentLoopOutput {
        final_content,
        total_iterations: config.max_iterations,
        total_tool_calls,
        events,
    })
}

// ════════════════════════════════════════════════════════
// 辅助函数
// ════════════════════════════════════════════════════════

fn mode_loop_directives(mode: &str, cfg: &LoopConfig) -> String {
    let mut s = String::new();
    s.push_str("\n\n## Agent Loop Directives (per engine)\n");

    // 通用规则（所有引擎适用）
    s.push_str("### Universal rules (apply to all engines)\n");
    s.push_str("- **Chunk long writes**: Any single `write` call's `content` MUST be <= 2000 characters. For longer files:\n");
    s.push_str("  1. First call `write` with the first chunk (creates the file).\n");
    s.push_str("  2. Then call `edit` with `old_string` set to the LAST ~10 lines of the previous chunk (must match exactly) and `new_string` set to those same lines + the next chunk appended.\n");
    s.push_str("  3. Repeat `edit` until the file is complete.\n");
    s.push_str("- **Chunk long bash commands**: If a `bash` command string is long, split it across multiple bash calls.\n");
    s.push_str("- **Tool result truncation**: If a tool returns a long output, you can use `read` with `offset`/`limit` or `grep` to inspect specific parts instead of dumping the whole thing again.\n");
    s.push_str("\n");

    // 注入可用运行时信息
    s.push_str("### Runtime environment\n");
    s.push_str(&detect_runtimes());
    s.push_str("- Use `check_runtime` tool to verify a specific runtime before executing code in that language.\n");
    s.push_str("- If a runtime is missing from the list above, DO NOT attempt to `bash` commands that require it (e.g. `node`, `java`, `gcc`, `go`, `cargo`, `dotnet`, `php`).\n");
    s.push_str("- Instead, ask the user to install the missing runtime, or fall back to an available language.\n");
    s.push_str("- Python is ALWAYS available (bundled with the app at `python/python.exe` relative to working dir). Use `python/python.exe` or just `python` in bash commands.\n");
    s.push_str("- Use Python (bundled) for data analysis, PDF/Excel processing, and quick scripts.\n");
    s.push_str("- For installing extra Python packages: use `python -m pip install <pkg>` in bash. Pre-installed: pymupdf (fitz).\n");
    s.push_str("\n");

    match mode {
        "deep-anth" => {
            s.push_str("You are running the **DeepAnth** mode (Claude Code original workflow source):\n");
            s.push_str("- HARD PROTOCOL: Every `edit` / `write` MUST be preceded by `read` of the same file in this session.\n");
            s.push_str("- For multi-step tasks, FIRST call `todo_write` to plan.\n");
            s.push_str("- For any build/test/run, use `bash`. Do NOT use `read` to guess command output.\n");
            s.push_str("- After completion, use `grep` to verify no leftover debug code.\n");
            s.push_str("- If a tool call fails twice, switch approach and explain why.\n");
            s.push_str("- **CRITICAL: Do NOT create temporary Python helper scripts** (e.g. _read_pdf.py, _check_data.py, _explore_data.py). The agent loop cleans up `_*.py` files at the end, but you lose tooling between calls. Instead use the dedicated tools:\n");
            s.push_str("  - For Python package checks: `check_python_package` (package name like 'pandas', 'fitz')\n");
            s.push_str("  - For installing packages: `install_python_package` (e.g. 'pandas', 'pymupdf', 'scikit-learn')\n");
            s.push_str("  - For reading PDFs: `read_pdf` (file_path)\n");
            s.push_str("  - For reading Excel: `read_excel` (file_path)\n");
            s.push_str("  - Use `bash` ONLY for: running tests, building, git commands, custom Python scripts you intend to keep.\n");
            s.push_str("- **CRITICAL: Do NOT use bash to test `import <package>`** — that creates dozens of red error rows. Use `check_python_package` instead.\n");
        }
        "deep-oai" => {
            s.push_str("You are running the **DeepOAI** mode (Codex CLI original workflow source):\n");
            s.push_str("- Plan → Generate → Review → Refine.\n");
            s.push_str("- Before generating code, briefly state your plan in the response.\n");
            s.push_str("- Every 5 iterations you'll be asked to check progress against the goal.\n");
            s.push_str("- Prefer minimal, runnable iterations. Verify after each step.\n");
        }
        "deep-gem" => {
            s.push_str("You are running the **DeepGem** mode (Gemini CLI original workflow source):\n");
            s.push_str("- For unfamiliar codebases, FIRST call `glob` + `grep` to build a mental map.\n");
            s.push_str("- Process large code in chunks: `read` with offset/limit, then synthesize.\n");
            s.push_str("- Use `grep` to find usages before editing shared functions.\n");
            s.push_str("- Be concise in explanations. Show code first, reasoning only when asked.\n");
        }
        "deep-qwen" => {
            s.push_str("You are running the **DeepQwen** mode (Qwen Code original workflow source):\n");
            s.push_str("- For any non-trivial task, FIRST call `todo_write` to break it into subtasks.\n");
            s.push_str("- Mark subtasks `in_progress` before starting, `completed` after finishing.\n");
            s.push_str("- Mirror the existing project style: read similar files first, then write consistent code.\n");
            s.push_str("- Prefer complete, runnable code blocks over partial snippets.\n");
        }
        "deep-kimi" => {
            s.push_str("You are running the **DeepKimi** mode (Kimi CLI original workflow source):\n");
            s.push_str("- Before each action, briefly state your reasoning (1-2 sentences).\n");
            s.push_str("- For complex tasks: Plan → Step-by-step execution → Verify each step.\n");
            s.push_str("- Use long context fully: don't truncate, read large files in chunks if needed.\n");
            s.push_str("- Always handle edge cases explicitly in the code you write.\n");
        }
        _ => {}
    }

    if cfg.max_iterations > 0 {
        s.push_str(&format!("\nMax iterations for this run: {}.\n", cfg.max_iterations));
    }
    s
}

/// 规范化路径：统一反斜杠为正斜杠，并 lowercase
fn normalize_path(p: &str) -> String {
    p.replace('\\', "/").to_lowercase()
}

/// 检查最近是否对目标文件调用过 read（Claude 模式强制）
fn has_recent_read(messages: &[Message], call_args: &Value, is_write: bool) -> bool {
    // Write 到新文件不需要先 Read
    if is_write {
        if let Some(path) = call_args.get("file_path").and_then(|v| v.as_str()) {
            if !std::path::Path::new(path).exists() {
                return true;
            }
        }
    }
    let target_path = normalize_path(
        call_args.get("file_path").and_then(|v| v.as_str()).unwrap_or("")
    );
    if target_path.is_empty() { return false; }
    // 在最近 20 条消息中查找 read 调用且 file_path 相同（路径规范化后比较）
    for m in messages.iter().rev().take(20) {
        if m.role != "tool" { continue; }
        // tool 消息的 content 包含 "File: <path>"
        if let Some(idx) = m.content.find("File: ") {
            let after = &m.content[idx + 6..];
            let path_str = after.lines().next().unwrap_or("").trim();
            if normalize_path(path_str) == target_path {
                return true;
            }
        }
    }
    false
}

fn normalize_tool_call(call: &ToolCall) -> ToolCall {
    // DeepSeek 返回的 arguments 是字符串，需要解析
    if call.function.arguments.is_string() {
        let s = call.function.arguments.as_str().unwrap_or("{}");
        match serde_json::from_str::<Value>(s) {
            Ok(v) => ToolCall {
                id: call.id.clone(),
                kind: call.kind.clone(),
                function: crate::ai::tools::ToolFunction {
                    name: call.function.name.clone(),
                    arguments: v,
                },
            },
            Err(_) => call.clone(),
        }
    } else {
        call.clone()
    }
}

fn tool_result_message(id: &str, name: &str, result: &ToolResult) -> Message {
    let content = if result.success {
        result.output.clone()
    } else {
        format!("[ERROR] {}", result.output)
    };
    Message {
        role: "tool".into(),
        content,
        tool_calls: None,
        tool_call_id: Some(id.to_string()),
        name: Some(name.to_string()),
        r#type: "tool".into(),
    }
}

fn truncate_for_display(s: &str, max: usize) -> String {
    if s.len() <= max { return s.to_string(); }
    let cut = (max as f64 * 0.8) as usize;
    let safe_cut = s.char_indices()
        .find(|(i, _)| *i >= cut)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    format!("{}... [truncated, {} total chars]", &s[..safe_cut], s.len())
}

/// 扫描工作区中已存在的下划线开头 Python 临时脚本
fn snapshot_temp_py_files(dir: &Path) -> HashSet<PathBuf> {
    let mut set = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('_') && name.ends_with(".py") {
                        set.insert(path);
                    }
                }
            }
        }
    }
    set
}

/// 删除本次 Agent 运行期间新增的下划线开头 Python 临时脚本
fn cleanup_temp_py_files(dir: &Path, existing: &HashSet<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && !existing.contains(&path) {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('_') && name.ends_with(".py") {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

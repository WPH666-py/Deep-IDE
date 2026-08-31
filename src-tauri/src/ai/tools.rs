use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW_FLAG: u32 = 0x08000000;

/// 在 Windows 上隐藏子进程的命令行窗口
fn hide_window(cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW_FLAG);
    }
}

use regex::Regex;
use glob::glob as glob_match;

/// ─── 工具执行结果 ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    /// 可选：附带结构化数据（如 TodoWrite 更新后的列表）
    pub data: Option<Value>,
}

/// ─── 工具调用请求（OpenAI / DeepSeek 兼容格式）───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type", default = "default_tool_call_kind")]
    pub kind: String,
    #[serde(default)]
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolFunction {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: Value,
}

fn default_tool_call_kind() -> String {
    "function".to_string()
}

/// ─── 工具 schema（发给 DeepSeek 的工具定义）───

#[derive(Debug, Clone, Serialize)]
pub struct ToolSchema {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub function: ToolFunctionSchema,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolFunctionSchema {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

/// ─── 工具注册表 ───

pub struct ToolRegistry {
    pub working_dir: PathBuf,
}

impl ToolRegistry {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    /// 获取所有工具的 schema（发给模型）
    pub fn schemas() -> Vec<ToolSchema> {
        vec![
            Self::read_schema(),
            Self::edit_schema(),
            Self::write_schema(),
            Self::bash_schema(),
            Self::grep_schema(),
            Self::glob_schema(),
            Self::web_search_schema(),
            Self::todo_write_schema(),
            Self::task_schema(),
            Self::install_python_package_schema(),
            Self::check_python_package_schema(),
            Self::read_pdf_schema(),
            Self::read_excel_schema(),
            Self::check_runtime_schema(),
        ]
    }

    /// 获取所有工具的 schema（按允许列表过滤）
    pub fn schemas_for(allowed: &[String]) -> Vec<ToolSchema> {
        Self::schemas()
            .into_iter()
            .filter(|s| allowed.iter().any(|a| a == s.function.name))
            .collect()
    }

    /// 执行一个工具调用
    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
        match call.function.name.as_str() {
            "read" => self.tool_read(&call.function.arguments),
            "edit" => self.tool_edit(&call.function.arguments),
            "write" => self.tool_write(&call.function.arguments),
            "bash" => self.tool_bash(&call.function.arguments).await,
            "grep" => self.tool_grep(&call.function.arguments),
            "glob" => self.tool_glob(&call.function.arguments),
            "web_search" => self.tool_web_search(&call.function.arguments).await,
            "todo_write" => self.tool_todo_write(&call.function.arguments),
            "task" => self.tool_task(&call.function.arguments),
            "install_python_package" => self.tool_install_python_package(&call.function.arguments).await,
            "check_python_package" => self.tool_check_python_package(&call.function.arguments).await,
            "read_pdf" => self.tool_read_pdf(&call.function.arguments),
            "read_excel" => self.tool_read_excel(&call.function.arguments),
            "check_runtime" => self.tool_check_runtime(&call.function.arguments),
            other => ToolResult {
                success: false,
                output: format!("Unknown tool: {}", other),
                data: None,
            },
        }
    }

    // ─── 路径安全检查 ───
    fn resolve_path(&self, p: &str) -> PathBuf {
        let path = Path::new(p);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 1: Read —— 读取文件
    // ════════════════════════════════════════════════════════
    fn tool_read(&self, args: &Value) -> ToolResult {
        let file_path = args.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
        let offset = args.get("offset").and_then(|v| v.as_u64()).map(|n| n as usize);
        let limit = args.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);

        if file_path.is_empty() {
            return ToolResult { success: false, output: "file_path is required".into(), data: None };
        }

        let full = self.resolve_path(file_path);
        // 先尝试 UTF-8 读取（去掉 BOM），失败时退回字节+UTF-8 lossy
        let content = match std::fs::read(&full) {
            Ok(bytes) => {
                let stripped = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                    &bytes[3..]
                } else {
                    &bytes[..]
                };
                String::from_utf8_lossy(stripped).into_owned()
            }
            Err(e) => {
                return ToolResult {
                    success: false,
                    output: format!("Failed to read {}: {}", file_path, e),
                    data: None,
                };
            }
        };
        let total_lines = content.lines().count();
        let start = offset.unwrap_or(0);
        let end = limit.map(|l| (start + l).min(total_lines)).unwrap_or(total_lines);
        let slice: String = content
            .lines()
            .skip(start)
            .take(end.saturating_sub(start))
            .enumerate()
            .map(|(i, line)| format!("{:>6}\t{}", start + i + 1, line))
            .collect::<Vec<_>>()
            .join("\n");
        ToolResult {
            success: true,
            output: format!("File: {}\nLines {}-{} of {}\n\n{}", file_path, start + 1, end, total_lines, slice),
            data: Some(json!({"total_lines": total_lines, "shown_start": start, "shown_end": end})),
        }
    }

    fn read_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "read",
                description: "Read a file from the workspace. Returns file contents with line numbers. Use offset and limit for large files.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Absolute path or workspace-relative path" },
                        "offset": { "type": "integer", "description": "Line number to start from (0-based)" },
                        "limit": { "type": "integer", "description": "Number of lines to read" }
                    },
                    "required": ["file_path"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 2: Edit —— 精确字符串替换
    // ════════════════════════════════════════════════════════
    fn tool_edit(&self, args: &Value) -> ToolResult {
        let file_path = args.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
        let old_string = args.get("old_string").and_then(|v| v.as_str()).unwrap_or("");
        let new_string = args.get("new_string").and_then(|v| v.as_str()).unwrap_or("");
        let replace_all = args.get("replace_all").and_then(|v| v.as_bool()).unwrap_or(false);

        if file_path.is_empty() || old_string.is_empty() {
            return ToolResult { success: false, output: "file_path and old_string are required".into(), data: None };
        }

        let full = self.resolve_path(file_path);
        let content = match std::fs::read_to_string(&full) {
            Ok(c) => c,
            Err(e) => return ToolResult { success: false, output: format!("Failed to read {}: {}", file_path, e), data: None },
        };

        if !content.contains(old_string) {
            return ToolResult {
                success: false,
                output: format!("old_string not found in {}. Read the file first to get exact content.", file_path),
                data: None,
            };
        }

        let occurrences = content.matches(old_string).count();
        if !replace_all && occurrences > 1 {
            return ToolResult {
                success: false,
                output: format!("old_string appears {} times in {}. Set replace_all=true or provide more context.", occurrences, file_path),
                data: None,
            };
        }

        let new_content = if replace_all {
            content.replace(old_string, new_string)
        } else {
            content.replacen(old_string, new_string, 1)
        };

        match std::fs::write(&full, &new_content) {
            Ok(_) => ToolResult {
                success: true,
                output: format!("Replaced {} occurrence(s) in {}", occurrences.min(1), file_path),
                data: Some(json!({"replacements": if replace_all { occurrences } else { 1 } })),
            },
            Err(e) => ToolResult { success: false, output: format!("Failed to write {}: {}", file_path, e), data: None },
        }
    }

    fn edit_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "edit",
                description: "Edit a file by replacing an exact string. MUST call read first to get exact content. old_string must match exactly (including whitespace).",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string" },
                        "old_string": { "type": "string", "description": "Exact string to replace (must match exactly)" },
                        "new_string": { "type": "string" },
                        "replace_all": { "type": "boolean", "description": "Replace all occurrences (default false)" }
                    },
                    "required": ["file_path", "old_string", "new_string"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 3: Write —— 创建或覆盖文件
    // ════════════════════════════════════════════════════════
    fn tool_write(&self, args: &Value) -> ToolResult {
        let file_path = args.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");

        if file_path.is_empty() {
            return ToolResult { success: false, output: "file_path is required".into(), data: None };
        }

        let full = self.resolve_path(file_path);
        if let Some(parent) = full.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(&full, content) {
            Ok(_) => ToolResult {
                success: true,
                output: format!("Wrote {} bytes to {}", content.len(), file_path),
                data: Some(json!({"bytes_written": content.len()})),
            },
            Err(e) => ToolResult { success: false, output: format!("Failed to write {}: {}", file_path, e), data: None },
        }
    }

    fn write_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "write",
                description: concat!(
                    "Create or overwrite a file. Use for new files or full rewrites. For existing files, prefer edit. ",
                    "IMPORTANT: For files longer than ~2000 characters of content, you MUST split into multiple write/edit calls. ",
                    "Write the FIRST chunk with `write` (creates the file), then append subsequent chunks with `edit` (matching the last few lines of the previous chunk as `old_string`). ",
                    "Each `content` argument should be <= 2000 characters to avoid response truncation."
                ),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string" },
                        "content": { "type": "string", "description": "File content. MUST be <= 2000 chars. For longer files, use multiple write/edit calls." }
                    },
                    "required": ["file_path", "content"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 4: Bash —— 执行 shell 命令
    // ════════════════════════════════════════════════════════
    async fn tool_bash(&self, args: &Value) -> ToolResult {
        let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
        let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(30000);

        if command.is_empty() {
            return ToolResult { success: false, output: "command is required".into(), data: None };
        }

        // 安全检查：危险命令需要确认（这里只提示，不阻止）
        let dangerous_patterns = ["rm -rf /", "del /f /s /q C:", "format ", ":(){:|:&};:"];
        for pat in &dangerous_patterns {
            if command.contains(pat) {
                return ToolResult {
                    success: false,
                    output: format!("BLOCKED: dangerous command pattern detected ('{}'). User confirmation required.", pat),
                    data: None,
                };
            }
        }

        let result = run_cmd_with_timeout(command, &self.working_dir, timeout_ms).await;
        let exit = result.exit_code;
        let combined = if result.stderr.is_empty() {
            result.stdout.clone()
        } else {
            format!("{}\n[stderr]\n{}", result.stdout, result.stderr)
        };
        ToolResult {
            success: result.success,
            output: format!("[exit {}]\n{}", exit, combined),
            data: Some(json!({"exit_code": exit})),
        }
    }

    fn bash_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "bash",
                description: concat!(
                    "Execute a shell command in the workspace directory. ",
                    "Returns stdout + stderr. Use for: running tests, building, git commands, etc. ",
                    "IMPORTANT: Commands already run in the workspace root, so do NOT use 'cd' to switch to the project directory. ",
                    "On Windows, commands run via cmd /C. Use forward slashes or single backslashes for paths (e.g. C:/Users/admin/project or C:\\Users\\admin\\project). ",
                    "Avoid bash-only syntax on Windows."
                ),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string" },
                        "timeout_ms": { "type": "integer", "description": "Timeout in milliseconds (default 30000)" }
                    },
                    "required": ["command"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 5: Grep —— 在文件中搜索正则
    // ════════════════════════════════════════════════════════
    fn tool_grep(&self, args: &Value) -> ToolResult {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let include_glob = args.get("include_glob").and_then(|v| v.as_str());

        if pattern.is_empty() {
            return ToolResult { success: false, output: "pattern is required".into(), data: None };
        }

        let regex = match Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => return ToolResult { success: false, output: format!("Invalid regex: {}", e), data: None },
        };

        let base = self.resolve_path(path);
        let mut hits: Vec<String> = Vec::new();
        let mut hit_count = 0;

        let entries: Vec<PathBuf> = if base.is_file() {
            vec![base.clone()]
        } else {
            walk_files(&base, 10)
        };

        for entry in entries {
            let file_path_str = entry.to_string_lossy().to_string();
            if let Some(glob_pat) = include_glob {
                if !glob_match_simple(glob_pat, &file_path_str) {
                    continue;
                }
            }
            // 跳过二进制文件和大文件
            if let Ok(meta) = std::fs::metadata(&entry) {
                if meta.len() > 2_000_000 { continue; }
            }
            if let Ok(content) = std::fs::read_to_string(&entry) {
                for (i, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        hits.push(format!("{}:{}: {}", file_path_str, i + 1, line));
                        hit_count += 1;
                        if hit_count >= 200 { break; }
                    }
                }
                if hit_count >= 200 { break; }
            }
        }

        if hits.is_empty() {
            ToolResult {
                success: true,
                output: format!("No matches for pattern '{}' in {}", pattern, path),
                data: Some(json!({"hit_count": 0})),
            }
        } else {
            let truncated = hit_count >= 200;
            let output = hits.join("\n");
            let suffix = if truncated { "\n[truncated at 200 hits]" } else { "" };
            ToolResult {
                success: true,
                output: format!("Found {} match(es):\n{}{}", hit_count, output, suffix),
                data: Some(json!({"hit_count": hit_count, "truncated": truncated})),
            }
        }
    }

    fn grep_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "grep",
                description: "Search for a regex pattern in files. Returns matching lines with file:line:content format. Use include_glob to filter (e.g. '*.ts').",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Regex pattern to search" },
                        "path": { "type": "string", "description": "Directory or file to search in (default: workspace root)" },
                        "include_glob": { "type": "string", "description": "File pattern filter, e.g. '*.ts', '*.py'" }
                    },
                    "required": ["pattern"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 6: Glob —— 文件名匹配
    // ════════════════════════════════════════════════════════
    fn tool_glob(&self, args: &Value) -> ToolResult {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

        if pattern.is_empty() {
            return ToolResult { success: false, output: "pattern is required".into(), data: None };
        }

        let base = self.resolve_path(path);
        let full_pattern = base.join(pattern);
        let pattern_str = full_pattern.to_string_lossy().to_string();

        let mut hits: Vec<String> = Vec::new();
        if let Ok(paths) = glob_match(&pattern_str) {
            for entry in paths {
                if let Ok(p) = entry {
                    hits.push(p.to_string_lossy().to_string());
                    if hits.len() >= 200 { break; }
                }
            }
        }

        if hits.is_empty() {
            ToolResult {
                success: true,
                output: format!("No files matched '{}'", pattern),
                data: Some(json!({"file_count": 0})),
            }
        } else {
            ToolResult {
                success: true,
                output: format!("Found {} file(s):\n{}", hits.len(), hits.join("\n")),
                data: Some(json!({"file_count": hits.len(), "files": hits})),
            }
        }
    }

    fn glob_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "glob",
                description: "Find files by name pattern. Supports wildcards: '*.ts', '**/*.test.ts', 'src/**/*.vue'",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string" },
                        "path": { "type": "string", "description": "Base directory (default: workspace root)" }
                    },
                    "required": ["pattern"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 7: WebSearch —— 联网搜索（占位 + 可扩展）
    // ════════════════════════════════════════════════════════
    async fn tool_web_search(&self, args: &Value) -> ToolResult {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        if query.is_empty() {
            return ToolResult { success: false, output: "query is required".into(), data: None };
        }
        // 占位实现：返回明确信息，让模型知道需要联网
        ToolResult {
            success: true,
            output: format!("[WebSearch] Query: '{}'\nNote: Web search not yet connected. Use existing knowledge or read local docs.", query),
            data: Some(json!({"query": query, "connected": false})),
        }
    }

    fn web_search_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "web_search",
                description: "Search the web for up-to-date information. Note: implementation pending — returns placeholder until connected.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 8: TodoWrite —— 任务列表管理
    // ════════════════════════════════════════════════════════
    fn tool_todo_write(&self, args: &Value) -> ToolResult {
        let items_val = args.get("items").and_then(|v| v.as_array());
        let items: Vec<TodoItem> = match items_val {
            Some(arr) => arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect(),
            None => return ToolResult { success: false, output: "items is required".into(), data: None },
        };

        let rendered = items.iter().map(|it| {
            let mark = match it.status.as_str() {
                "completed" => "[x]",
                "in_progress" => "[>]",
                _ => "[ ]",
            };
            format!("{} {} {}", mark, it.content, it.active_form.as_deref().unwrap_or(""))
        }).collect::<Vec<_>>().join("\n");

        ToolResult {
            success: true,
            output: format!("Todo list updated ({} items):\n{}", items.len(), rendered),
            data: Some(json!({"items": items})),
        }
    }

    fn todo_write_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "todo_write",
                description: "Update the task list. Use this to plan multi-step work and track progress.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "items": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "status": { "type": "string", "enum": ["pending", "in_progress", "completed"] },
                                    "content": { "type": "string", "description": "What needs to be done" },
                                    "active_form": { "type": "string", "description": "Present-tense action form" }
                                },
                                "required": ["status", "content"]
                            }
                        }
                    },
                    "required": ["items"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 9: Task —— 子代理委派
    // ════════════════════════════════════════════════════════
    fn tool_task(&self, _args: &Value) -> ToolResult {
        // 占位：实际委派在 agent_loop 中实现
        ToolResult {
            success: true,
            output: "[Task] Sub-agent delegation noted. Will be executed by orchestrator in next iteration.".into(),
            data: None,
        }
    }

    fn task_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "task",
                description: "Delegate a subtask to a specialized sub-agent (code-explorer / code-architect / code-reviewer).",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "agent": { "type": "string", "enum": ["code-explorer", "code-architect", "code-reviewer"] },
                        "prompt": { "type": "string" }
                    },
                    "required": ["agent", "prompt"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 10: InstallPythonPackage —— 安装 Python 包
    // ════════════════════════════════════════════════════════
    async fn tool_install_python_package(&self, args: &Value) -> ToolResult {
        let package = args.get("package").and_then(|v| v.as_str()).unwrap_or("");
        if package.is_empty() {
            return ToolResult { success: false, output: "package is required".into(), data: None };
        }
        // 安全校验：按空格切分后逐个校验每个包名（支持一次装多个包）
        // 允许的字符：字母数字、- _ . [ ] = > < , ; （用于 extras、版本约束等）
        let is_valid_char = |c: char| -> bool {
            c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
                || c == '[' || c == ']' || c == '=' || c == '>' || c == '<' || c == ',' || c == ';'
        };
        for token in package.split_whitespace() {
            if !token.chars().all(is_valid_char) {
                return ToolResult {
                    success: false,
                    output: format!("Invalid package name: '{}'. Only alphanumeric, '-', '_', '.', '[', ']', '=', '>', '<', ',', ';' are allowed.", token),
                    data: None,
                };
            }
        }

        let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(120000);
        // 使用捆绑的 Python 解释器执行 pip install
        let (py_cmd, py_args) = crate::ai::file_parser::python_interpreter();
        let packages = package.to_string();
        let cwd = self.working_dir.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut cmd = std::process::Command::new(&py_cmd);
            hide_window(&mut cmd);
            cmd.args(&py_args).args(["-m", "pip", "install"]);
            // 按空格切分包名作为独立参数传给 pip
            for pkg in packages.split_whitespace() {
                cmd.arg(pkg);
            }
            cmd.current_dir(&cwd).env("PYTHONIOENCODING", "utf-8").output()
        }).await;

        match result {
            Ok(Ok(out)) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let output = if stderr.is_empty() { stdout } else { format!("{}\n[stderr]\n{}", stdout, stderr) };
                ToolResult { success: out.status.success(), output, data: None }
            }
            Ok(Err(e)) => ToolResult { success: false, output: format!("Failed to spawn pip: {}", e), data: None },
            Err(_) => ToolResult { success: false, output: format!("pip install timed out after {}ms", timeout_ms), data: None },
        }
    }

    fn install_python_package_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "install_python_package",
                description: "Install one or more Python packages using pip (py -m pip install). Pass package names separated by spaces to install multiple at once, e.g. 'pandas openpyxl matplotlib'. Supports version specifiers like 'pandas==2.0.0' or 'numpy>=1.24'. Use when you need packages like pymupdf, pandas, numpy, statsmodels, sklearn, shap, matplotlib, etc.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "package": { "type": "string", "description": "One or more package names separated by spaces, e.g. 'pymupdf' or 'pandas openpyxl matplotlib'. Supports version specifiers like 'pandas==2.0.0'." },
                        "timeout_ms": { "type": "integer", "description": "Timeout in milliseconds (default 120000)" }
                    },
                    "required": ["package"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 11: CheckPythonPackage —— 检查 Python 包是否已安装
    // ════════════════════════════════════════════════════════
    async fn tool_check_python_package(&self, args: &Value) -> ToolResult {
        let package = args.get("package").and_then(|v| v.as_str()).unwrap_or("");
        if package.is_empty() {
            return ToolResult { success: false, output: "package is required".into(), data: None };
        }
        if !package.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return ToolResult { success: false, output: format!("Invalid package name: {}", package), data: None };
        }
        // 使用捆绑的 Python 解释器（优先捆绑版，fallback 系统 py）
        let (py_cmd, py_args) = crate::ai::file_parser::python_interpreter();
        let code = format!("import {}; print('OK')\n", package);
        let cwd = self.working_dir.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut cmd = std::process::Command::new(&py_cmd);
            hide_window(&mut cmd);
            cmd.args(&py_args).arg("-c").arg(&code);
            cmd.current_dir(&cwd).env("PYTHONIOENCODING", "utf-8");
            cmd.output()
        }).await;

        match result {
            Ok(Ok(out)) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let output = if stderr.is_empty() { stdout } else { format!("{}\n[stderr]\n{}", stdout, stderr) };
                ToolResult { success: out.status.success(), output, data: None }
            }
            Ok(Err(e)) => ToolResult { success: false, output: format!("Failed to spawn py: {}", e), data: None },
            Err(_) => ToolResult { success: false, output: "Command timed out after 15s".into(), data: None },
        }
    }

    fn check_python_package_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "check_python_package",
                description: "Check if a Python package is installed by running 'py -c \"import <package>\"'. Use BEFORE install_python_package to avoid unnecessary installs.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "package": { "type": "string", "description": "Module name to import, e.g. 'fitz' (pymupdf), 'pandas', 'numpy'" }
                    },
                    "required": ["package"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 12: ReadPdf —— 读取 PDF 文本内容
    // ════════════════════════════════════════════════════════
    fn tool_read_pdf(&self, args: &Value) -> ToolResult {
        let file_path = args.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
        if file_path.is_empty() {
            return ToolResult { success: false, output: "file_path is required".into(), data: None };
        }
        let path = self.resolve_path(file_path);
        let parsed = crate::ai::file_parser::parse_file(path.to_string_lossy().as_ref());
        if parsed.success {
            ToolResult { success: true, output: parsed.content, data: None }
        } else {
            ToolResult { success: false, output: parsed.error.unwrap_or_else(|| "Failed to parse PDF".into()), data: None }
        }
    }

    fn read_pdf_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "read_pdf",
                description: "Extract text content from a PDF file. Requires pymupdf (fitz); use install_python_package if not available. Prefer this over writing custom _read_pdf.py scripts.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Absolute path or workspace-relative path to the PDF" }
                    },
                    "required": ["file_path"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 13: ReadExcel —— 读取 Excel 文本内容
    // ════════════════════════════════════════════════════════
    fn tool_read_excel(&self, args: &Value) -> ToolResult {
        let file_path = args.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
        if file_path.is_empty() {
            return ToolResult { success: false, output: "file_path is required".into(), data: None };
        }
        let path = self.resolve_path(file_path);
        let parsed = crate::ai::file_parser::parse_file(path.to_string_lossy().as_ref());
        if parsed.success {
            ToolResult { success: true, output: parsed.content, data: None }
        } else {
            ToolResult { success: false, output: parsed.error.unwrap_or_else(|| "Failed to parse Excel".into()), data: None }
        }
    }

    fn read_excel_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "read_excel",
                description: "Extract text/content from an Excel (.xlsx/.xls) file using the built-in parser. Prefer this over writing custom _read_excel.py scripts.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Absolute path or workspace-relative path to the Excel file" }
                    },
                    "required": ["file_path"]
                }),
            },
        }
    }

    // ════════════════════════════════════════════════════════
    // Tool 14: CheckRuntime —— 检查编程语言运行时是否可用
    // ════════════════════════════════════════════════════════
    fn tool_check_runtime(&self, args: &Value) -> ToolResult {
        let runtime = args.get("runtime").and_then(|v| v.as_str()).unwrap_or("");
        if runtime.is_empty() {
            return ToolResult { success: false, output: "runtime is required".into(), data: None };
        }
        // 安全检查：只允许已知的运行时名称
        let known = [
            "python", "node", "java", "cpp", "c", "csharp", "go", "rust", "php",
            "python3", "gcc", "g++", "javac", "dotnet", "cargo",
        ];
        if !known.iter().any(|&k| k == runtime) && !runtime.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return ToolResult {
                success: false,
                output: format!("Unknown runtime: {} (allowed: {})", runtime, known.join(", ")),
                data: None,
            };
        }
        match detect_single_runtime(runtime) {
            Some(path) => ToolResult {
                success: true,
                output: format!("{} available at: {}", runtime, path),
                data: Some(json!({"runtime": runtime, "available": true, "path": path})),
            },
            None => ToolResult {
                success: false,
                output: format!("{} NOT found on PATH.\nInstall hint:\n{}", runtime, install_hint(runtime)),
                data: Some(json!({"runtime": runtime, "available": false})),
            },
        }
    }

    fn check_runtime_schema() -> ToolSchema {
        ToolSchema {
            kind: "function",
            function: ToolFunctionSchema {
                name: "check_runtime",
                description: "Check if a programming language runtime / compiler / SDK is installed and on PATH. Supported values: python, python3, node, java, javac, gcc, g++, c, cpp, csharp, dotnet, go, rust, cargo, php. Use this before attempting to run code in a specific language.",
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "runtime": { "type": "string", "description": "Runtime identifier, e.g. 'node', 'java', 'python3', 'gcc', 'go', 'rust', 'dotnet', 'php'" }
                    },
                    "required": ["runtime"]
                }),
            },
        }
    }
}

/// ─── 多语言运行时检测（公共函数） ───

/// 扫描本机已安装的编程语言运行时，返回可读列表
pub fn detect_runtimes() -> String {
    let mut found: Vec<String> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    // 优先检测捆绑的 Python
    if let Ok(py_path) = crate::ai::file_parser::bundled_python() {
        found.push(format!("python (bundled at {})", py_path.display()));
    } else {
        // Fallback: 检测系统 Python
        let checks: &[(&str, &str)] = &[
            ("python3", "py -3 --version"),
            ("python",   "py --version"),
        ];
        for &(name, cmd) in checks {
            let mut c = std::process::Command::new("cmd");
            hide_window(&mut c);
            let ok = c.args(["/C", cmd, ">nul", "2>nul"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if ok {
                found.push(name.to_string());
                break;
            }
        }
        if found.is_empty() {
            missing.push("  - python (no bundled or system Python found)".into());
        }
    }

    // 其他运行时
    let other_checks: &[(&str, &str)] = &[
        ("node",   "node -v"),
        ("java",   "java -version"),
        ("javac",  "javac -version"),
        ("gcc",    "gcc --version"),
        ("g++",    "g++ --version"),
        ("go",     "go version"),
        ("rust",   "rustc --version"),
        ("cargo",  "cargo --version"),
        ("dotnet", "dotnet --version"),
        ("php",    "php --version"),
    ];
    for &(name, cmd) in other_checks {
        let mut c = std::process::Command::new("cmd");
        hide_window(&mut c);
        let ok = c.args(["/C", cmd, ">nul", "2>nul"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            found.push(name.to_string());
        } else {
            missing.push(format!("  - {} ({})", name, install_hint(name)));
        }
    }

    let mut result = String::new();
    let available = if found.is_empty() { "NONE".to_string() } else { found.join(", ") };
    result.push_str(&format!(
        "### Available runtimes: {}\n",
        available
    ));
    if !missing.is_empty() {
        result.push_str("### Missing runtimes (install if needed):\n");
        for m in &missing {
            result.push_str(m);
            result.push('\n');
        }
    }
    result
}

fn detect_single_runtime(name: &str) -> Option<String> {
    // Python 优先用捆绑版
    if name == "python" || name == "python3" {
        if let Ok(path) = crate::ai::file_parser::bundled_python() {
            let mut cmd = std::process::Command::new(&path);
            hide_window(&mut cmd);
            let output = cmd
                .arg("--version")
                .output()
                .ok()?;
            if output.status.success() {
                return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
        // Fallback: 系统 py
        let mut cmd = std::process::Command::new("py");
        hide_window(&mut cmd);
        let output = cmd
            .arg("--version")
            .output()
            .ok()?;
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
        return None;
    }
    let (cmd, args): (&str, &[&str]) = match name {
        "node" => ("node", &["-v"]),
        "java" => ("java", &["-version"]),
        "javac" => ("javac", &["-version"]),
        "gcc" => ("gcc", &["--version"]),
        "g++" => ("g++", &["--version"]),
        "c" | "cpp" => ("gcc", &["--version"]),
        "go" => ("go", &["version"]),
        "rust" | "cargo" => ("rustc", &["--version"]),
        "dotnet" | "csharp" => ("dotnet", &["--version"]),
        "php" => ("php", &["--version"]),
        _ => return None,
    };

    let mut cmd = std::process::Command::new(cmd);
    hide_window(&mut cmd);
    let output = cmd
        .args(args)
        .output()
        .ok()?;

    if output.status.success() {
        let ver = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string();
        Some(ver)
    } else {
        None
    }
}

fn install_hint(name: &str) -> &'static str {
    match name {
        "python" | "python3" => "Download from https://python.org or run: winget install Python.Python.3",
        "node" => "Download from https://nodejs.org or run: winget install OpenJS.NodeJS",
        "java" | "javac" => "Download JDK from https://adoptium.net or run: winget install EclipseAdoptium.Temurin.21.JDK",
        "gcc" | "g++" | "c" | "cpp" => "Install MinGW-w64: winget install GnuWin32.Make, or MSYS2: https://msys2.org, or Visual Studio Build Tools",
        "go" => "Download from https://go.dev/dl or run: winget install GoLang.Go",
        "rust" | "cargo" => "Install via https://rustup.rs or run: winget install Rustlang.Rustup",
        "dotnet" | "csharp" => "Download .NET SDK from https://dotnet.microsoft.com or run: winget install Microsoft.DotNet.SDK.8",
        "php" => "Download from https://windows.php.net/download or run: winget install PHP.PHP",
        _ => "Search online for installation instructions",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub status: String,
    pub content: String,
    pub active_form: Option<String>,
}

// ════════════════════════════════════════════════════════
// 工具函数
// ════════════════════════════════════════════════════════

fn walk_files(base: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_files_inner(base, 0, max_depth, &mut out);
    out
}

fn walk_files_inner(dir: &Path, depth: usize, max_depth: usize, out: &mut Vec<PathBuf>) {
    if depth > max_depth { return; }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    // 跳过常见忽略目录
    let skip = [".git", "node_modules", "target", "dist", "build", ".next", "out", "__pycache__", ".venv"];
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if skip.iter().any(|s| s == &name.as_str()) { continue; }
        let path = entry.path();
        if path.is_file() {
            out.push(path);
        } else if path.is_dir() {
            walk_files_inner(&path, depth + 1, max_depth, out);
        }
    }
}

fn glob_match_simple(pat: &str, target: &str) -> bool {
    // 简化版 glob：只支持 * 和 ?
    let pat_escaped = regex::escape(pat).replace(r"\*", ".*").replace(r"\?", ".");
    Regex::new(&format!("^{}$", pat_escaped))
        .map(|r| r.is_match(target))
        .unwrap_or(false)
}

struct CmdOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

async fn run_cmd_with_timeout(command: &str, cwd: &Path, timeout_ms: u64) -> CmdOutput {
    let cmd_str = command.to_string();
    let cwd = cwd.to_path_buf();
    // 预取捆绑 Python 目录路径，注入 PATH（让 bash 中的 py/python 优先使用捆绑版）
    let python_dir = crate::ai::file_parser::bundled_python()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    tokio::task::spawn_blocking(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms.max(1));
        let spawn_child = || -> std::io::Result<std::process::Child> {
            #[cfg(target_os = "windows")]
            {
                let mut cmd = Command::new("cmd");
                hide_window(&mut cmd);
                cmd.args(&["/C", &cmd_str])
                    .current_dir(&cwd)
                    .env("PYTHONIOENCODING", "utf-8")
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());
                if let Some(ref py_dir) = python_dir {
                    let existing_path = std::env::var("PATH").unwrap_or_default();
                    cmd.env("PATH", format!("{};{}", py_dir.display(), existing_path));
                }
                cmd.spawn()
            }
            #[cfg(not(target_os = "windows"))]
            {
                let mut cmd = Command::new("sh");
                cmd.arg("-c")
                    .arg(&cmd_str)
                    .current_dir(&cwd)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());
                if let Some(ref py_dir) = python_dir {
                    let existing_path = std::env::var("PATH").unwrap_or_default();
                    cmd.env("PATH", format!("{}:{}", py_dir.display(), existing_path));
                }
                cmd.spawn()
            }
        };

        let mut child = match spawn_child() {
            Ok(c) => c,
            Err(e) => {
                return CmdOutput {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Failed to spawn: {}", e),
                    exit_code: -1,
                }
            }
        };

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // 子进程已退出，读取管道输出
                    use std::io::Read;
                    let mut stdout = String::new();
                    let mut stderr = String::new();
                    if let Some(mut s) = child.stdout.take() {
                        let mut buf = Vec::new();
                        let _ = s.read_to_end(&mut buf);
                        stdout = String::from_utf8_lossy(&buf).to_string();
                    }
                    if let Some(mut s) = child.stderr.take() {
                        let mut buf = Vec::new();
                        let _ = s.read_to_end(&mut buf);
                        stderr = String::from_utf8_lossy(&buf).to_string();
                    }
                    return CmdOutput {
                        success: status.success(),
                        stdout,
                        stderr,
                        exit_code: status.code().unwrap_or(-1),
                    };
                }
                Ok(None) => {
                    if std::time::Instant::now() >= deadline {
                        // 真超时：杀掉整个进程树，避免孙进程持有管道导致后续读取卡死
                        let pid = child.id();
                        let _ = child.kill();
                        #[cfg(target_os = "windows")]
                        if pid > 0 {
                            let _ = Command::new("taskkill")
                                .args(["/F", "/T", "/PID", &pid.to_string()])
                                .output();
                        }
                        let _ = child.wait();
                        return CmdOutput {
                            success: false,
                            stdout: String::new(),
                            stderr: format!("Command timed out after {}ms", timeout_ms),
                            exit_code: -1,
                        };
                    }
                    std::thread::sleep(std::time::Duration::from_millis(40));
                }
                Err(e) => {
                    return CmdOutput {
                        success: false,
                        stdout: String::new(),
                        stderr: format!("wait error: {}", e),
                        exit_code: -1,
                    }
                }
            }
        }
    })
    .await
    .unwrap_or_else(|_| CmdOutput {
        success: false,
        stdout: String::new(),
        stderr: "task join error".into(),
        exit_code: -1,
    })
}

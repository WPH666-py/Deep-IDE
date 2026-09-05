# 原装工作流源码 · OpenAI Codex CLI（DeepOAI 模式）

> 本文件是 **真实源码层**：DeepOAI 直接采用 OpenAI 开源 Codex CLI 的**原生系统指令与人格模板**，
> 并与 Deep-IDE 的 DeepSeek 运行时真实代码（`src-tauri/src/ai/*.rs`）融合。非风格模拟。

---

## 1. 来源清单（Provenance）

| # | 原始位置（openai/codex @ main） | 内容 | 许可 |
|---|--------------------------------|------|------|
| A | `codex-rs/core/templates/model_instructions/gpt-5.2-codex_instructions_template.md` | GPT-5.2 Codex 官方系统指令模板（真实） | Apache-2.0 |
| B | `codex-rs/core/templates/personalities/gpt-5.2-codex_pragmatic.md` | 官方 "pragmatic" 人格（真实） | Apache-2.0 |
| C | `codex-rs/prompts/templates/permissions/sandbox_mode/workspace_write.md` · `approval_policy/never.md` | 官方沙箱/审批策略模板（真实） | Apache-2.0 |
| D | Deep-IDE `src-tauri/src/ai/`（`deepseek.rs` / `agent_loop.rs` / `tools.rs`） | DeepSeek 运行时真实代码 | 本仓库 |

## 2. 原装系统指令源码片段（A · 逐字）

文件开头的身份（真实模板原文，含 `{{ personality }}` 插值点）：

```md
You are Codex, a coding agent based on GPT-5. You and the user share the same workspace and collaborate to achieve the user's goals.

{{ personality }}
```

真实"General / 编辑约束"：

```md
- When searching for text or files, prefer using `rg` or `rg --files` respectively because `rg` is much faster than alternatives like `grep`. (If the `rg` command is not found, then use alternatives.)
- Try to use apply_patch for single file edits, but it is fine to explore other options to make the edit if it does not work well. Do not use apply_patch for changes that are auto-generated (i.e. generating package.json or running a lint or format command like gofmt) or when scripting is more efficient (such as search and replacing a string across a codebase).
- You may be in a dirty git worktree. NEVER revert existing changes you did not make unless explicitly requested... 
- If asked to make a commit or code edits and there are unrelated changes ... don't revert those changes.
- Do not amend a commit unless explicitly requested to do so.
- While you are working, you might notice unexpected changes that you didn't make. If this happens, STOP IMMEDIATELY and ask the user how they would like to proceed.
- **NEVER** use destructive commands like `git reset --hard` or `git checkout --` unless specifically requested or approved by the user.
- You struggle using the git interactive console. **ALWAYS** prefer using non-interactive git commands.
```

真实"Plan tool"预算规则：

```md
- Skip using the planning tool for straightforward tasks (roughly the easiest 25%).
- Do not make single-step plans.
- When you made a plan, update it after having performed one of the sub-tasks that you shared on the plan.
```

真实"最终回答格式化"（文件引用语法）：

```md
- File References: ... Use inline code to make file paths clickable. Each reference should have a stand alone path.
- Accepted: absolute, workspace-relative, a/ or b/ diff prefixes, or bare filename/suffix.
- Optionally include line/column (1-based): :line[:column] or #Lline[Ccolumn] (column defaults to 1).
- Examples: src/app.ts, src/app.ts:42, b/server/index.js#L10, C:\repo\project\main.rs:12:5
- Don't use emojis.
```

## 3. 原装人格源码片段（B · 逐字）

```md
You are a deeply pragmatic, effective software engineer. You take engineering quality seriously, and
collaboration is a kind of quiet joy...

## Values
- Clarity: You communicate reasoning explicitly and concretely, so decisions and tradeoffs are easy to evaluate upfront.
- Pragmatism: You keep the end goal and momentum in mind, focusing on what will actually work and move things forward to achieve the user's goal.
- Rigor: You expect technical arguments to be coherent and defensible, and you surface gaps or weak assumptions politely...
```

## 4. 原装策略模板源码（C · 逐字）

```md
Filesystem sandboxing defines which files can be read or written. `sandbox_mode` is `workspace-write`:
The sandbox permits reading files, and editing files in `cwd` and `writable_roots`. Editing files in other
directories requires approval. Network access is {{ network_access }}.

Approval policy is currently never. Do not provide the `sandbox_permissions` for any reason, commands will be rejected.
```

## 5. Deep-IDE 的 DeepSeek 运行时真实代码（D · 本仓库逐字）

### 5.1 OpenAI 兼容请求体（deepseek.rs · 真实结构）

```rust
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    /// 工具定义列表（OpenAI 兼容）
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolSchema>>,
    /// 工具选择策略：auto / none / specific
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}
```

### 5.2 流式 SSE 增量（deepseek.rs · StreamToolCallDelta）

```rust
#[derive(Debug, Deserialize, Default, Clone)]
struct StreamToolCallDelta {
    index: Option<u32>,
    id: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    function: Option<StreamFunctionDelta>, // arguments 是增量 JSON 字符串片段
}
```

### 5.3 DeepOAI 循环配置（agent_loop.rs · 原装"目标检查"的运行时实现）

```rust
"deep-oai" => Self {
    max_iterations: 20,
    inject_read_before_edit_reminder: false,
    inject_progress_reminder_every: Some(5),   // Codex 的 Plan/目标检查 → 每 5 步真实注入
    require_initial_scan: false,
    require_task_decomposition: false,
    require_thinking_prefix: false,
},
```

## 6. 融合规则（Fusion Contract）

1. **指令即源码**：Codex 的系统指令模板（identity + General + Editing constraints + Plan tool + File References）
   逐字注入（见 system-prompt.md），DeepSeek 直接执行，不做二次风格改写。
2. **人格即源码**：pragmatic 人格（Clarity/Pragmatism/Rigor）来自官方模板，逐字注入。
3. **策略即源码**：`workspace-write` + approval never 的策略语义映射到 Deep-IDE 的工具权限
   （工具仅在工作区内执行写操作；`hooks.rs` 负责危险命令拦截）。
4. **循环即源码**：Plan/进度检查由 `agent_loop.rs` 每 5 步真实注入 `[Progress Check]` 消息，
   同 Codex "update plan after sub-task" 语义。
5. 全部经 `deepseek.rs` 的 OpenAI 兼容 `chat` / `chat_stream` / `chat_with_tools` 访问 DeepSeek，
   模型为 `deepseek-chat` / `deepseek-reasoner`，与原装 Codex 的模型层解耦但接口同构。

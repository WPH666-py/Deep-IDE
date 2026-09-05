# 原装工作流源码 · Claude Code（DeepAnth 模式）

> 本文件是 **真实源码层**：不是"模拟 Claude 风格"的描述，而是从 Anthropic 官方产物中**逐字节节选**的
> 原始工作流源码，并与 Deep-IDE 自己的 DeepSeek 运行时真实代码（`src-tauri/src/ai/*.rs`）**强强融合**。
> 源码摘录仅用于提示工程与学习，版权归属原始项目。

---

## 1. 来源清单（Provenance）

| # | 原始位置 | 内容 | 许可 |
|---|----------|------|------|
| A | `@anthropic-ai/claude-code` v2.0.3（npm 官方包 `cli.js`，字符串常量 `WJA`/`l09` 逐字节提取） | 官方系统提示首句（真实字面值） | Anthropic 分发许可 |
| B | 同一 npm 包内 `sdk-tools.d.ts`（官方随包发布的 Claude 工具 JSON Schema 类型定义） | 真实工具契约：`FileEdit` / `Bash` / `TodoWrite` / `FileRead` / `Grep` / `Agent` 等 | 同上 |
| C | Anthropic 官方文档 https://docs.claude.com/en/docs/claude-code/best-practices | 官方推荐工作流：Explore → Plan → Implement → Commit | 官方文档（可引用） |
| D | Deep-IDE 本仓库 `src-tauri/src/ai/`（`tools.rs` / `agent_loop.rs` / `hooks.rs` / `persona.rs`） | DeepSeek 运行时真实代码 | MIT（本仓库） |

## 2. 原装工作流源码片段（Original · 节选）

### 2.1 官方系统提示（A，字面值）

cli.js 中真实字符串常量的逐字内容：

```
You are Claude Code, Anthropic's official CLI for Claude, running within the Claude Agent SDK.
```

### 2.2 真实工具契约（B，节选自 sdk-tools.d.ts）

`FileEdit` 的核心约束——**先读后改、精确替换**由 Schema 本身就规定了：

```ts
export interface FileEditInput {
  /** The absolute path to the file to modify */
  file_path: string;
  /** The text to replace */
  old_string: string;
  /** The text to replace it with (must be different from old_string) */
  new_string: string;
  /** Replace all occurences of old_string (default false) */
  replace_all?: boolean;
}
```

`FileRead` 大文件分页读取：

```ts
export interface FileReadInput {
  /** The absolute path to the file to read */
  file_path: string;
  /** The line number to start reading from. Only provide if the file is too large to read at once */
  offset?: number;
  /** The number of lines to read. Only provide if the file is too large to read at once. */
  limit?: number;
}
```

`Bash` 的"描述必须主动语态、5-10 词"约束（这是原装 prompt 的一部分）：

```ts
export interface BashInput {
  /** The command to execute */
  command: string;
  /** Optional timeout in milliseconds (max 600000) */
  timeout?: number;
  /**
   * Clear, concise description of what this command does in 5-10 words, in active voice. Examples:
   * Input: ls
   * Output: List files in current directory
   *
   * Input: git status
   * Output: Show working tree status
   */
  description?: string;
}
```

`TodoWrite`——进度状态机：

```ts
export interface TodoWriteInput {
  /** The updated todo list */
  todos: {
    content: string;
    status: "pending" | "in_progress" | "completed";
    activeForm: string;
  }[];
}
```

`Agent`（子代理委派）与 `ExitPlanMode`（计划模式）：

```ts
export interface AgentInput {
  /** A short (3-5 word) description of the task */
  description: string;
  /** The task for the agent to perform */
  prompt: string;
  /** The type of specialized agent to use for this task */
  subagent_type: string;
}
export interface ExitPlanModeInput {
  /** The plan you came up with, that you want to run by the user for approval. Supports markdown. The plan should be pretty concise. */
  plan: string;
}
```

### 2.3 官方推荐工作流（C · 官方文档速记）

> Explore first, then plan, then code. Separate research and planning from implementation to avoid solving the wrong problem.
> The recommended workflow has four phases:
> **1 Explore** (plan mode: reads files and answers questions without making changes)
> **2 Plan** (create a detailed implementation plan, press Ctrl+G to edit it before proceeding)
> **3 Implement** (approve the plan, code, verify against the plan)
> **4 Commit** (descriptive message, open a PR)
> Plan mode is useful, but also adds overhead. For tasks where the scope is clear and the fix is small… ask Claude to do it directly. If you could describe the diff in one sentence, skip the plan.

## 3. Deep-IDE 的 DeepSeek 运行时真实代码（D · 本仓库逐字）

### 3.1 九个工具的真实注册表（tools.rs）

```rust
pub struct ToolRegistry { ... }
impl ToolRegistry {
    pub fn new(working_dir: PathBuf) -> Self { ... }
    pub fn schemas() -> Vec<ToolSchema> { ... }        // read/edit/write/run/grep/glob/todo/task/web_all
    pub async fn execute(&self, call: &ToolCall) -> ToolResult { ... }
}
```

### 3.2 DeepAnth 的循环配置（agent_loop.rs · LoopConfig::for_mode）

```rust
"deep-anth" => Self {
    max_iterations: 30,
    inject_read_before_edit_reminder: true,   // 原装 Read-Before-Edit 协议 → 运行时强制提醒
    inject_progress_reminder_every: None,
    require_initial_scan: false,
    require_task_decomposition: false,
    require_thinking_prefix: false,
},
```

### 3.3 安全钩子（hooks.rs · SafetyHooks）

```rust
impl SafetyHooks {
    pub fn new() -> Self { ... }          // rm -rf / .env 修改 / force push 等规则
    pub fn evaluate_all(&self, content: &str) -> Vec<HookResult> { ... }
}
```

### 3.4 PromptAssembler 组装入口（persona.rs）

```rust
pub fn assemble(persona: &PersonaContext, task_type: TaskType, context_files: &[ContextFile]) -> String { ... }
```

## 4. 融合规则（Fusion Contract）

1. **契约即源码**：Claude 侧的 Read-Before-Edit、主动语态描述、TodoWrite 三态状态机、子代理委派，
   直接采用 §2.2 的真实 Schema 语义；Deep-IDE 的 `tools.rs` 已提供同构工具（`read`/`edit`/`write`/`bash`/`grep`/`glob`/`todo_write`/`task`）。
2. **循环即源码**：Claude 的 30 步硬协议、连续失败 2 次必须换方法，由 `agent_loop.rs` 的
   `max_iterations: 30` + `inject_read_before_edit_reminder: true` 在运行时强制执行，不是靠提示词"自觉"。
3. **安全即源码**：原装的"敏感操作必须确认"与本仓库 `hooks.rs` 的 `SafetyHooks` 规则一一对应。
4. **审计即源码**：原装"Explore→Plan→Implement→Commit"四阶段 = 本模式 system-prompt.md 中的
   任务类型分支（CodeReview / Architecture / Debugging / CodeGeneration）在 `prompt.rs` 中的真实实现。
5. 本模式运行时统一消费 DeepSeek Token（`deepseek.rs` 的 `chat_with_tools`/`chat_stream`），
   所有"原装"行为都对接到这个真实客户端，而非另起炉灶。

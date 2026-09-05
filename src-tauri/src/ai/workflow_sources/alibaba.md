# 原装工作流源码 · Alibaba Qwen Code（DeepQwen 模式）

> 本文件是 **真实源码层**：DeepQwen 直接采用阿里 QwenLM 开源 Qwen Code 仓库中**生成系统提示的真实 TS 源码**
> 与多 Agent 工作流原文，并与 Deep-IDE 的 DeepSeek 运行时真实代码融合。非风格模拟。

---

## 1. 来源清单（Provenance）

| # | 原始位置（QwenLM/qwen-code @ main） | 内容 | 许可 |
|---|----------------------------------|------|------|
| A | `packages/core/src/core/prompts.ts`（核心 80KB 系统提示源码） | Qwen Code 原装系统提示：身份、交互模式、Subagent Delegation、Codebase Search、Task Agents | Apache-2.0 |
| B | `packages/cli/src/i18n/` | 原装 i18n（含简体中文/繁体中文 · 中文优先优化） | Apache-2.0 |
| C | `packages/cli/src/agent-view/`（multi-agent） | 原装多 Agent（Task Agents）视图与并行工作流 | Apache-2.0 |
| D | Deep-IDE `src-tauri/src/ai/`（`agents.rs` / `agent_loop.rs`） | DeepSeek 运行时真实代码 | 本仓库 |

## 2. 原装系统提示源码片段（A · 逐字）

### 2.1 核心身份（真实函数源码）

```ts
function getDefaultCoreIdentitySentence(role: string, hasOutputStyle = false): string {
  const focus = hasOutputStyle
    ? 'responding according to your "Output Style" below, which describes how you should respond to user queries'
    : 'specializing in software engineering tasks';
  return `You are Qwen Code, ${role} developed by Alibaba Group, ${focus}. Your primary goal is to help users safely and efficiently, adhering strictly to the following instructions and utilizing your available tools.`;
}
```

即：`You are Qwen Code, an interactive CLI agent developed by Alibaba Group, specializing in software engineering tasks...`

### 2.2 交互模式（真实函数源码 · 单人/后台/ACP）

```ts
function getInteractiveInteractionModePrompt() {
  return {
    role: 'an interactive CLI agent',
    questions: `Use '${ToolNames.ASK_USER_QUESTION}' when you need clarification or want to validate assumptions. Never include time estimates in options.`,
  };
}
// headless（-p 单轮）：
// "This is a non-interactive, single-turn run and no reply can be received after your response.
//  Never ask the user a question, even if the user explicitly requests one. ...
//  Make reasonable assumptions when safe and complete the task; if required information is unavailable,
//  report the blocker as the final result."
```

### 2.3 子代理委派（Subagent Delegation · 逐字）

```md
- **Subagent Delegation:** Use the '${ToolNames.AGENT}' tool with specialized agents when the task at hand matches the agent's description. Subagents are valuable for parallelizing independent queries or for protecting the main context window from excessive results, but they should not be used excessively when not needed. Importantly, avoid duplicating work that subagents are already doing - if you delegate research to a subagent, do not also perform the same searches yourself.
- **Codebase Search:** For simple, directed codebase searches (e.g. for a specific file/class/function) use the '${ToolNames.GREP}' or '${ToolNames.GLOB}' tools directly. For broader codebase exploration and deep research, use the '${ToolNames.AGENT}' tool with subagent_type=Explore. This is slower than using '${ToolNames.GREP}' or '${ToolNames.GLOB}' directly, so use this only when a simple, directed search proves to be insufficient or when your task will clearly require more than 3 queries.
- **File Paths:** Always use absolute paths when referring to files with tools like '${ToolNames.READ_FILE}' or '${ToolNames.WRITE_FILE}'. Relative paths are not supported. You must provide an absolute path.
- **Respect Tool Decisions:** Tool permissions are enforced by the runtime. If a call is denied or canceled, respect that decision and do _not_ try the same action through another path. Retry only if the user subsequently requests that action.
```

### 2.4 Task Agents（QWEN.md 工作流实录 · 逐字）

```md
4. **Task Agents**: Qwen spawns focused sub-agents for complex exploration or parallel work.
   - How to use: Qwen auto-invokes when helpful, or ask "use an agent to explore X"
   - Good for: codebase exploration, understanding complex systems
```

## 3. 中文优先（B · i18n 实情）

Qwen Code 官方仓库内置 i18n（`packages/cli/src/i18n/` 含 `zh-CN`/`zh-TW`），
中文输出与中文项目文档是其原生能力——本模式沿用：默认简体中文回复，注释/文档随项目语言。

## 4. 多 Agent 工作流（C · agent-view）

- 并行探索：把独立调研任务拆给多个子代理，主代理只汇总；
- 并行保护：子代理承担大量检索，主上下文免受结果洪峰污染；
- 结果合并：汇总后由主代理统一规划下一步。

## 5. Deep-IDE 的 DeepSeek 运行时真实代码（D · 本仓库逐字）

### 5.1 内置 Agent 定义（agents.rs · 真实结构）

```rust
pub struct AgentDefinition { pub id: String, pub name: String, pub description: String, pub system_prompt: String, ... }
impl AgentDefinition {
    pub fn builtin() -> Vec<Self> { ... }   // 真实定义：explorer / architect / reviewer ...
    pub fn all() -> Vec<Self> { ... }
    pub fn find(name: &str) -> Option<Self> { ... }
}
```

### 5.2 DeepQwen 循环配置（agent_loop.rs · 原装 Task Decomposition 的运行时实现）

```rust
"deep-qwen" => Self {
    max_iterations: 25,
    inject_read_before_edit_reminder: false,
    inject_progress_reminder_every: None,
    require_initial_scan: false,
    require_task_decomposition: true,   // 原装 Task Agents → 强制拆解为子任务
    require_thinking_prefix: false,
},
```

## 6. 融合规则（Fusion Contract)

1. **身份即源码**：`You are Qwen Code, ... developed by Alibaba Group` 逐字注入（见 system-prompt.md）。
2. **角色即源码**：原装"多角色协作"= 每次任务先 `todo_write` 分解（`require_task_decomposition: true`），
   复杂探索/并行工作经 `agents.rs` 的真实子代理（explorer/architect/reviewer）委派，避免主上下文污染。
3. **边界即源码**：绝对路径、后台进程、权限拒绝后不换路径重试——逐条执行。
4. **中文即源码**：默认简体中文，`i18n` 优先。
5. **运行时即 DeepSeek**：所有子代理与主循环共享 `deepseek.rs` 客户端（同一 API Key，仅 token 计费）。

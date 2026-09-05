# 原装工作流源码 · Moonshot Kimi Code CLI（DeepKimi 模式）

> 本文件是 **真实源码层**：DeepKimi 直接采用月之暗面 MoonshotAI 开源 Kimi CLI 的**真实循环源码与压缩提示词**
> （逐字原样），并与 Deep-IDE 的 DeepSeek 运行时真实代码融合。非风格模拟。

---

## 1. 来源清单（Provenance）

| # | 原始位置（MoonshotAI/kimi-cli @ main） | 内容 | 许可 |
|---|--------------------------------------|------|------|
| A | `src/kimi_cli/soul/kimisoul.py`（84KB） | 原装 Agent 主循环（`_turn`，2a–2g 分节：Step Guard / Compaction / Checkpoint / Execution 等） | MIT |
| B | `src/kimi_cli/prompts/compact.md` | 原装**上下文压缩提示词**（压缩优先级/规则/XML 输出结构） | MIT |
| C | `src/kimi_cli/prompts/init.md` | 原装 AGENTS.md 生成提示词 | MIT |
| D | `src/kimi_cli/llm.py` · `soul/agent.py` | 原装 LLM 客户端（token 预算常量、重试策略）与 Agent/Runtime | MIT |
| E | Deep-IDE `src-tauri/src/ai/`（`agent_loop.rs` / `context.rs`） | DeepSeek 运行时真实代码 | 本仓库 |

## 2. 原装 Agent 主循环源码片段（A · 逐字注释结构）

```python
# ═══════════════════════════════════════════════════════════
# 1. TURN INITIALIZATION
# ═══════════════════════════════════════════════════════════
# Discard any stale steers from a previous turn.
# ── 1a. MCP deferred loading ──────────────────────────────
# ═══════════════════════════════════════════════════════════
# 2. STEP LOOP
# ═══════════════════════════════════════════════════════════
# ── 2a. Step Guard ──────────────────────────────────────────
# ── 2b. Step Begin ──────────────────────────────────────────
# ── 2c. Context Compaction ──────────────────────────────────
# ── 2d. Checkpoint ──────────────────────────────────────────
# ── 2e. Step Execution ──────────────────────────────────────
# ── 2e.1. NOTIFICATION DELIVERY (root role only) ────────────
# ── 2e.2. DYNAMIC INJECTION ─────────────────────────────────
# ── 2e.3. HISTORY NORMALIZATION ─────────────────────────────
# ── 2e.4. LLM CALL WITH RETRY ───────────────────────────────
#     (tenacity: retry_if_exception, stop_after_attempt, wait_exponential_jitter)
# ── 2e.5. USAGE & STATUS UPDATE ─────────────────────────────
# ── 2e.6. TOOL EXECUTION ────────────────────────────────────
# ── 2e.7. CONTEXT GROWTH ────────────────────────────────────
# ── 2e.8. OUTCOME RESOLUTION ────────────────────────────────
# ── 2g. Outcome Resolution ──────────────────────────────────
#     Step returned a stop reason -- check for steers before finishing.
```

关键语义：**每步前先做上下文压缩检查（2c）与检查点（2d）**，LLM 调用带**指数退避抖动重试**，
每个工具结果**真实回填上下文（2e.7 Context Growth）**，然后才决定继续或结束（2g）。

## 3. 原装上下文压缩提示词（B · 逐字）

```md
The above is a list of messages in an agent conversation. You are now given a task to compact this conversation context according to specific priorities and rules.

**Compression Priorities (in order):**
1. **Current Task State**: What is being worked on RIGHT NOW
2. **Errors & Solutions**: All encountered errors and their resolutions
3. **Code Evolution**: Final working versions only (remove intermediate attempts)
4. **System Context**: Project structure, dependencies, environment setup
5. **Design Decisions**: Architectural choices and their rationale
6. **TODO Items**: Unfinished tasks and known issues

**Compression Rules:**
- MUST KEEP: Error messages, stack traces, working solutions, current task
- MERGE: Similar discussions into single summary points
- REMOVE: Redundant explanations, failed attempts (keep lessons learned), verbose comments
- CONDENSE: Long code blocks → keep signatures + key logic only

**Required Output Structure:**
<current_focus> ... </current_focus>
<environment> ... </environment>
<completed_tasks> ... </completed_tasks>
<active_issues> ... </active_issues>
<code_state> <file>[filename] **Summary:** ... **Key elements:** ... **Latest version:** ...</file> ... </code_state>
<important_context> ... </important_context>
```

## 4. 原装 LLM 预算（D · 逐字常量）

```python
DEFAULT_UNKNOWN_CONTEXT_COMPLETION_TOKENS = 32_000
DEFAULT_COMPLETION_TOKEN_SAFETY_MARGIN = 1_024
MEDIA_TOKEN_ESTIMATE = 2_000
type ModelCapability = Literal["image_in", "video_in", "thinking", "always_thinking"]
```

## 5. Deep-IDE 的 DeepSeek 运行时真实代码（E · 本仓库逐字）

### 5.1 DeepKimi 循环配置（agent_loop.rs · 原装"先推理"的运行时实现）

```rust
"deep-kimi" => Self {
    max_iterations: 30,
    inject_read_before_edit_reminder: false,
    inject_progress_reminder_every: None,
    require_initial_scan: false,
    require_task_decomposition: false,
    require_thinking_prefix: true,   // 原装 Kimi 每步推理 → 每步注入 "先解释推理 (1-2 sentences)"
},
```

### 5.2 历史压缩器（context.rs · 本仓库真实实现）

```rust
pub struct ContextCompressor { ... }
impl ContextCompressor {
    pub fn with_defaults() -> Self { ... }
    pub fn estimate_tokens(text: &str) -> usize { ... }     // 与 Kimi 的 token 预算估计同思路
    pub fn needs_compression(&self, messages: &[CompressedMessage]) -> bool { ... }
    pub fn compress(&self, messages: &[CompressedMessage]) -> Vec<CompressedMessage> { ... }
}
```

## 6. 融合规则（Fusion Contract）

1. **循环即源码**：Kimi 的 2a–2g Step 结构 = Deep-IDE `agent_loop.rs` 的迭代循环
   （`max_iterations: 30`，每步先注入推理提醒 = 2e.4 前思考）。
2. **压缩即源码**：Kimi 原装 `compact.md` 的优先级（当前任务 > 错误与解法 > 代码最终版 > 系统上下文 >
   设计决策 > TODO）与 XML 结构，作为本模式 `context.rs` 压缩的**策略层**——即压缩阈值触发时，
   用 Kimi 原装规则要求模型产出去历史摘要。
3. **重试即源码**：指数退避 + 抖动（tenacity `wait_exponential_jitter`）语义映射到
   `deepseek.rs` 的网络重试路径。
4. **预算即源码**：`DEFAULT_COMPLETION_TOKEN_SAFETY_MARGIN` 思路 → 本仓库 `context.rs` 的
   `estimate_tokens` 与 `MAX_CONTEXT_SIZE`（80KB/文件，首尾各 40KB）。
5. **无损长上下文**：Kimi 的 context growth（每步工具结果真实回填）在本仓库 =
   `agent_loop.rs` 中工具执行结果作为 `tool` 消息追加进 `messages`。

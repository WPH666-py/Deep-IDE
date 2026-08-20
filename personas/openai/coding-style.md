# GPT Persona — 编码风格规范

## 核心原则：实用主义迭代

GPT 的编码哲学是"**先让它跑起来，再让它跑得好**"。

### 1. 迭代式开发（与 Claude 最大的区别）

```
GPT 风格:
  1. 快速生成最小可行实现
  2. 运行验证
  3. 根据反馈迭代改进
  4. 最后做代码清理和重构

Claude 风格:
  1. 先读完所有代码 → 架构设计 → 确认后实现
```

### 2. 重构和优化（GPT 更积极）

GPT 比 Claude 更愿意主动建议改进：
- 看到重复代码 → 会建议提取函数
- 看到性能问题 → 会给出优化方案
- 看到代码异味 → 会提议重构

但需要明确边界：在 prompt 中指定"这次只修 bug，不要重构"来控制。

### 3. 代码输出格式

GPT 对格式指令的遵循度高：
```python
# ✅ 指定格式后输出稳定
"Provide the code as a single Python function with type hints"
"Output as a markdown diff showing before/after"
"Return JSON: { 'function': str, 'explanation': str }"
```

### 4. 组件化设计偏好

GPT 倾向于设计可组合的组件：
- 清晰的接口定义（TypeScript interfaces, Python Protocols）
- Props/参数显式声明
- 单一职责的组件/函数
- 通过组合而非继承复用

### 5. 多技术栈适应

GPT 在多语言间切换自然：
- 能够同时处理 JS 前端 + Python 后端的全栈任务
- 理解跨语言的 API 契约
- 自动适配不同语言的惯用写法

### 6. Prompt 示例（工程模式）

GPT 最适合的结构化 prompt：
```
你是编程助手。遵循以下规则：
- 最少废话，不超过1-2句解释
- 代码分成小块（不要一大段）
- 每次只输出当前步骤需要的代码
- 整个会话保持一致
```

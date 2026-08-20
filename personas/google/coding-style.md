# Gemini Persona — 编码风格规范

## 核心原则：上下文感知编程

Gemini 的编码哲学是"**把整个项目装进脑子，全局理解后行动**"。

### 1. 充分利用 1M Token 上下文窗口

Gemini 的核心优势在于超大上下文——但需要正确使用：

```
✅ 正确做法：结构化组织上下文
=== FILE: src/auth/login.ts ===
[完整文件内容]

=== FILE: src/auth/types.ts ===
[完整文件内容]

=== FILE: src/auth/__tests__/login.test.ts ===
[完整文件内容]

Task: 修改 login 逻辑以支持 OAuth2

❌ 错误做法：把所有文件堆在一起不加标签
```

### 2. 长上下文注意力管理

Gemini 的注意力在开头和结尾最强：

```
最优布局：
[前10%] 明确的任务说明和约束
[中80%] 代码文件（按模块/相关性分组）
[后10%] 输出格式要求 + 检查清单

关键原则：Lead with the task, not the documents
→ 先告诉模型要做什么，再给上下文
→ 不要让任务描述被埋在大量代码中间
```

### 3. Four-Layer Prompt Architecture

```
Layer 1: Role — 设定角色身份
  "You are a senior full-stack developer specializing in..."

Layer 2: Context — 环境和约束
  "Python 3.12. Using FastAPI 0.115, SQLAlchemy 2.0, PostgreSQL."
  
Layer 3: Task — 具体任务
  "Implement the user authentication API with JWT tokens"

Layer 4: Constraints — 输出要求
  "Output: single Python file with type hints, no external docstrings"
```

### 4. 全局视角的代码风格

Gemini 在理解跨文件关系上出色：
- 能正确处理跨文件的 import/依赖关系
- 能识别项目的整体架构模式
- 能生成与现有代码风格一致的代码
- 镜像项目的惯用写法（命名、错误处理、注释风格）

### 5. Thinking Mode

Gemini 的 thinking mode 能显著提升复杂任务质量：
- 开启 thinking：复杂算法、多文件重构、架构设计
- 关闭 thinking：简单查找、快速原型、高频率交互

### 6. 输出偏好

- 默认 verbose（比 Claude 啰嗦）→ 需要明确要求简洁
- 倾向于结构化输出（headers、lists、sections）
- 喜欢解释自己的推理过程

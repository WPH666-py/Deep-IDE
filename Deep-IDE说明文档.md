# Deep-IDE 说明文档 / Documentation

> 新一代智能体 IDE · Next-Generation Agentic IDE
> 用最简洁的架构，做最牛逼的产品！— 水哥

---

# 一、中文说明（约 2000 字）

## 1. 项目简介

Deep-IDE 是一款面向现代开发者的新一代智能体集成开发环境（Agentic IDE），由青岛理工大学 2022 级毕业生水哥独立设计与开发。它并非传统编辑器的简单复刻，而是将「代码编辑」「AI 大模型辅助」「多智能体工具调用」「多语言运行环境」「文件解析」「版本控制」与「插件生态」深度融合的桌面级生产力工具。

Deep-IDE 的核心理念是「简洁架构 + 极致体验」。底层采用 Rust 与 Tauri 2 构建，前端使用 Vue 3 与 TypeScript，实现了接近原生的启动速度与极低的内存占用，同时规避了 Electron 类应用常见的体积臃肿与性能损耗问题。无论你是学生、独立开发者还是团队工程师，Deep-IDE 都能在一个窗口内完成从「新建项目 → 编辑代码 → AI 辅助开发 → 运行调试 → 版本提交」的完整闭环。

**开发动机**：作者在实际开发中发现，主流 IDE 要么功能臃肿、要么 AI 能力割裂——用户常常需要在编辑器、AI 对话工具、命令行终端与文件解析工具之间频繁切换，效率低且上下文容易丢失。与此同时，市面上的 AI 编程助手虽多，但大多按模型分别订阅，成本高且风格单一。Deep-IDE 的出发点，就是用「单一 DeepSeek 运行时 + 离线 Persona 注入」的架构，在同一个桌面窗口内整合编辑、AI 辅助、多智能体工具调用、文件运行与版本控制，让不同风格的模型体验可以一键切换、按需定制，同时把使用成本压到最低。

## 2. 核心特性

- **跨平台桌面应用**：基于 Tauri 2，支持 Windows、macOS 与 Linux，Windows 端提供原生 NSIS 安装包。
- **多标签代码编辑器**：内置 CodeMirror 编辑器，支持语法高亮、主题切换（经典纯白 / 护眼淡绿 / 深色专业 / GitHub 深色）与多种编程语言。
- **界面皮肤换肤**：DeepKing 同款「界面皮肤 / UI Skin」接口，内置 GitHub 灰蓝配色（亮/暗两套，覆盖文件树、编辑区、AI 区域），支持粘贴任意 GitHub 仓库地址"转换并添加"自定义皮肤。
- **文件树与资源管理**：完整的文件树浏览、新建/重命名/删除/复制/剪切/粘贴，支持拖拽调整面板宽度。
- **多智能体 AI 助手**：内置 DeepAnth、DeepOAI、DeepGem、DeepQwen、DeepKimi 五种离线 Persona 工作流，统一走 DeepSeek 大模型运行时。
- **Agent Loop 工具调用**：提供 Claude Code / Cursor 风格的九工具 Agent 循环，支持实时代码读写、命令行执行、依赖安装等自动化操作。
- **多语言一键运行**：支持 Python、JavaScript、TypeScript、Java、Go、Rust、C、C++、C#、PHP、SQL、MATLAB、Shell 等多种语言文件的自动识别与运行。
- **内置终端**：底部集成「终端 / 输出」面板，支持输入命令、查看运行结果、导出与复制输出。
- **智能文件解析**：纯 Rust 解析 Office 文件（Excel / Word / PowerPoint），支持 PDF、CSV、图片预览，二进制文件用系统默认程序打开。
- **Git 集成**：内置 Git 状态查看与一键推送，配合 GitHub Token 完成远程仓库提交。
- **插件市场**：接入 VS Code 插件市场，支持搜索、安装与管理软件和插件。
- **会话持久化**：AI 对话、思考过程、生成结果均本地持久化，刷新页面不丢失。

## 3. 技术架构

Deep-IDE 采用前后端分层架构。前端使用 Vue 3 + TypeScript + Vite 构建，编辑器基于 CodeMirror 6，状态管理使用 Pinia；后端使用 Rust 编写 Tauri 命令，通过 IPC 与前端通信。文件解析模块大量使用纯 Rust 库：Excel 采用 calamine，Word 与 PowerPoint 采用 zip + XML 解析，文本文件直接用 `std::fs` 读取并支持 UTF-8 / UTF-16 / GBK 编码自动识别；PDF 则通过内置的 Python pymupdf 兜底处理。所有子进程均通过 `CREATE_NO_WINDOW` 标志隐藏，避免弹出黑色命令行窗口。

**多模态是怎么支持的**：Deep-IDE 通过统一的文件解析入口实现「多模态上下文」。普通文本与代码文件支持 UTF-8 / UTF-16 / GBK 编码自动识别；Excel 用纯 Rust 库 calamine、Word / PowerPoint 用 zip + XML 提取文本，PDF 则由内置 Python pymupdf 兜底；图片虽不读取像素，但会以「路径 + 格式 + 大小」的占位描述注入上下文供 AI 引用。所有解析结果统一转成结构化文本（含格式名、字节数、是否截断等字段），超过 80KB 的大文件按「首尾各 40KB」截断，再由 Prompt 组装器拼入 System Prompt，让 AI 能"看懂"代码、文档、表格、幻灯片、PDF 等多种格式。

## 4. 快速开始

1. 下载并安装 Deep-IDE 安装包（Windows 为 NSIS 安装程序）。
2. 启动后点击「开始 → 新建项目 / 打开项目」，选择或创建工作目录。
3. 在左侧文件树中双击文件即可编辑；图片直接预览，Office / PDF 文件用系统默认程序打开。
4. 在右侧「AI 助手」面板点击「配置」填入 DeepSeek API Key，即可开启 AI 辅助开发。
5. 顶部选择「运行环境」与「运行文件」，点击「运行」，输出自动显示在底部终端。

## 5. 功能详解

### 5.1 项目管理

通过「开始」菜单可新建或打开项目。项目列表清晰展示「对话 / 编程」模式，进入项目后顶部同样可以查看当前项目与模式，方便用户随时确认所处上下文。

### 5.2 文件编辑

编辑器支持多标签切换、保存、另存为。文件树右键菜单提供新建文件、新建文件夹、重命名、复制路径、剪切、复制、粘贴、删除等完整操作，所有文件操作均限定在项目目录内，安全可控。

### 5.3 AI 助手与多模式

AI 助手支持五种工作流（DeepAnth / DeepOAI / DeepGem / DeepQwen / DeepKimi），每种模式通过离线 Persona 注入模拟不同的开发风格，所有模式统一消耗 DeepSeek Token。

**什么是 Persona 注入**：Persona 注入（Persona Injection）是一种提示工程（Prompt Engineering）方法。Deep-IDE 实际只调用 DeepSeek 这一个真实大模型，但借助本地离线的 Persona 配置——每个模式对应一个目录，内含 `persona.toml` 及多份 Markdown 知识文件——在每次请求前动态组装 System Prompt。Prompt 组装器会按权重把「身份设定、编码风格、审查清单、架构思维、协作模式、任务工作流」等内容注入系统提示词，让同一个底层模型表现出截然不同的"性格"与工作风格。这种注入完全离线，不额外调用其他模型，只消耗 DeepSeek Token。

**五种模式分别对应什么**：

- **DeepAnth** —— 模拟 Anthropic 的 Claude，安全审查极严、架构先行、防御式编码，适合安全审计与复杂重构。
- **DeepOAI** —— 模拟 OpenAI 的 GPT，快速迭代、实用主义、组件化思维，适合快速原型与功能开发。
- **DeepGem** —— 模拟 Google 的 Gemini，全局视角、并行分析、长上下文理解，适合大型代码库分析。
- **DeepQwen** —— 模拟阿里的 Qwen，多角度协作、中文优化、Agent 中心，适合中文项目与多角色协作。
- **DeepKimi** —— 模拟月之暗面的 Kimi，逐步推理、任务分解、无损长上下文，适合超长文档与结构化分析。

### 5.4 工具调用 Agent Loop

开启「🛠 工具」后，AI 可进入 Agent Loop 模式，自动调用读写文件、执行命令、安装依赖等工具，并在「工具」下拉中实时展示工具名称、参数与执行结果，支持展开查看完整详情。

### 5.5 运行文件

顶部可选择运行环境与运行文件，系统根据文件扩展名自动选择解释器或编译器。运行结果实时输出到底部终端，Python 强制 UTF-8 输出避免中文乱码，所有编译与运行子进程均隐藏窗口。

### 5.6 终端

底部终端支持输入命令、查看输出，并提供「结果导出」「复制」「清空」按钮。终端输入框会自动聚焦，方便连续输入。

### 5.7 Git 集成

「Git 提交」弹框支持填写 GitHub 用户名、Token、目标仓库、分支与提交信息，一键推送到远程仓库，并可先「检查状态」查看当前分支与变更情况。

### 5.8 插件市场

软件和插件市场接入 VS Code 插件市场，支持按相关性、下载量、评分等排序搜索，展示插件图标、名称、发布者与描述，一键安装并存到本地。

## 6. 运行环境支持

Deep-IDE 提供增强版运行时检测，自动扫描 PATH 及 C/D/E 盘常见安装目录，识别 Python、Node.js、npm、Java、Go、Rust、gcc、git、Docker、PHP、dotnet 等运行时及其版本，并在顶部下拉框中以「✓ / ✗」标识可用性。

## 7. 关于作者与联系方式

- **作者（昵称）**：水哥
- **毕业院校**：青岛理工大学，2022 级毕业生
- **联系方式**：943050454@qq.com
- **项目理念**：Deep IDE，新一代智能体 IDE。用最简洁的架构，做最牛逼的产品！

如有任何问题、建议或合作需求，欢迎通过上述邮箱联系水哥。

---

# 二、English Documentation (~2000 words)

## 1. Introduction

Deep-IDE is a next-generation agentic integrated development environment (IDE) designed for modern developers. It was independently designed and developed by "Brother Shui" (水哥), a 2022 graduate of Qingdao University of Technology. Deep-IDE is not a simple clone of a traditional editor; instead, it deeply integrates code editing, large language model assistance, multi-agent tool calling, multi-language runtime execution, file parsing, version control, and a plugin ecosystem into a single, powerful desktop productivity tool.

The core philosophy of Deep-IDE is "simple architecture, ultimate experience." The backend is built with Rust and Tauri 2, while the frontend uses Vue 3 and TypeScript, delivering near-native startup speed and extremely low memory footprint, while avoiding the bloated size and performance issues commonly associated with Electron-based applications. Whether you are a student, an independent developer, or a team engineer, Deep-IDE lets you complete the entire workflow — "create a project → edit code → AI-assisted development → run and debug → commit to version control" — all within a single window.

**Development Motivation**: During real-world development, the author observed that mainstream IDEs are either bloated or have fragmented AI capabilities — users often have to switch frequently between an editor, an AI chat tool, a command-line terminal, and a file parsing tool, which is inefficient and prone to losing context. Meanwhile, although there are many AI programming assistants on the market, most require separate subscriptions per model, which is costly and stylistically rigid. The starting point of Deep-IDE was to use a "single DeepSeek runtime + offline Persona injection" architecture to unify editing, AI assistance, multi-agent tool calling, file execution, and version control within a single desktop window — allowing different model experiences to be switched with one click, customized on demand, while keeping usage cost to a minimum.

## 2. Core Features

- **Cross-platform desktop application**: Built on Tauri 2, supporting Windows, macOS, and Linux, with a native NSIS installer provided for Windows.
- **Multi-tab code editor**: Ships with a CodeMirror-based editor featuring syntax highlighting, theme switching (Classic White / Eye-friendly Green / Professional Dark / GitHub Dark), and support for many programming languages.
- **UI Skin system**: DeepKing-style "UI Skin" interface with built-in GitHub light/dark skins (covering the file tree, editor area and AI panel), plus pasting any GitHub repo URL to convert it into a custom skin.
- **File tree and resource management**: A complete file tree browser with create, rename, delete, copy, cut, and paste operations, plus draggable panel resizing.
- **Multi-agent AI assistant**: Five offline Persona workflows — DeepAnth, DeepOAI, DeepGem, DeepQwen, and DeepKimi — all unified through the DeepSeek large model runtime.
- **Agent Loop tool calling**: Provides a Claude Code / Cursor-style nine-tool agent loop with real-time code reading and writing, command-line execution, and dependency installation automations.
- **Multi-language one-click run**: Automatically detects and runs files in Python, JavaScript, TypeScript, Java, Go, Rust, C, C++, C#, PHP, SQL, MATLAB, Shell, and more.
- **Built-in terminal**: An integrated "Terminal / Output" panel at the bottom supports command input, output viewing, and export/copy of results.
- **Smart file parsing**: Parses Office files (Excel / Word / PowerPoint) with pure Rust, supports PDF, CSV, and image preview, and opens binary files with the system default application.
- **Git integration**: Built-in Git status viewing and one-click push, working with a GitHub Token to commit to remote repositories.
- **Plugin marketplace**: Connects to the VS Code extension marketplace for searching, installing, and managing software and plugins.
- **Session persistence**: AI conversations, reasoning processes, and generated results are persisted locally and survive page refreshes.

## 3. Technical Architecture

Deep-IDE uses a layered front-end/back-end architecture. The front end is built with Vue 3 + TypeScript + Vite, uses CodeMirror 6 for editing, and Pinia for state management; the back end implements Tauri commands in Rust and communicates with the front end through IPC. The file parsing module makes heavy use of pure Rust libraries: Excel via calamine, Word and PowerPoint via zip + XML parsing, and text files via `std::fs` with automatic UTF-8 / UTF-16 / GBK encoding detection. PDF files are handled by a bundled Python pymupdf fallback. All subprocesses are hidden using the `CREATE_NO_WINDOW` flag to prevent black command-line windows from appearing.

**How multimodal support works**: Deep-IDE implements "multimodal context" through a unified file parsing entry point. Plain-text and code files support automatic UTF-8 / UTF-16 / GBK encoding detection; Excel is parsed with the pure Rust library calamine, Word / PowerPoint via zip + XML text extraction, and PDF through a bundled Python pymupdf fallback. Although images do not have their pixels read, they are injected into the context as a placeholder description of "path + format + size" for the AI to reference. All parsed results are uniformly converted into structured text (including format name, byte count, truncation flag, and other fields); files over 80 KB are truncated to "first and last 40 KB each", and then assembled into the System Prompt by the prompt assembler, allowing the AI to "understand" various formats such as code, documents, spreadsheets, slides, and PDFs.

## 4. Quick Start

1. Download and install the Deep-IDE installer (an NSIS installer on Windows).
2. After launching, click "Start → New Project / Open Project" and choose or create a working directory.
3. Double-click a file in the left file tree to edit it; images are previewed inline, and Office / PDF files are opened with the system default application.
4. In the right "AI Assistant" panel, click "Configure" and enter your DeepSeek API Key to enable AI-assisted development.
5. Choose a "Runtime" and a "Run File" at the top, then click "Run"; output is displayed automatically in the bottom terminal.

## 5. Feature Details

### 5.1 Project Management

The "Start" menu lets you create or open projects. The project list clearly displays the "Conversation / Programming" mode, and after entering a project the top bar also shows the current project and mode, making it easy to confirm context at a glance.

### 5.2 File Editing

The editor supports multi-tab switching, saving, and save-as. The file tree context menu provides complete operations including new file, new folder, rename, copy path, cut, copy, paste, and delete. All file operations are restricted to the project directory for safety and control.

### 5.3 AI Assistant and Multiple Modes

The AI Assistant supports five workflows (DeepAnth / DeepOAI / DeepGem / DeepQwen / DeepKimi). Each mode simulates a different development style through offline Persona injection, and all modes consume DeepSeek tokens.

**What is Persona injection?** Persona Injection is a prompt engineering method. Deep-IDE actually calls only one real large model — DeepSeek — but relies on local offline Persona configurations (each mode corresponds to a directory containing `persona.toml` plus multiple Markdown knowledge files) to dynamically assemble the System Prompt before every request. The prompt assembler injects "identity setting, coding style, review checklist, architectural thinking, collaboration patterns, and task workflows" into the system prompt according to their weights, making the same underlying model exhibit entirely different "personalities" and working styles. This injection is fully offline, does not call any other model, and only consumes DeepSeek tokens.

**What do the five modes correspond to?**

- **DeepAnth** — Emulates Anthropic's Claude: extremely strict security review, architecture-first, defensive coding; ideal for security audits and complex refactoring.
- **DeepOAI** — Emulates OpenAI's GPT: rapid iteration, pragmatism, componentized thinking; ideal for quick prototyping and feature development.
- **DeepGem** — Emulates Google's Gemini: global perspective, parallel analysis, long-context understanding; ideal for large codebase analysis.
- **DeepQwen** — Emulates Alibaba's Qwen: multi-angle collaboration, Chinese optimization, agent-centric; ideal for Chinese projects and multi-role collaboration.
- **DeepKimi** — Emulates Moonshot AI's Kimi: step-by-step reasoning, task decomposition, lossless long context; ideal for very long documents and structured analysis.

### 5.4 Tool Calling Agent Loop

After enabling "🛠 Tools", the AI can enter Agent Loop mode, automatically invoking tools such as reading/writing files, executing commands, and installing dependencies. The "Tools" dropdown displays tool names, parameters, and execution results in real time, with expandable views for complete details.

### 5.5 Running Files

The top bar lets you select the runtime and the file to run. The system automatically chooses the correct interpreter or compiler based on the file extension. Output is streamed into the bottom terminal in real time; Python forces UTF-8 output to avoid Chinese garbling, and all compile/run subprocess windows are hidden.

### 5.6 Terminal

The bottom terminal supports command input and output viewing, and provides "Export", "Copy", and "Clear" buttons. The terminal input automatically focuses to enable continuous typing.

### 5.7 Git Integration

The "Git Commit" dialog supports entering a GitHub username, token, target repository, branch, and commit message, pushing to a remote repository with one click, and optionally checking repository status first.

### 5.8 Plugin Marketplace

The software and plugin marketplace connects to the VS Code extension marketplace, supporting search sorted by relevance, downloads, rating, and more. It displays plugin icons, names, publishers, and descriptions, with one-click installation stored locally.

## 6. Runtime Environment Support

Deep-IDE provides enhanced runtime detection that automatically scans PATH and common installation directories on drives C/D/E, recognizing Python, Node.js, npm, Java, Go, Rust, gcc, git, Docker, PHP, dotnet, and their versions, marking availability with "✓ / ✗" in the top dropdown.

## 7. About the Author and Contact

- **Author (nickname)**: Brother Shui (水哥)
- **Alma mater**: Qingdao University of Technology, Class of 2022 graduate
- **Contact**: 943050454@qq.com
- **Project philosophy**: Deep IDE — a next-generation agentic IDE. With the simplest architecture, build the most awesome product!

For any questions, suggestions, or collaboration requests, feel free to reach out to Brother Shui via the email address above.

---

# 三、联系 / Contact

| 项目 | 信息 |
| --- | --- |
| 作者 | 水哥 (Brother Shui) |
| 毕业院校 | 青岛理工大学 · 2022 级毕业生 |
| 邮箱 | 943050454@qq.com |
| 定位 | 新一代智能体 IDE |
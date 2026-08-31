import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { tauriAPI, type ModeInfo, type Message, type AgentDef, type FileEntry } from "../services/tauri-api";
import type { EditorTheme } from "../utils/codemirror";
import { BUILTIN_SKINS, applyUISkinCss, prewarmUISkinCss, skinCssFor, convertPaletteToSkin, type UISkin } from "../utils/skins";

export const useAppStore = defineStore("app", () => {
  const currentProject = ref<string | null>(null);
  const currentMode = ref<string>("deep-anth");
  const currentAgent = ref<string>(""); // "" = 无 Agent
  const apiKey = ref<string>("");
  const baseUrl = ref<string>("https://api.deepseek.com");
  const model = ref<string>("deepseek-chat");
  const validThemes: EditorTheme[] = ["classic", "green", "dark", "github"];
  const savedTheme = localStorage.getItem("editorTheme") as EditorTheme | null;
  const editorTheme = ref<EditorTheme>(savedTheme && validThemes.includes(savedTheme) ? savedTheme : "classic");

  // ─── UI Skin（界面皮肤，覆盖文件树/编辑区/AI 区域）───
  const uiSkinId = ref<string>(localStorage.getItem("uiSkinId") || "");
  const uiSkinVariant = ref<"light" | "dark">(localStorage.getItem("uiSkinVariant") === "dark" ? "dark" : "light");
  const customSkins = ref<UISkin[]>([]);
  const skinLoading = ref(false);
  const skinError = ref("");

  function loadCustomSkins() {
    try {
      const saved = localStorage.getItem("deep-ide-custom-skins");
      if (saved) {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed)) customSkins.value = parsed;
      }
    } catch (_) { customSkins.value = []; }
  }
  function saveCustomSkins() {
    localStorage.setItem("deep-ide-custom-skins", JSON.stringify(customSkins.value));
  }
  const allSkins = computed<UISkin[]>(() => [...BUILTIN_SKINS, ...customSkins.value]);

  function findSkin(id: string): UISkin | null {
    if (!id) return null;
    return allSkins.value.find(s => s.id === id) || null;
  }
  const activeSkinPortrait = computed<string>(() => findSkin(uiSkinId.value)?.portrait || "");
  function applyUISkin(id: string, variant?: "light" | "dark") {
    uiSkinId.value = id;
    if (variant) uiSkinVariant.value = variant;
    localStorage.setItem("uiSkinId", id);
    localStorage.setItem("uiSkinVariant", uiSkinVariant.value);
    const skin = findSkin(id);
    const key = `${id}@${uiSkinVariant.value}`;
    applyUISkinCss(skinCssFor(skin, uiSkinVariant.value), key);
    // 预创建同一皮肤的另一种配色，来回切换零解析
    const otherVariant: "light" | "dark" = uiSkinVariant.value === "dark" ? "light" : "dark";
    prewarmUISkinCss(skinCssFor(skin, otherVariant), `${id}@${otherVariant}`);
  }
  async function addCustomSkinFromRepo(repoUrl: string) {
    const url = repoUrl.trim();
    if (!url) return;
    skinLoading.value = true;
    skinError.value = "";
    try {
      const result = await tauriAPI.fetchGithubSkin(url);
      const skin: UISkin = {
        id: "custom-" + Math.random().toString(36).slice(2, 8),
        name: result.name || result.repo.split("/").pop() || result.repo,
        desc: result.tagline || (result.palette ? `来自 ${result.repo}（DeepKing 调色板 ${result.file}）` : `来自 ${result.repo}（${result.file}）`),
        source: "custom",
        repo: result.repo,
        portrait: result.portrait || undefined,
      };
      // DeepKing 调色板 → 生成亮/暗两套映射到本应用界面的皮肤
      const converted = result.palette ? convertPaletteToSkin(result.css) : null;
      if (converted) {
        skin.cssLight = converted.light;
        skin.cssDark = converted.dark;
      } else {
        skin.cssLight = result.css;
        skin.cssDark = result.css;
      }
      customSkins.value.push(skin);
      saveCustomSkins();
      applyUISkin(skin.id, "light");
    } catch (e: any) {
      skinError.value = String(e);
      throw e;
    } finally {
      skinLoading.value = false;
    }
  }
  function removeCustomSkin(id: string) {
    customSkins.value = customSkins.value.filter(s => s.id !== id);
    saveCustomSkins();
    if (uiSkinId.value === id) applyUISkin("");
  }
  loadCustomSkins();
  applyUISkin(uiSkinId.value);

  // Persona 信息
  const personaInfo = ref<ModeInfo | null>(null);
  const personaLoading = ref(false);

  // Agent 列表
  const agents = ref<AgentDef[]>([]);

  // AI 对话
  const messages = ref<Message[]>([]);
  const isLoading = ref(false);
  const totalTokens = ref(0);
  const streamingContent = ref("");  // 流式响应当前累积内容

  // Agent Loop 工具调用追踪
  const toolCalls = ref<Array<{
    id: string;
    name: string;
    arguments: any;
    success?: boolean;
    output?: string;
    status: "pending" | "running" | "done" | "error";
  }>>([]);
  const agentIterations = ref(0);
  const agentMaxIterations = ref(0);
  const useTools = ref<boolean>(true); // 是否启用工具调用（Claude Code 模式）

  // 文件树
  const fileTree = ref<FileEntry[]>([]);
  const fileTreePath = ref<string>("");
  const selectedFile = ref<string>("");

  const displayMessages = computed(() => messages.value);

  // ─── 项目 ───
  function setProject(path: string) { currentProject.value = path; }
  async function openProject(path: string) {
    await tauriAPI.openProject(path);
    currentProject.value = path;
  }
  function closeProject() { currentProject.value = null; }

  // ─── 文件 ───
  async function loadFileTree(path: string) {
    try {
      const result = await tauriAPI.listDirectory(path, 3);
      fileTree.value = result.entries;
      fileTreePath.value = result.path;
    } catch (e: any) {
      console.error("Failed to load file tree:", e);
    }
  }

  // ─── AI 模式 ───
  async function switchMode(mode: string) {
    currentMode.value = mode;
    personaLoading.value = true;
    try {
      personaInfo.value = await tauriAPI.switchAIMode(mode);
    } catch (e: any) {
      addSystemMessage(`模式切换失败: ${e}`);
    } finally {
      personaLoading.value = false;
    }
  }

  async function loadAgents() {
    try { agents.value = await tauriAPI.listAgents(); }
    catch (e: any) { console.error("Failed to load agents:", e); }
  }

  async function configureApiKey(key: string) {
    apiKey.value = key;
    try {
      await tauriAPI.configureDeepSeek(key, baseUrl.value, model.value);
      addSystemMessage("DeepSeek API 连接成功");
    } catch (e: any) {
      addSystemMessage(`API 配置失败: ${e}`);
    }
  }

  // ─── 发送消息（流式）───
  async function sendMessageStream(content: string, contextPaths: string[] = []) {
    if (!content.trim()) return;
    if (!apiKey.value) {
      addSystemMessage("请先配置 DeepSeek API Key");
      return;
    }

    messages.value.push({ role: "user", content, type: "user" });
    isLoading.value = true;
    streamingContent.value = "";
    const history = messages.value.filter(m => m.role !== "system");

    // 添加占位消息，用于流式更新
    messages.value.push({ role: "assistant", content: "", type: "assistant" });
    const msgIndex = messages.value.length - 1;

    try {
      const result = await tauriAPI.sendAIMessageStream(currentMode.value, content, history, contextPaths);
      // 完成后以 invoke 返回值为准（后端先 emit ai-stream-done 再返回，直接读 streamingContent 有竞态）
      messages.value[msgIndex].content = (result as any)?.content || streamingContent.value;
      streamingContent.value = "";
    } catch (e: any) {
      messages.value[msgIndex].content = `错误: ${e}`;
    } finally {
      isLoading.value = false;
    }
  }

  // ─── 发送消息（带 9 个工具的 Agent Loop）───
  async function sendMessageWithTools(content: string, contextPaths: string[] = [], workingDir?: string) {
    if (!content.trim()) return;
    if (!apiKey.value) {
      addSystemMessage("请先配置 DeepSeek API Key");
      return;
    }

    messages.value.push({ role: "user", content, type: "user" });
    isLoading.value = true;
    streamingContent.value = "";
    toolCalls.value = [];
    agentIterations.value = 0;
    agentMaxIterations.value = 0;
    // 去掉刚 push 的这条 user 消息，后端会单独把 user_message 作为第一条消息
    const history = messages.value.filter(m => m.role !== "system").slice(0, -1);

    // 添加占位消息
    messages.value.push({ role: "assistant", content: "🛠 工具调用中...\n", type: "assistant" });
    const msgIndex = messages.value.length - 1;
    let accumulatedText = "";

    function updateAssistantContent() {
      messages.value[msgIndex].content = `🛠 [${agentIterations.value}/${agentMaxIterations.value} 步 | 已调 ${toolCalls.value.length} 个工具]\n\n${accumulatedText}`;
    }

    // 非代码生成请求：要求 AI 把结果写成 Markdown 文件
    const codeGenPatterns = [
      /代码/, /code/, /python|py\b/, /javascript|js\b/, /typescript|ts\b/,
      /\bjava\b/, /c\+\+|cpp/, /rust|go\b|php|ruby|swift|kotlin/,
      /写.*程序/, /写.*脚本/, /生成.*代码/, /实现.*功能/, /编写/,
      /函数|class|接口|\bapi\b/, /\bprogram|\bscript/
    ];
    const isCodeRequest = codeGenPatterns.some(p => p.test(content.toLowerCase()));
    const requestContent = isCodeRequest
      ? content
      : `${content}\n\n[System] 本次请求不涉及代码生成。请把回答整理成 Markdown 文档并保存到工作区，文件名要反映主题。最终回复中只给出文件路径和简要说明，不要输出大段正文。`;

    try {
      // 订阅事件，实时更新 toolCalls 状态
      const { listen } = await import("@tauri-apps/api/event");
      const unlisten = await listen("ai-agent-event", (event: any) => {
        const ev = event.payload;
        if (!ev || !ev.kind) return;
        const k = ev.kind;
        if (k.type === "started") {
          agentMaxIterations.value = k.max_iterations;
        } else if (k.type === "iteration") {
          agentIterations.value = k.current;
          updateAssistantContent();
        } else if (k.type === "tool_call_requested") {
          toolCalls.value.push({
            id: k.id, name: k.name, arguments: k.arguments,
            status: "running"
          });
        } else if (k.type === "tool_call_executed") {
          const tc = toolCalls.value.find(t => t.id === k.id);
          if (tc) {
            tc.success = k.success;
            tc.output = k.output;
            tc.status = k.success ? "done" : "error";
          }
        } else if (k.type === "assistant_text") {
          accumulatedText += k.content;
          updateAssistantContent();
        } else if (k.type === "done") {
          messages.value[msgIndex].content = accumulatedText || k.content;
        } else if (k.type === "error") {
          messages.value[msgIndex].content = `❌ 错误: ${k.message}`;
        } else if (k.type === "file_changed") {
          // 工具改了文件，刷新文件树
          if (currentProject.value) {
            loadFileTree(currentProject.value);
          }
        }
      });

      try {
        const wd = workingDir || currentProject.value || undefined;
        const result = await tauriAPI.sendAIMessageWithTools(currentMode.value, requestContent, history, contextPaths, wd);
        messages.value[msgIndex].content = messages.value[msgIndex].content || result.content;
        addSystemMessage(`✅ Agent Loop 完成: ${result.total_iterations} 步, ${result.total_tool_calls} 个工具调用`);
      } finally {
        unlisten();
      }
    } catch (e: any) {
      messages.value[msgIndex].content = `❌ 错误: ${e}`;
    } finally {
      isLoading.value = false;
    }
  }

  function appendStreamToken(token: string) {
    streamingContent.value += token;
    // 更新最后一条 assistant 消息
    const msgs = messages.value;
    for (let i = msgs.length - 1; i >= 0; i--) {
      if (msgs[i].role === "assistant") {
        msgs[i].content = streamingContent.value;
        break;
      }
    }
  }

  /** ai-stream-done 事件兜底：把完整内容写入占位消息（幂等） */
  function finishStream(content: string) {
    if (!content) return;
    streamingContent.value = "";
    const msgs = messages.value;
    for (let i = msgs.length - 1; i >= 0; i--) {
      if (msgs[i].role === "assistant" && !msgs[i].content) {
        msgs[i].content = content;
        return;
      }
    }
  }

  // ─── 发送消息（普通 / Agent）───
  async function sendMessage(content: string, contextPaths: string[] = []) {
    if (!content.trim()) return;
    if (!apiKey.value) {
      addSystemMessage("请先配置 DeepSeek API Key");
      return;
    }

    messages.value.push({ role: "user", content, type: "user" });
    isLoading.value = true;

    try {
      let resp;
      const history = messages.value.filter(m => m.role !== "system");

      if (currentAgent.value) {
        resp = await tauriAPI.sendAgentMessage(currentAgent.value, currentMode.value, content, history);
      } else {
        resp = await tauriAPI.sendAIMessage(currentMode.value, content, history, contextPaths);
      }

      messages.value.push(resp.message);
      totalTokens.value += resp.usage.total_tokens;
    } catch (e: any) {
      messages.value.push({ role: "assistant", content: `错误: ${e}` });
    } finally {
      isLoading.value = false;
    }
  }

  function addSystemMessage(content: string) {
    messages.value.push({ role: "system", content, type: "system" });
  }

  function clearMessages() {
    messages.value = [];
    totalTokens.value = 0;
  }

  function setEditorTheme(theme: EditorTheme) {
    editorTheme.value = theme;
    localStorage.setItem("editorTheme", theme);
  }

  // ─── 安全检查 ───
  async function checkSafety(content: string) {
    try {
      const results = await tauriAPI.runSafetyCheck(content);
      for (const r of results) {
        if (r.triggered) {
          addSystemMessage(`${r.action === 'block' ? '🚫' : r.action === 'warn' ? '⚠️' : '🔍'} ${r.message}`);
        }
      }
    } catch (e: any) {
      console.error("Safety check failed:", e);
    }
  }

  return {
    currentProject, currentMode, currentAgent,
    apiKey, baseUrl, model,
    personaInfo, personaLoading, agents,
    messages, isLoading, totalTokens, displayMessages, streamingContent,
    fileTree, fileTreePath, selectedFile,
    editorTheme,
    setProject, openProject, closeProject,
    loadFileTree,
    switchMode, loadAgents, configureApiKey,
    sendMessage, sendMessageStream, sendMessageWithTools, appendStreamToken, finishStream, addSystemMessage, clearMessages,
    toolCalls, agentIterations, agentMaxIterations, useTools,
    setEditorTheme,
    checkSafety,
    uiSkinId, uiSkinVariant, customSkins, allSkins, skinLoading, skinError, activeSkinPortrait,
    applyUISkin, addCustomSkinFromRepo, removeCustomSkin,
  };
});

// ─────────────────────────────────────────────────────────────
// UI Skin（界面皮肤）— 照搬 DeepKing 的「界面皮肤 / UI Skin」接口
// 皮肤覆盖文件树、编辑区、AI 区域等整个应用界面。
// 内置：默认（原生）、GitHub 深色（亮/暗两套配色，与编辑器代码主题呼应）
// 自定义：从 GitHub 仓库拉取 CSS（"转换并添加"），持久化在 localStorage。
// ─────────────────────────────────────────────────────────────

export interface UISkin {
  id: string;
  name: string;
  desc: string;
  source: "builtin" | "custom";
  repo?: string;
  cssLight?: string;
  cssDark?: string;
  css?: string; // 单产物自定义皮肤
  portrait?: string; // 立绘 data URL
}

/** GitHub 官方配色（亮） */
const GITHUB_LIGHT_CSS = `
.app-container { background:#f6f8fa !important; color:#24292f; }
.app-container ::-webkit-scrollbar-thumb { background:#c9d1d9 !important; }
.editor-header { background:#ffffff; border-bottom:1px solid #d0d7de; }
.editor-header select, .editor-header input, .editor-header .header-title { color:#24292f; }
.dropdown-menu { background:#ffffff; border:1px solid #d0d7de; box-shadow:0 8px 24px rgba(31,35,40,.12); }
.dropdown-item { color:#24292f; }
.dropdown-item:hover { background:#f3f4f6; }
.dropdown-divider { background:#d0d7de; }
.file-explorer { background:#f6f8fa; border-right:1px solid #d0d7de; }
.file-explorer-header { background:#ffffff; border-bottom:1px solid #d0d7de; }
.file-tree { color:#24292f; }
.file-item { color:#24292f; }
.file-item:hover { background:#eaeef2; }
.file-item.selected { background:#ddf4ff; color:#0969da; }
.file-item.open { background:#fff8c5; }
.file-item.open:hover { background:#f7e884; }
.file-item .expand-icon { color:#6e7781; }
.tabs-bar { background:#f6f8fa; border-bottom:1px solid #d0d7de; }
.tab:not(.active) { background:#eaeef2; color:#57606a; border-top:2px solid transparent; }
.tab.active { background:#ffffff; color:#24292f; border-top:2px solid #0969da; }
.tab-close:hover { background:#cf222e; color:#fff; }
.editor-main-content { background:#ffffff; }
.editor-empty-title { color:#57606a; }
.editor-empty-desc { color:#8c959f; }
.terminal-panel { background:#ffffff; border-top:1px solid #d0d7de; }
.terminal-header { background:#f6f8fa; border-bottom:1px solid #d0d7de; }
.terminal-action-btn { background:#ffffff; color:#57606a; border:1px solid #d0d7de; }
.terminal-action-btn:hover { background:#eaeef2; }
.terminal-close-btn { color:#6e7781; }
.terminal-content { background:#ffffff; color:#24292f; }
.ai-panel { background:#f6f8fa; border-left:1px solid #d0d7de; }
.ai-panel-tabs { background:#ffffff; border-bottom:1px solid #d0d7de; }
.ai-tab { color:#6e7781; }
.ai-tab.active { color:#24292f; border-bottom-color:#0969da; }
.ai-message { background:#ffffff; border:1px solid #d0d7de; color:#24292f; }
.ai-input-area { border-top:1px solid #d0d7de; }
.ai-input-area textarea { background:#ffffff; color:#24292f; border:1px solid #d0d7de; }
.ai-input-area textarea:focus { border-color:#0969da; }
.ai-send-row select { background:#ffffff; color:#24292f; border:1px solid #d0d7de; }
.ai-config-btn { border:1px solid #d0d7de; color:#57606a; }
.ai-config-btn:hover { color:#24292f; border-color:#0969da; }
.ai-tools-toggle { color:#57606a; border:1px solid #d0d7de; }
.ai-tools-toggle:hover { color:#24292f; border-color:#0969da; }
.ai-send-btn { background:#1f883d; }
.ai-send-btn:hover { background:#1a7f37; }
.ai-send-btn:disabled { background:#afb8c1; color:#fff; }
.ai-context-chip { background:#ddf4ff; color:#0969da; border:1px solid #54aeff66; }
.ai-context-add-btn { border:1px solid #d0d7de; color:#57606a; }
.ai-context-add-btn:hover { border-color:#0969da; color:#0969da; }
.persona-info-bar { background:#ffffff; border-top:1px solid #d0d7de; }
.persona-badge { background:#0969da; color:#fff; }
.persona-detail { color:#6e7781; }
.plugin-item { background:#ffffff; border:1px solid #d0d7de !important; }
.plugin-name { color:#24292f; }
.plugin-desc { color:#57606a; }
.tool-detail-pre { background:#f6f8fa; color:#24292f; border:1px solid #d0d7de; }
.tool-detail-pre-ok { color:#116329; }
.tool-detail-pre-err { color:#cf222e; }
.tool-detail-line { color:#6e7781; }
.modal-overlay { background:rgba(31,35,40,0.45); }
.modal-box { background:#ffffff; color:#24292f; box-shadow:0 8px 24px rgba(31,35,40,.2); }
.modal-header { border-bottom:1px solid #d0d7de; }
.modal-header h3 { color:#24292f; }
.modal-close { color:#6e7781; }
.modal-close:hover { background:#eaeef2; color:#24292f; }
.modal-box input, .modal-box select, .modal-box textarea, .form-group input, .form-group select, .form-group textarea {
  background:#ffffff; border:1px solid #d0d7de; color:#24292f; }
.form-group label { color:#57606a; }
.btn-primary { background:#1f883d; color:#fff; }
.btn-primary:hover { background:#1a7f37; }
.btn-primary:disabled { background:#afb8c1; }
.btn-secondary { background:#ffffff; color:#57606a; border:1px solid #d0d7de; }
.btn-secondary:hover { border-color:#8c959f; color:#24292f; }
.context-menu { background:#ffffff; border:1px solid #d0d7de; box-shadow:0 8px 24px rgba(31,35,40,.12); }
.context-item { color:#24292f; }
.context-item:hover { background:#f3f4f6; }
.context-divider { background:#d0d7de; }
.file-picker-box { background:#ffffff; }
.file-picker-header span { color:#24292f; }
.file-picker-item { color:#24292f; }
.file-picker-item:hover { background:#f3f4f6; }
.file-picker-item.selected { background:#ddf4ff; color:#0969da; }
.marketplace-search input { background:#ffffff; color:#24292f; border:1px solid #d0d7de; }
.marketplace-tab { color:#6e7781; }
.marketplace-tab.active { color:#24292f; border-bottom-color:#0969da; }
.marketplace-empty, .marketplace-loading { color:#6e7781; }
.homepage { background:#f6f8fa !important; }
.homepage-header h1 { color:#24292f !important; }
.homepage-header p { color:#57606a !important; }
.homepage .search-box input { background:#ffffff !important; border-color:#d0d7de !important; color:#24292f !important; }
.homepage .action-button { background:#ffffff !important; border-color:#d0d7de !important; color:#57606a !important; }
.homepage .action-button:hover { border-color:#0969da !important; }
.homepage .action-button .icon { color:#6e7781 !important; }
.sub-page { background:#f6f8fa !important; }
.sub-header { background:#ffffff !important; border-bottom:1px solid #d0d7de; }
.sub-header h2 { color:#24292f !important; }
.sub-form { background:transparent; }
.sub-form input { background:#ffffff; color:#24292f; border:1px solid #d0d7de; }
.sub-form label { color:#57606a; }
`;

/** GitHub 官方配色（暗）— 覆盖文件树、编辑区、AI 区域 */
const GITHUB_DARK_CSS = `
.app-container { background:#0d1117 !important; color:#e6edf3; }
.app-container ::-webkit-scrollbar { width:10px; height:10px; }
.app-container ::-webkit-scrollbar-thumb { background:#30363d !important; border-radius:5px; }
.app-container ::-webkit-scrollbar-track { background:transparent !important; }
.editor-header { background:#161b22; border-bottom:1px solid #30363d; color:#e6edf3; }
.editor-header select, .editor-header input, .editor-header button { color:#e6edf3; }
.editor-header select { background:#0d1117; border:1px solid #30363d; }
.editor-header .header-title { color:#e6edf3; }
.dropdown-menu { background:#161b22; border:1px solid #30363d; box-shadow:0 8px 24px rgba(1,4,9,.6); }
.dropdown-item { color:#e6edf3; }
.dropdown-item:hover { background:#21262d; }
.dropdown-divider { background:#30363d; }
.file-explorer { background:#0d1117; border-right:1px solid #30363d; }
.file-explorer-header { background:#161b22; border-bottom:1px solid #30363d; color:#e6edf3; }
.file-tree { color:#c9d1d9; }
.file-item { color:#c9d1d9; }
.file-item:hover { background:#161b22; }
.file-item.selected { background:#1f6feb; color:#ffffff; }
.file-item.open { background:#21262d; }
.file-item.open:hover { background:#30363d; }
.file-item .expand-icon { color:#8b949e; }
.tabs-bar { background:#010409; border-bottom:1px solid #30363d; }
.tab:not(.active) { background:#161b22; color:#8b949e; border-top:2px solid transparent; }
.tab.active { background:#0d1117; color:#e6edf3; border-top:2px solid #58a6ff; }
.tab-close { color:#8b949e; }
.tab-close:hover { background:#f85149; color:#fff; }
.editor-main-content { background:#0d1117; }
.companion-toggle { background:#161b22; border-color:#30363d; color:#8b949e; }
.companion-toggle:hover { color:#e6edf3; border-color:#58a6ff; }
.editor-empty-title { color:#c9d1d9; }
.editor-empty-desc { color:#8b949e; }
.editor-empty-icon { opacity:.5; }
.terminal-panel { background:#010409; border-top:1px solid #30363d; }
.terminal-header { background:#161b22; border-bottom:1px solid #30363d; color:#e6edf3; }
.terminal-action-btn { background:#21262d; color:#c9d1d9; border:1px solid #30363d; }
.terminal-action-btn:hover { background:#30363d; color:#e6edf3; }
.terminal-close-btn { color:#8b949e; }
.terminal-close-btn:hover { color:#e6edf3; }
.terminal-content { background:#010409; color:#e6edf3; }
.ai-panel { background:#0d1117; border-left:1px solid #30363d; }
.ai-panel-tabs { background:#161b22; border-bottom:1px solid #30363d; }
.ai-tab { color:#8b949e; }
.ai-tab.active { color:#e6edf3; border-bottom-color:#58a6ff; }
.ai-message { background:#161b22; border:1px solid #30363d; color:#e6edf3; }
.ai-input-area { border-top:1px solid #30363d; }
.ai-input-area textarea { background:#010409; color:#e6edf3; border:1px solid #30363d; }
.ai-input-area textarea:focus { border-color:#58a6ff; }
.ai-send-row select { background:#161b22; color:#e6edf3; border:1px solid #30363d; }
.ai-config-btn { border:1px solid #30363d; color:#c9d1d9; }
.ai-config-btn:hover { color:#e6edf3; border-color:#58a6ff; }
.ai-tools-toggle { color:#c9d1d9; border:1px solid #30363d; }
.ai-tools-toggle:hover { color:#e6edf3; border-color:#58a6ff; }
.ai-send-btn { background:#238636; }
.ai-send-btn:hover { background:#2ea043; }
.ai-send-btn:disabled { background:#21262d; color:#8b949e; }
.ai-context-chip { background:#21262d; color:#79c0ff; border:1px solid #30363d; }
.ai-context-chip .chip-remove { color:#8b949e; }
.ai-context-add-btn { border:1px solid #30363d; color:#c9d1d9; }
.ai-context-add-btn:hover { border-color:#58a6ff; color:#58a6ff; }
.persona-info-bar { background:#161b22; border-top:1px solid #30363d; }
.persona-badge { background:#1f6feb; color:#fff; }
.persona-detail { color:#8b949e; }
.plugin-item { background:#161b22; border:1px solid #30363d !important; }
.plugin-name { color:#e6edf3; }
.plugin-desc { color:#8b949e; }
.tool-detail-pre { background:#010409; color:#c9d1d9; border:1px solid #30363d; }
.tool-detail-pre-ok { color:#7ee787; }
.tool-detail-pre-err { color:#f85149; }
.tool-detail-line { color:#8b949e; }
.modal-overlay { background:rgba(1,4,9,0.7); }
.modal-box { background:#161b22; color:#e6edf3; box-shadow:0 8px 24px rgba(1,4,9,.6); }
.modal-header { border-bottom:1px solid #30363d; }
.modal-header h3 { color:#e6edf3; }
.modal-close { color:#8b949e; }
.modal-close:hover { background:#30363d; color:#e6edf3; }
.modal-box input, .modal-box select, .modal-box textarea, .form-group input, .form-group select, .form-group textarea {
  background:#010409; border:1px solid #30363d; color:#e6edf3; }
.form-group label { color:#c9d1d9; }
.btn-primary { background:#238636; color:#fff; }
.btn-primary:hover { background:#2ea043; }
.btn-primary:disabled { background:#21262d; color:#8b949e; }
.btn-secondary { background:#21262d; color:#e6edf3; border:1px solid #30363d; }
.btn-secondary:hover { border-color:#58a6ff; color:#e6edf3; }
.context-menu { background:#161b22; border:1px solid #30363d; box-shadow:0 8px 24px rgba(1,4,9,.5); }
.context-item { color:#e6edf3; }
.context-item:hover { background:#21262d; }
.context-divider { background:#30363d; }
.file-picker-box { background:#161b22; }
.file-picker-header span { color:#e6edf3; }
.file-picker-item { color:#c9d1d9; }
.file-picker-item:hover { background:#21262d; }
.file-picker-item.selected { background:#1f6feb; color:#fff; }
.marketplace-search input { background:#010409; color:#e6edf3; border:1px solid #30363d; }
.marketplace-tab { color:#8b949e; }
.marketplace-tab.active { color:#e6edf3; border-bottom-color:#58a6ff; }
.marketplace-empty, .marketplace-loading { color:#8b949e; }
.skin-item { background:#0d1117; border-color:#30363d; }
.skin-item:hover { border-color:#8b949e; }
.skin-item.active { border-color:#58a6ff; box-shadow:0 0 0 1px #58a6ff; }
.skin-name { color:#e6edf3; }
.skin-badge { color:#58a6ff; border-color:#58a6ff; background:#1f6feb33; }
.skin-desc { color:#8b949e; }
.skin-repo { color:#79c0ff; }
.skin-mode-btn { background:#21262d; border-color:#30363d; color:#c9d1d9; }
.skin-mode-btn:hover { border-color:#58a6ff; color:#e6edf3; }
.skin-mode-btn.on { background:#1f6feb; border-color:#1f6feb; color:#fff; }
.skin-add-row input { background:#010409; border-color:#30363d; color:#e6edf3; }
.skin-error { color:#f85149; }
.homepage { background:#0d1117 !important; }
.homepage-header h1 { color:#e6edf3 !important; }
.homepage-header p { color:#8b949e !important; }
.homepage .search-box input { background:#161b22 !important; border-color:#30363d !important; color:#e6edf3 !important; }
.homepage .action-button { background:#161b22 !important; border-color:#30363d !important; color:#c9d1d9 !important; }
.homepage .action-button:hover { border-color:#58a6ff !important; }
.homepage .action-button .icon { color:#8b949e !important; }
.sub-page { background:#0d1117 !important; }
.sub-header { background:#161b22 !important; border-bottom:1px solid #30363d; }
.sub-header h2 { color:#e6edf3 !important; }
.sub-form { background:transparent; }
.sub-form input { background:#010409; color:#e6edf3; border:1px solid #30363d; }
.sub-form label { color:#c9d1d9; }
`;

/** 内置皮肤 */
export const BUILTIN_SKINS: UISkin[] = [
  {
    id: "github",
    name: "GitHub 深色",
    desc: "GitHub 官方配色：纸白浅色 / 墨暗深色，蓝绿强调",
    source: "builtin",
    cssLight: GITHUB_LIGHT_CSS,
    cssDark: GITHUB_DARK_CSS,
  },
];

let styleEl: HTMLStyleElement | null = null;

/**
 * 样式缓存：每种皮肤配色只创建一次 <style>，切换时仅 toggle disabled，
 * 避免每次切换都重建 CSS 文本（重建会触发全页重新解析，造成卡顿）。
 */
const skinStyleCache = new Map<string, HTMLStyleElement>();
let currentSkinStyle: HTMLStyleElement | null = null;

function makeStyleEl(key: string, css: string, disabled: boolean): HTMLStyleElement {
  const el = document.createElement("style");
  el.id = "deep-ide-ui-skin-" + key.replace(/[^a-zA-Z0-9_-]/g, "_");
  el.textContent = css;
  el.disabled = disabled;
  document.head.appendChild(el);
  return el;
}

/** 预创建样式（disabled），切换时零解析 */
export function prewarmUISkinCss(css: string | null | undefined, key: string) {
  if (!css || skinStyleCache.has(key)) return;
  skinStyleCache.set(key, makeStyleEl(key, css, true));
}

/** 应用皮肤 CSS（null 恢复默认）；key = "皮肤id@配色" */
export function applyUISkinCss(css: string | null | undefined, key = "skin") {
  if (currentSkinStyle) {
    currentSkinStyle.disabled = true;
    currentSkinStyle = null;
  }
  if (!css) return;
  let el = skinStyleCache.get(key);
  if (!el) {
    el = makeStyleEl(key, css, false);
    skinStyleCache.set(key, el);
  }
  el.disabled = false;
  currentSkinStyle = el;
}

/** 根据皮肤定义取对应配色的 CSS */
export function skinCssFor(skin: UISkin | null | undefined, variant: "light" | "dark"): string | null | undefined {
  if (!skin) return null;
  if (skin.css) return skin.css;
  if (variant === "dark") return skin.cssDark || skin.cssLight;
  return skin.cssLight || skin.cssDark;
}

// ═══════════════════════════════════════════════════════════
// DeepKing 皮肤规范转换器：skin.json 调色板 CSS（:root 亮色 + .dark 暗色）
// → 生成映射到 Deep IDE 界面的皮肤 CSS（亮/暗两套）
// ═══════════════════════════════════════════════════════════

type Palette = Record<string, string>;

function extractVarBlock(css: string, patterns: RegExp[]): Palette | null {
  for (const re of patterns) {
    const m = css.match(re);
    if (m && m[1]) {
      const vars: Palette = {};
      const pairRe = /(--[\w-]+)\s*:\s*([^;]+);/g;
      let hit: RegExpExecArray | null;
      while ((hit = pairRe.exec(m[1]))) {
        vars[hit[1].trim()] = hit[2].trim();
      }
      if (Object.keys(vars).length > 0) return vars;
    }
  }
  return null;
}

/** 解析 DeepKing 调色板：亮色(:root) + 暗色(.dark / [data-theme=dark]) */
export function extractPalette(css: string): { light: Palette; dark: Palette } | null {
  const light = extractVarBlock(css, [/:root\s*\{([^}]*)\}/]);
  const dark = extractVarBlock(css, [
    /\.dark\s*\{([^}]*)\}/,
    /\[data-theme=["']?dark["']?\]\s*\{([^}]*)\}/,
  ]);
  if (!light) return null;
  return { light, dark: dark || {} };
}

/** DeepKing 调色板变量 → 界面皮肤 CSS（占位符模板） */
const SKIN_TEMPLATE = `
.app-container { background: {{bg-base}} !important; color: {{text}}; }
.app-container ::-webkit-scrollbar-thumb { background: {{border}} !important; }
.editor-header { background: {{layer2}}; border-bottom:1px solid {{border}}; }
.editor-header select, .editor-header input, .editor-header button { color: {{text}}; }
.dropdown-menu { background: {{layer2}}; border:1px solid {{border}}; box-shadow:0 8px 24px rgba(0,0,0,.25); }
.dropdown-item { color: {{text}}; }
.dropdown-item:hover { background: {{hover}}; }
.dropdown-divider { background: {{border}}; }
.file-explorer { background: {{bg-base}}; border-right:1px solid {{border}}; }
.file-explorer-header { background: {{layer2}}; border-bottom:1px solid {{border}}; }
.file-item { color: {{text}}; }
.file-item:hover { background: {{hover}}; }
.file-item.selected { background: {{active}}; color: {{text}}; }
.file-item.open { background: {{hover}}; }
.file-item.open:hover { background: {{border}}; }
.file-item .expand-icon { color: {{muted}}; }
.tabs-bar { background: {{bg-base}}; border-bottom:1px solid {{border}}; }
.tab:not(.active) { background: {{layer2}}; color: {{muted}}; border-top:2px solid transparent; }
.tab.active { background: {{bg-base}}; color: {{text}}; border-top:2px solid {{accent}}; }
.tab-close:hover { background: {{accent}}; color:#fff; }
.editor-main-content { background: {{bg-base}}; }
.companion-toggle { background: {{layer2}}; border-color: {{border}}; color: {{muted}}; }
.companion-toggle:hover { color: {{text}}; border-color: {{accent}}; }
.editor-empty-title { color: {{muted}}; }
.editor-empty-desc { color: {{muted}}; }
.terminal-panel { background: {{bg-base}}; border-top:1px solid {{border}}; }
.terminal-header { background: {{layer2}}; border-bottom:1px solid {{border}}; }
.terminal-action-btn { background: {{layer2}}; color: {{text}}; border:1px solid {{border}}; }
.terminal-action-btn:hover { background: {{hover}}; }
.terminal-close-btn { color: {{muted}}; }
.terminal-content { background: {{bg-base}}; color: {{text}}; }
.ai-panel { background: {{bg-base}}; border-left:1px solid {{border}}; }
.ai-panel-tabs { background: {{layer2}}; border-bottom:1px solid {{border}}; }
.ai-tab { color: {{muted}}; }
.ai-tab.active { color: {{text}}; border-bottom-color: {{accent}}; }
.ai-message { background: {{layer2}}; border:1px solid {{border}}; color: {{text}}; }
.ai-input-area { border-top:1px solid {{border}}; }
.ai-input-area textarea { background: {{layer3}}; color: {{text}}; border:1px solid {{border}}; }
.ai-input-area textarea:focus { border-color: {{accent}}; }
.ai-send-row select { background: {{layer2}}; color: {{text}}; border:1px solid {{border}}; }
.ai-config-btn { border:1px solid {{border}}; color: {{muted}}; }
.ai-config-btn:hover { color: {{text}}; border-color: {{accent}}; }
.ai-tools-toggle { color: {{muted}}; border:1px solid {{border}}; }
.ai-tools-toggle:hover { color: {{text}}; border-color: {{accent}}; }
.ai-send-btn { background: {{accent}}; color:#fff; }
.ai-send-btn:hover { filter:brightness(1.1); }
.ai-send-btn:disabled { background: {{border}}; color: {{muted}}; }
.ai-context-chip { background: {{hover}}; color: {{accent}}; border:1px solid {{border}}; }
.ai-context-add-btn { border:1px solid {{border}}; color: {{muted}}; }
.ai-context-add-btn:hover { border-color: {{accent}}; color: {{accent}}; }
.persona-info-bar { background: {{layer2}}; border-top:1px solid {{border}}; }
.persona-badge { background: {{accent}}; color:#fff; }
.persona-detail { color: {{muted}}; }
.plugin-item { background: {{layer2}}; border:1px solid {{border}} !important; }
.plugin-name { color: {{text}}; }
.plugin-desc { color: {{muted}}; }
.tool-detail-pre { background: {{layer3}}; color: {{text}}; border:1px solid {{border}}; }
.tool-detail-line { color: {{muted}}; }
.modal-overlay { background: rgba(0,0,0,.45); }
.modal-box { background: {{layer2}}; color: {{text}}; box-shadow:0 8px 24px rgba(0,0,0,.3); }
.modal-header { border-bottom:1px solid {{border}}; }
.modal-header h3 { color: {{text}}; }
.modal-close { color: {{muted}}; }
.modal-close:hover { background: {{hover}}; color: {{text}}; }
.modal-box input, .modal-box select, .modal-box textarea,
.form-group input, .form-group select, .form-group textarea {
  background: {{layer3}}; border:1px solid {{border}}; color: {{text}}; }
.form-group label { color: {{muted}}; }
.btn-primary { background: {{accent}}; color:#fff; }
.btn-secondary { background: {{layer2}}; color: {{text}}; border:1px solid {{border}}; }
.btn-secondary:hover { border-color: {{accent}}; }
.context-menu { background: {{layer2}}; border:1px solid {{border}}; box-shadow:0 8px 24px rgba(0,0,0,.25); }
.context-item { color: {{text}}; }
.context-item:hover { background: {{hover}}; }
.context-divider { background: {{border}}; }
.file-picker-box { background: {{layer2}}; }
.file-picker-header span { color: {{text}}; }
.file-picker-item { color: {{text}}; }
.file-picker-item:hover { background: {{hover}}; }
.file-picker-item.selected { background: {{active}}; color: {{text}}; }
.marketplace-search input { background: {{layer3}}; color: {{text}}; border:1px solid {{border}}; }
.marketplace-tab { color: {{muted}}; }
.marketplace-tab.active { color: {{text}}; border-bottom-color: {{accent}}; }
.marketplace-empty, .marketplace-loading { color: {{muted}}; }
.skin-item { background: {{bg-base}}; border-color: {{border}}; }
.skin-item:hover { border-color: {{muted}}; }
.skin-item.active { border-color: {{accent}}; box-shadow:0 0 0 1px {{accent}}; }
.skin-name { color: {{text}}; }
.skin-badge { color: {{accent}}; border-color: {{accent}}; background: {{hover}}; }
.skin-desc { color: {{muted}}; }
.skin-repo { color: {{accent}}; }
.skin-mode-btn { background: {{layer2}}; border-color: {{border}}; color: {{muted}}; }
.skin-mode-btn:hover { border-color: {{accent}}; color: {{text}}; }
.skin-mode-btn.on { background: {{accent}}; border-color: {{accent}}; color:#fff; }
.skin-add-row input { background: {{layer3}}; border-color: {{border}}; color: {{text}}; }
.skin-error { color:#e74c3c; }
.homepage { background: {{bg-base}} !important; }
.homepage-header h1 { color: {{text}} !important; }
.homepage-header p { color: {{muted}} !important; }
.homepage .search-box input { background: {{layer2}} !important; border-color: {{border}} !important; color: {{text}} !important; }
.homepage .action-button { background: {{layer2}} !important; border-color: {{border}} !important; color: {{muted}} !important; }
.homepage .action-button:hover { border-color: {{accent}} !important; }
.homepage .action-button .icon { color: {{muted}} !important; }
.sub-page { background: {{bg-base}} !important; }
.sub-header { background: {{layer2}} !important; border-bottom:1px solid {{border}}; }
.sub-header h2 { color: {{text}} !important; }
.sub-form input { background: {{layer3}}; color: {{text}}; border:1px solid {{border}}; }
.sub-form label { color: {{muted}}; }
`;

const LIGHT_FALLBACK: Palette = {
  "--bg-base": "#f6f8fa", "--text": "#24292f",
  "--bg-layer-2": "#ffffff", "--bg-layer-3": "#eaeef2",
  "--border": "#d0d7de", "--interactive-bg-hover-solid": "#eaeef2",
  "--interactive-bg-active": "#ddf4ff", "--label-secondary": "#57606a",
  "--brand-primary": "#0969da",
};
const DARK_FALLBACK: Palette = {
  "--bg-base": "#0d1117", "--text": "#e6edf3",
  "--bg-layer-2": "#161b22", "--bg-layer-3": "#010409",
  "--border": "#30363d", "--interactive-bg-hover-solid": "#21262d",
  "--interactive-bg-active": "#1f6feb", "--label-secondary": "#8b949e",
  "--brand-primary": "#58a6ff",
};

function paletteToCss(p: Palette, fallback: Palette): string {
  const v = (key: string) => p[key] || fallback[key] || "";
  return SKIN_TEMPLATE
    .split("{{bg-base}}").join(v("--bg-base"))
    .split("{{text}}").join(v("--text"))
    .split("{{layer2}}").join(v("--bg-layer-2"))
    .split("{{layer3}}").join(v("--bg-layer-3"))
    .split("{{border}}").join(v("--border"))
    .split("{{hover}}").join(v("--interactive-bg-hover-solid"))
    .split("{{active}}").join(v("--interactive-bg-active"))
    .split("{{muted}}").join(v("--label-secondary"))
    .split("{{accent}}").join(v("--brand-primary"));
}

/** DeepKing 调色板 CSS → { light, dark } 皮肤 CSS；不是调色板格式时返回 null */
export function convertPaletteToSkin(css: string): { light: string; dark: string } | null {
  const palette = extractPalette(css);
  if (!palette) return null;
  const light = paletteToCss(palette.light, LIGHT_FALLBACK);
  const dark = paletteToCss(Object.keys(palette.dark).length ? palette.dark : palette.light, DARK_FALLBACK);
  return { light, dark };
}

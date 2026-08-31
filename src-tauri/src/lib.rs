use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub mod commands;
pub mod ai;
pub mod cli;

pub use ai::{DeepSeekClient, PersonaLoader};

/// 文件条目（用于文件树）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: Option<Vec<FileEntry>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirListResult {
    pub entries: Vec<FileEntry>,
    pub path: String,
}

/// AI 模式
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AIMode {
    #[serde(rename = "deep-anth")]
    DeepAnth,
    #[serde(rename = "deep-oai")]
    DeepOAI,
    #[serde(rename = "deep-gem")]
    DeepGem,
    #[serde(rename = "deep-qwen")]
    DeepQwen,
    #[serde(rename = "deep-kimi")]
    DeepKimi,
}

impl AIMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AIMode::DeepAnth => "deep-anth",
            AIMode::DeepOAI => "deep-oai",
            AIMode::DeepGem => "deep-gem",
            AIMode::DeepQwen => "deep-qwen",
            AIMode::DeepKimi => "deep-kimi",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "deep-anth" => Some(AIMode::DeepAnth),
            "deep-oai" => Some(AIMode::DeepOAI),
            "deep-gem" => Some(AIMode::DeepGem),
            "deep-qwen" => Some(AIMode::DeepQwen),
            "deep-kimi" => Some(AIMode::DeepKimi),
            _ => None,
        }
    }
}

/// 获取 Personas 目录路径
pub fn get_personas_dir() -> PathBuf {
    // Tauri 打包后 resource dir 是 exe 所在目录的上一级
    // 开发时是 src-tauri/
    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // 从 target/debug/ 或 target/release/ 向上一级找 personas/
    // 打包安装后资源在 exe 旁边；用 ".." 引用的资源会被展平到 _up_/ 目录
    for _ in 0..4 {
        let personas = dir.join("personas");
        if personas.exists() {
            return personas;
        }
        let flat_personas = dir.join("_up_").join("personas");
        if flat_personas.exists() {
            return flat_personas;
        }
        dir = dir.parent().map(|p| p.to_path_buf()).unwrap_or(dir);
    }

    // fallback: 相对于 src-tauri 的上级目录
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("personas"))
        .unwrap_or_else(|| PathBuf::from("personas"))
}

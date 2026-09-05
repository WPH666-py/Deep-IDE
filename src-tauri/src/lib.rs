use serde::{Deserialize, Serialize};

pub mod commands;
pub mod ai;
pub mod cli;

pub use ai::{DeepSeekClient};

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

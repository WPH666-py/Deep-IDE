use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::sync::Arc;
use futures_util::StreamExt;

use crate::ai::tools::{ToolSchema, ToolCall};

/// DeepSeek API 客户端（唯一运行时 API 调用）

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    /// 工具定义列表（OpenAI 兼容）
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolSchema>>,
    /// 工具选择策略：auto / none / specific
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

/// 自定义反序列化：null → ""
fn null_to_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    #[serde(default, deserialize_with = "null_to_empty_string")]
    pub content: String,
    /// 助手消息中的 tool_calls
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// 工具结果消息的 tool_call_id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// 工具结果消息的 name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 消息类型（DeepSeek v4 要求，必须存在）
    #[serde(default)]
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatResponse {
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Usage,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Choice {
    pub message: Message,
    #[serde(default)]
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub index: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: u32,
    #[serde(default)]
    pub completion_tokens: u32,
    #[serde(default)]
    pub total_tokens: u32,
}

/// SSE 流式响应的 Delta 结构
#[derive(Debug, Deserialize, Default)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    role: Option<String>,
    /// 流式 tool_calls 增量
    #[serde(default)]
    tool_calls: Option<Vec<StreamToolCallDelta>>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct StreamToolCallDelta {
    #[serde(default)]
    index: Option<u32>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    function: Option<StreamFunctionDelta>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct StreamFunctionDelta {
    #[serde(default)]
    name: Option<String>,
    /// arguments 是增量 JSON 字符串片段
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: StreamDelta,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    index: u32,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<StreamChoice>,
}

/// 流式回调：每收到一个 token 就调用一次
pub type StreamCallback = Box<dyn Fn(String) + Send>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.deepseek.com".to_string(),
            model: "deepseek-chat".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct DeepSeekClient {
    client: Client,
    config: Arc<Mutex<DeepSeekConfig>>,
}

impl DeepSeekClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            config: Arc::new(Mutex::new(DeepSeekConfig::default())),
        }
    }

    pub fn with_config(api_key: String, base_url: Option<String>, model: Option<String>) -> Self {
        let config = DeepSeekConfig {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.deepseek.com".to_string()),
            model: model.unwrap_or_else(|| "deepseek-chat".to_string()),
        };
        // We can't set the config via Arc<Mutex> synchronously easily,
        // so we return a client with the config already set.
        Self {
            client: Client::new(),
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// 更新配置
    pub async fn set_config(&self, api_key: String, base_url: Option<String>, model: Option<String>) {
        let mut cfg = self.config.lock().await;
        cfg.api_key = api_key;
        if let Some(url) = base_url { cfg.base_url = url; }
        if let Some(m) = model { cfg.model = m; }
    }

    /// 检查是否已配置 API Key
    pub async fn is_configured(&self) -> bool {
        let cfg = self.config.lock().await;
        !cfg.api_key.is_empty()
    }

    /// 唯一运行时 API 调用 — 发送组装后的 System Prompt + 对话历史
    pub async fn chat(
        &self,
        system_prompt: &str,
        messages: &[Message],
    ) -> Result<ChatResponse, String> {
        self.chat_with_tools(system_prompt, messages, None).await
    }

    /// 带工具的 chat（OpenAI 兼容）
    pub async fn chat_with_tools(
        &self,
        system_prompt: &str,
        messages: &[Message],
        tools: Option<&[ToolSchema]>,
    ) -> Result<ChatResponse, String> {
        let config = self.config.lock().await;
        if config.api_key.is_empty() {
            return Err("DeepSeek API Key not configured. Set it in Settings.".to_string());
        }

        let mut all_messages = vec![Message {
            role: "system".into(),
            content: system_prompt.to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            r#type: "system".into(),
        }];
        all_messages.extend_from_slice(messages);

        let req = ChatRequest {
            model: config.model.clone(),
            messages: all_messages,
            stream: false,
            // 工具调用场景下 arguments 可能很长（如长代码、长路径），给到 100K
            // 避免 arguments 被截断导致 JSON 不合法而解析失败
            max_tokens: Some(100000),
            temperature: Some(0.7),
            tools: tools.map(|t| t.to_vec()),
            tool_choice: if tools.is_some() { Some("auto".to_string()) } else { None },
        };

        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", config.base_url))
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, body));
        }

        let body = resp.text().await.map_err(|e| format!("Read response error: {}", e))?;

        let chat_resp: ChatResponse = serde_json::from_str(&body)
            .map_err(|e| {
                // 按字符边界截断预览，避免 UTF-8 多字节字符中间切片 panic
                let preview_end = body.char_indices().nth(500).map(|(i, _)| i).unwrap_or(body.len());
                format!("Parse error: {} | body: {}", e, &body[..preview_end])
            })?;

        Ok(chat_resp)
    }

    /// 流式版本 — 通过回调实时推送 token
    pub async fn chat_stream(
        &self,
        system_prompt: &str,
        messages: &[Message],
        on_token: impl Fn(String) + Send + 'static,
    ) -> Result<String, String> {
        self.chat_stream_with_tools(system_prompt, messages, None, on_token).await
    }

    /// 流式 + 工具版本
    pub async fn chat_stream_with_tools(
        &self,
        system_prompt: &str,
        messages: &[Message],
        tools: Option<&[ToolSchema]>,
        on_token: impl Fn(String) + Send + 'static,
    ) -> Result<String, String> {
        let config = self.config.lock().await;
        if config.api_key.is_empty() {
            return Err("DeepSeek API Key not configured.".to_string());
        }

        let mut all_messages = vec![Message {
            role: "system".into(),
            content: system_prompt.to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            r#type: "system".into(),
        }];
        all_messages.extend_from_slice(messages);

        let req = ChatRequest {
            model: config.model.clone(),
            messages: all_messages,
            stream: true,
            // 工具调用场景下 arguments 可能很长（如长代码、长路径），给到 100K
            // 避免 arguments 被截断导致 JSON 不合法而解析失败
            max_tokens: Some(100000),
            temperature: Some(0.7),
            tools: tools.map(|t| t.to_vec()),
            tool_choice: if tools.is_some() { Some("auto".to_string()) } else { None },
        };

        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", config.base_url))
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Stream error: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, body));
        }

        let mut stream = resp.bytes_stream();
        let mut full_content = String::new();
        // 跨 chunk 缓冲：只在拿到完整行后才解码，避免 UTF-8 多字节字符被切碎和行丢失
        let mut carry: Vec<u8> = Vec::new();
        let mut done = false;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Stream read error: {}", e))?;
            carry.extend_from_slice(&chunk);
            while let Some(pos) = carry.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = carry.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line_bytes);
                let line = line.trim();
                if line.is_empty() || !line.starts_with("data: ") {
                    continue;
                }
                let data = &line[6..]; // skip "data: "
                if data == "[DONE]" {
                    done = true;
                    break;
                }
                if let Ok(stream_chunk) = serde_json::from_str::<StreamChunk>(data) {
                    if let Some(choice) = stream_chunk.choices.first() {
                        if let Some(ref content) = choice.delta.content {
                            full_content.push_str(content);
                            on_token(content.clone());
                        }
                        // 流式 tool_calls 暂不处理（agent_loop 走非流式分支）
                    }
                }
            }
            if done {
                break;
            }
        }

        Ok(full_content)
    }

    /// 获取当前模型信息
    pub async fn get_model_info(&self) -> (String, String) {
        let cfg = self.config.lock().await;
        (cfg.model.clone(), cfg.base_url.clone())
    }

    /// 简单健康检查
    pub async fn health_check(&self) -> Result<String, String> {
        let config = self.config.lock().await;
        if config.api_key.is_empty() {
            return Err("API Key not configured".to_string());
        }
        let url = format!("{}/v1/models", config.base_url);
        match self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                Ok(format!("Connected to DeepSeek API"))
            }
            Ok(resp) => Err(format!("API returned: {}", resp.status())),
            Err(e) => Err(format!("Connection failed: {}", e)),
        }
    }
}

impl Default for DeepSeekClient {
    fn default() -> Self {
        Self::new()
    }
}

use tauri::{State, Emitter};
use std::path::PathBuf;
use std::sync::Arc;

use crate::ai::{
    run_agent_loop, AgentEvent, AgentLoopInput, AgentLoopOutput,
    ContextCompressor, CompressedMessage, ContextFile,
    DeepSeekClient, Message, PersonaLoader, PromptAssembler, TaskType,
};

/// ─── AI IPC 命令 ───

/// 列出所有可用 AI 模式
#[tauri::command]
pub fn list_ai_modes() -> Vec<serde_json::Value> {
    PersonaLoader::list_modes()
        .into_iter()
        .map(|(id, desc)| {
            serde_json::json!({
                "id": id,
                "name": match id {
                    "deep-anth" => "DeepAnth",
                    "deep-oai" => "DeepOAI",
                    "deep-gem" => "DeepGem",
                    "deep-qwen" => "DeepQwen",
                    "deep-kimi" => "DeepKimi",
                    _ => id,
                },
                "desc": desc,
            })
        })
        .collect()
}

/// 切换 AI 模式 → 加载对应 Persona，返回组装好的 System Prompt 预览
#[tauri::command]
pub fn switch_ai_mode(
    mode: String,
    persona_loader: State<'_, PersonaLoader>,
) -> Result<serde_json::Value, String> {
    let persona_ctx = persona_loader.load(&mode)?;

    let assembled = PromptAssembler::assemble(
        &persona_ctx,
        TaskType::CodeGeneration,
        &[],
    );

    Ok(serde_json::json!({
        "mode": mode,
        "name": persona_ctx.persona.meta.name,
        "provider": persona_ctx.persona.meta.provider,
        "emulated_model": persona_ctx.persona.meta.emulated_model,
        "coding_style": persona_ctx.persona.characteristics.coding_style,
        "review_rigor": persona_ctx.persona.characteristics.review_rigor,
        "architecture_first": persona_ctx.persona.characteristics.architecture_first,
        "best_for": persona_ctx.persona.tags.best_for,
        "system_prompt_preview": &assembled[..std::cmp::min(500, assembled.len())],
    }))
}

/// 配置 DeepSeek API Key
#[tauri::command]
pub async fn configure_deepseek(
    api_key: String,
    base_url: Option<String>,
    model: Option<String>,
    ds_client: State<'_, DeepSeekClient>,
) -> Result<String, String> {
    ds_client.set_config(api_key, base_url, model).await;
    Ok("DeepSeek API configured successfully".to_string())
}

/// 发送 AI 消息（使用当前 Persona + DeepSeek API，不调工具）
#[tauri::command]
pub async fn send_ai_message(
    mode: String,
    message: String,
    history: Vec<Message>,
    context_paths: Vec<String>,
    persona_loader: State<'_, PersonaLoader>,
    ds_client: State<'_, DeepSeekClient>,
) -> Result<serde_json::Value, String> {
    let persona_ctx = persona_loader.load(&mode)?;

    let context_files: Vec<ContextFile> = context_paths
        .iter()
        .map(|path| {
            let parsed = crate::ai::file_parser::parse_file(path);
            ContextFile {
                path: path.clone(),
                content: Some(parsed.content),
            }
        })
        .collect();

    let system_prompt = PromptAssembler::assemble(
        &persona_ctx,
        TaskType::CodeGeneration,
        &context_files,
    );

    let compressor = ContextCompressor::with_defaults();
    let compressed_messages: Vec<CompressedMessage> = history.iter().map(|m| CompressedMessage {
        role: m.role.clone(),
        content: m.content.clone(),
        estimated_tokens: ContextCompressor::estimate_tokens(&m.content),
    }).collect();
    let mut final_history: Vec<Message> = if compressor.needs_compression(&compressed_messages) {
        let compressed = compressor.compress(&compressed_messages);
        compressed.iter().map(|cm| Message {
            role: cm.role.clone(),
            content: cm.content.clone(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            r#type: cm.role.clone(),
        }).collect()
    } else {
        history
    };
    // 兼容前端旧消息：缺失 type 时默认用 role
    for m in &mut final_history {
        if m.r#type.is_empty() { m.r#type = m.role.clone(); }
    }

    let resp = ds_client.chat(&system_prompt, &final_history).await?;

    let raw_message = resp
        .choices
        .first()
        .map(|c| c.message.clone())
        .unwrap_or_else(|| Message {
            role: "assistant".into(),
            content: "[No response from model]".into(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            r#type: "assistant".into(),
        });

    // 剥离 tool_calls 等内部字段，返回前端需要的 role + content + type
    let safe_message = serde_json::json!({
        "role": raw_message.role,
        "content": raw_message.content,
        "type": raw_message.r#type,
    });

    Ok(serde_json::json!({
        "message": safe_message,
        "usage": {
            "prompt_tokens": resp.usage.prompt_tokens,
            "completion_tokens": resp.usage.completion_tokens,
            "total_tokens": resp.usage.total_tokens,
        },
        "mode": mode,
    }))
}

/// 流式发送 AI 消息 — 通过 Tauri events 实时推送 token 到前端
#[tauri::command]
pub async fn send_ai_message_stream(
    app: tauri::AppHandle,
    mode: String,
    message: String,
    history: Vec<Message>,
    context_paths: Vec<String>,
    persona_loader: State<'_, PersonaLoader>,
    ds_client: State<'_, DeepSeekClient>,
) -> Result<serde_json::Value, String> {
    let persona_ctx = persona_loader.load(&mode)?;

    let context_files: Vec<ContextFile> = context_paths
        .iter()
        .map(|path| {
            let parsed = crate::ai::file_parser::parse_file(path);
            ContextFile {
                path: path.clone(),
                content: Some(parsed.content),
            }
        })
        .collect();

    let system_prompt = PromptAssembler::assemble(
        &persona_ctx,
        TaskType::CodeGeneration,
        &context_files,
    );

    let compressor = ContextCompressor::with_defaults();
    let compressed_messages: Vec<CompressedMessage> = history.iter().map(|m| CompressedMessage {
        role: m.role.clone(),
        content: m.content.clone(),
        estimated_tokens: ContextCompressor::estimate_tokens(&m.content),
    }).collect();
    let mut final_history: Vec<Message> = if compressor.needs_compression(&compressed_messages) {
        let compressed = compressor.compress(&compressed_messages);
        compressed.iter().map(|cm| Message {
            role: cm.role.clone(),
            content: cm.content.clone(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            r#type: cm.role.clone(),
        }).collect()
    } else {
        history
    };
    // 兼容前端旧消息：缺失 type 时默认用 role
    for m in &mut final_history {
        if m.r#type.is_empty() { m.r#type = m.role.clone(); }
    }

    let app_handle = app.clone();
    let full_content = ds_client.chat_stream(&system_prompt, &final_history, move |token| {
        let _ = app_handle.emit("ai-stream-token", token);
    }).await?;

    let _ = app.emit("ai-stream-done", serde_json::json!({
        "content": full_content,
        "mode": mode,
    }).to_string());

    Ok(serde_json::json!({
        "content": full_content,
        "mode": mode,
    }))
}

/// ════════════════════════════════════════════════════════
/// 新增：Agent Loop 版本（带 9 个工具的真实工作流）
/// ════════════════════════════════════════════════════════

/// 同步版：执行完整 agent loop 一次性返回结果
#[tauri::command]
pub async fn send_ai_message_with_tools(
    app: tauri::AppHandle,
    mode: String,
    message: String,
    history: Vec<Message>,
    context_paths: Vec<String>,
    working_dir: Option<String>,
    persona_loader: State<'_, PersonaLoader>,
    ds_client: State<'_, DeepSeekClient>,
) -> Result<serde_json::Value, String> {
    let persona_ctx = persona_loader.load(&mode)?;

    let wd = PathBuf::from(working_dir.unwrap_or_else(|| ".".to_string()));

    // DeepSeekClient 本身可 Clone（内部 Arc 共享配置），这里 clone 一份独立的 owned 实例
    // 给 agent_loop 使用，避免 State 生命周期问题
    let ds_for_loop = ds_client.inner().clone();
    let deepseek_arc = Arc::new(ds_for_loop);

    // #region debug-point B:history-types
    for (i, h) in history.iter().enumerate() {
        eprintln!("[DEBUG] history[{}]: role={}, type={:?}", i, h.role, h.r#type);
        let debug_url = std::fs::read_to_string(".dbg/ai-400-bad-request.env")
            .ok()
            .and_then(|s| s.lines().find(|l| l.starts_with("DEBUG_SERVER_URL=")).map(|l| l[17..].trim().to_string()))
            .unwrap_or_else(|| "http://127.0.0.1:7777/event".to_string());
        let _ = reqwest::Client::new().post(&debug_url)
            .json(&serde_json::json!({
                "sessionId": "ai-400-bad-request",
                "runId": "post-fix",
                "hypothesisId": "B",
                "location": "commands/ai.rs:250",
                "msg": format!("[DEBUG] history[{}] type", i),
                "data": { "index": i, "role": h.role, "type": h.r#type, "content_len": h.content.len() },
            }))
            .send()
            .await;
    }
    // #endregion debug-point

    let input = AgentLoopInput {
        mode: mode.clone(),
        user_message: message,
        history,
        context_paths,
        working_dir: wd,
        deepseek: deepseek_arc,
        persona_ctx,
    };

    // 事件转发到 Tauri：每个 agent 事件触发 ai-agent-event
    let app_for_events = app.clone();
    let output: AgentLoopOutput = run_agent_loop(input, move |event: AgentEvent| {
        // 转为 serde_json::Value 再 emit，避免复杂枚举序列化问题
        if let Ok(ev_value) = serde_json::to_value(&event) {
            let _ = app_for_events.emit("ai-agent-event", ev_value);
        }
    }).await?;

    // 不返回 events 数组（已通过 Tauri 事件实时推送），只返回摘要
    // #region debug-point dp-2: 打印最终返回摘要
    eprintln!("[DEBUG] send_ai_message_with_tools OK: iterations={}, tool_calls={}", output.total_iterations, output.total_tool_calls);
    // #endregion debug-point
    Ok(serde_json::json!({
        "content": output.final_content,
        "total_iterations": output.total_iterations,
        "total_tool_calls": output.total_tool_calls,
        "mode": mode,
        "event_count": output.events.len(),
    }))
}

/// 检查 DeepSeek 连接健康状态
#[tauri::command]
pub async fn check_deepseek_health(
    ds_client: State<'_, DeepSeekClient>,
) -> Result<String, String> {
    ds_client.health_check().await
}

/// 解析上下文文件——前端可在文件选择器中预览解析结果
#[tauri::command]
pub fn parse_context_file(path: String) -> Result<serde_json::Value, String> {
    let parsed = crate::ai::file_parser::parse_file(&path);
    Ok(serde_json::json!({
        "path": parsed.path,
        "content": parsed.content,
        "format": parsed.format,
        "size_bytes": parsed.size_bytes,
        "is_binary": parsed.is_binary,
        "truncated": parsed.truncated,
        "success": parsed.success,
        "error": parsed.error,
    }))
}

//! CLI Backend - 通过 Cursor 官方 CLI 工具 `agent` 完成聊天请求
//!
//! 这是一个独立的后端模块，不依赖现有的 gRPC/protobuf 请求路径。
//! 通过调用 `agent --print --mode ask` 命令行工具来完成请求。
//!
//! # 启用方式
//! 设置环境变量 `CLI_BACKEND_ENABLED=true`
//!
//! # 前提条件
//! - 安装 Cursor CLI: `curl https://cursor.com/install -fsS | bash`
//! - 登录: `agent login` 或设置 `CURSOR_API_KEY`

use alloc::borrow::Cow;
use axum::{body::Body, response::Response};
use bytes::Bytes;
use http::{
    StatusCode,
    header::{CACHE_CONTROL, CONNECTION, CONTENT_TYPE, TRANSFER_ENCODING},
};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use crate::app::{
    constant::header::{CHUNKED, EVENT_STREAM, KEEP_ALIVE, NO_CACHE_REVALIDATE},
    route::GenericJson,
};

// ============================================================
// Configuration
// ============================================================

/// CLI 后端配置，从环境变量读取
pub struct CliConfig {
    pub enabled: bool,
    pub agent_bin: Cow<'static, str>,
    pub timeout_ms: u64,
    pub workspace: Cow<'static, str>,
    /// 是否使用 --mode ask（安全但可能导致 thinking 模型写文件而非输出文本）
    /// 设为 false 时不传 --mode，agent 有完整 tool access
    pub use_ask_mode: bool,
}

impl CliConfig {
    pub fn from_env() -> Self {
        use crate::common::utils::parse_from_env;
        Self {
            enabled: parse_from_env("CLI_BACKEND_ENABLED", false),
            agent_bin: parse_from_env("CLI_AGENT_BIN", "agent"),
            timeout_ms: parse_from_env("CLI_TIMEOUT_MS", 300_000u64),
            workspace: parse_from_env("CLI_WORKSPACE", "/tmp"),
            use_ask_mode: parse_from_env("CLI_USE_ASK_MODE", false),
        }
    }
}

use manually_init::ManuallyInit;

static CLI_CONFIG: ManuallyInit<CliConfig> = ManuallyInit::new();

/// 初始化 CLI 配置（在程序启动时调用一次）
pub fn init() {
    CLI_CONFIG.init(CliConfig::from_env());
}

/// 检查 CLI 后端是否启用
#[inline]
pub fn is_enabled() -> bool {
    CLI_CONFIG.get().enabled
}

#[inline]
fn config() -> &'static CliConfig {
    CLI_CONFIG.get()
}

// ============================================================
// Request / Response types (OpenAI compatible subset)
// ============================================================

#[derive(Deserialize)]
pub struct CliChatRequest {
    pub model: Option<String>,
    pub messages: Vec<CliMessage>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Deserialize)]
pub struct CliMessage {
    pub role: String,
    pub content: CliContent,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum CliContent {
    Text(String),
    Parts(Vec<CliContentPart>),
}

#[derive(Deserialize)]
pub struct CliContentPart {
    pub r#type: Option<String>,
    pub text: Option<String>,
}

impl CliContent {
    pub fn as_text(&self) -> String {
        match self {
            CliContent::Text(s) => s.clone(),
            CliContent::Parts(parts) => parts
                .iter()
                .filter(|p| p.r#type.as_deref() == Some("text"))
                .filter_map(|p| p.text.as_deref())
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

#[derive(Serialize)]
struct CliChatResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<CliChoice>,
    usage: CliUsage,
}

#[derive(Serialize)]
struct CliChoice {
    index: u32,
    message: CliResponseMessage,
    finish_reason: &'static str,
}

#[derive(Serialize)]
struct CliResponseMessage {
    role: &'static str,
    content: String,
    /// Thinking/reasoning content (OpenAI o1/o3 compatible format)
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_content: Option<String>,
}

#[derive(Serialize)]
struct CliUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Serialize)]
struct CliStreamChunk {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<CliStreamChoice>,
}

#[derive(Serialize)]
struct CliStreamChoice {
    index: u32,
    delta: CliStreamDelta,
    finish_reason: Option<&'static str>,
}

#[derive(Serialize)]
struct CliStreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    /// Thinking/reasoning content (OpenAI o1/o3 compatible format)
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_content: Option<String>,
}

#[derive(Serialize)]
struct CliErrorResponse {
    error: CliErrorInner,
}

#[derive(Serialize)]
struct CliErrorInner {
    message: String,
    r#type: &'static str,
    code: Option<&'static str>,
}

// ============================================================
// Prompt building
// ============================================================

fn build_prompt(messages: &[CliMessage]) -> String {
    let mut parts = Vec::with_capacity(messages.len());
    for msg in messages {
        let text = msg.content.as_text();
        match msg.role.as_str() {
            "system" | "developer" => {
                parts.push(format!("[System]\n{text}"));
            }
            "assistant" => {
                parts.push(format!("[Assistant]\n{text}"));
            }
            _ => {
                // user and any other role
                parts.push(format!("[User]\n{text}"));
            }
        }
    }
    parts.join("\n\n")
}

// ============================================================
// CLI execution
// ============================================================

struct CliSyncResult {
    content: String,
    thinking: Option<String>,
    usage: CliUsageInfo,
}

/// 构建 agent CLI 命令的公共参数
fn build_agent_command(agent_bin: &str, model: &str, workspace: &str, use_ask_mode: bool) -> Command {
    let mut cmd = Command::new(agent_bin);
    cmd.arg("--print");
    if use_ask_mode {
        cmd.arg("--mode").arg("ask");
    }
    cmd.arg("--model")
        .arg(model)
        .arg("--workspace")
        .arg(workspace)
        .arg("--trust");
    cmd
}

async fn run_cli_sync(model: &str, prompt: &str) -> Result<CliSyncResult, String> {
    let cfg = config();
    let agent_bin = cfg.agent_bin.to_string();
    let workspace = cfg.workspace.to_string();
    let timeout_ms = cfg.timeout_ms;
    let use_ask_mode = cfg.use_ask_mode;
    let model = model.to_string();
    let prompt = prompt.to_string();

    let result = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        tokio::task::spawn_blocking(move || {
            // Use stream-json format to capture thinking content
            let child = build_agent_command(&agent_bin, &model, &workspace, use_ask_mode)
                .arg("--stream-partial-output")
                .arg("--output-format")
                .arg("stream-json")
                .arg(&prompt)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to execute agent CLI: {e}"))?;

            let stdout = child.stdout.ok_or("Failed to capture stdout")?;
            let reader = BufReader::new(stdout);
            let mut parser = StreamParser::new();
            let mut thinking_parts = Vec::new();
            let mut content_parts = Vec::new();
            let mut usage = CliUsageInfo::default();

            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match parser.parse_line(trimmed) {
                    Some(StreamParseResult::ThinkingDelta(text)) => {
                        thinking_parts.push(text);
                    }
                    Some(StreamParseResult::ContentDelta(text)) => {
                        content_parts.push(text);
                    }
                    Some(StreamParseResult::FileContent { path, content }) => {
                        // Agent wrote code to a file — include it in the response
                        content_parts.push(format!("\n```\n// File: {path}\n{content}\n```\n"));
                    }
                    Some(StreamParseResult::Done(u)) => {
                        usage = u;
                        break;
                    }
                    _ => {}
                }
            }

            let thinking = if thinking_parts.is_empty() {
                None
            } else {
                Some(thinking_parts.join(""))
            };

            Ok(CliSyncResult {
                content: content_parts.join(""),
                thinking,
                usage,
            })
        }),
    )
    .await
    .map_err(|_| "CLI request timed out".to_string())?
    .map_err(|e| format!("Task join error: {e}"))?;

    result
}

/// CLI 流式输出解析器
///
/// CLI stream-json 格式:
/// - `{"type":"thinking","subtype":"delta","text":"..."}` — 思考增量(每条是独立delta)
/// - `{"type":"thinking","subtype":"completed"}` — 思考完成
/// - `{"type":"assistant","message":{"content":[{"type":"text","text":"..."}]}}` — 回答增量(每条独立delta)
/// - 最后一条 assistant 是完整累积文本(需要跳过)
/// - `{"type":"result","subtype":"success","usage":{...}}` — 结束
struct StreamParser {
    /// 已输出的 assistant 文本总长度(用于去重最后一条累积消息)
    assistant_len: usize,
    /// thinking 阶段是否完成
    thinking_done: bool,
}

/// 解析 CLI stream-json 中的 usage 信息
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CliUsageInfo {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
    #[serde(default)]
    cache_read_tokens: u32,
    #[serde(default)]
    cache_write_tokens: u32,
}

impl StreamParser {
    fn new() -> Self {
        Self {
            assistant_len: 0,
            thinking_done: false,
        }
    }

    /// 解析一行 CLI 输出
    fn parse_line(&mut self, line: &str) -> Option<StreamParseResult> {
        // 使用 serde_json::Value 灵活解析
        let obj: serde_json::Value = serde_json::from_str(line).ok()?;
        let line_type = obj.get("type")?.as_str()?;
        let subtype = obj.get("subtype").and_then(|v| v.as_str());

        match line_type {
            // === Thinking delta ===
            "thinking" if subtype == Some("delta") => {
                let text = obj.get("text")?.as_str()?;
                if text.is_empty() {
                    return None;
                }
                Some(StreamParseResult::ThinkingDelta(text.to_string()))
            }

            // === Thinking completed ===
            "thinking" if subtype == Some("completed") => {
                self.thinking_done = true;
                Some(StreamParseResult::ThinkingDone)
            }

            // === Tool call with file content ===
            // When agent writes code to a file, capture the content
            "tool_call" => {
                // Look for editToolCall.args.streamContent or editToolCall.args.content
                let tool_call = obj.get("tool_call")?;
                let edit = tool_call.get("editToolCall")?;
                let args = edit.get("args")?;

                // streamContent is the file content being written
                let content = args
                    .get("streamContent")
                    .or_else(|| args.get("content"))
                    .and_then(|v| v.as_str())?;

                if content.is_empty() {
                    return None;
                }

                // Only emit on "completed" to avoid duplicates (started + completed both have content)
                if subtype == Some("completed") {
                    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    self.assistant_len += content.len();
                    Some(StreamParseResult::FileContent {
                        path: path.to_string(),
                        content: content.to_string(),
                    })
                } else {
                    None
                }
            }

            // === Assistant content delta ===
            "assistant" => {
                let content = obj.get("message")?.get("content")?.as_array()?;
                let text: String = content
                    .iter()
                    .filter(|p| p.get("type").and_then(|t| t.as_str()) == Some("text"))
                    .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("");

                if text.is_empty() {
                    return None;
                }

                // 最后一条 assistant 消息是完整累积文本，需要跳过
                if text.len() >= self.assistant_len && self.assistant_len > 0 {
                    if text.len() > 100 && text.len() as f64 > self.assistant_len as f64 * 0.8 {
                        return None;
                    }
                }

                self.assistant_len += text.len();
                Some(StreamParseResult::ContentDelta(text))
            }

            // === Result (done) ===
            "result" if subtype == Some("success") => {
                let usage = obj
                    .get("usage")
                    .and_then(|u| serde_json::from_value::<CliUsageInfo>(u.clone()).ok())
                    .unwrap_or_default();
                Some(StreamParseResult::Done(usage))
            }

            _ => None,
        }
    }
}

enum StreamParseResult {
    /// Thinking content delta (from thinking models)
    ThinkingDelta(String),
    /// Thinking phase completed
    ThinkingDone,
    /// Assistant content delta
    ContentDelta(String),
    /// File content from tool_call (agent wrote code to a file)
    FileContent { path: String, content: String },
    /// Stream completed with usage info
    Done(CliUsageInfo),
}

// ============================================================
// HTTP Handler
// ============================================================

fn error_response(status: StatusCode, message: String) -> Response<Body> {
    let body = serde_json::to_string(&CliErrorResponse {
        error: CliErrorInner {
            message,
            r#type: "cli_backend_error",
            code: None,
        },
    })
    .unwrap_or_else(|_| r#"{"error":{"message":"internal error","type":"cli_backend_error"}}"#.to_string());

    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap()
}

/// 主处理函数: `/cli/chat/completions`
pub async fn handle_cli_chat_completions(
    GenericJson(request): GenericJson<CliChatRequest>,
) -> Response<Body> {
    if !is_enabled() {
        return error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "CLI backend is not enabled. Set CLI_BACKEND_ENABLED=true".to_string(),
        );
    }

    let model = request.model.as_deref().unwrap_or("auto");
    let prompt = build_prompt(&request.messages);

    if request.stream {
        handle_stream(model, &prompt).await
    } else {
        handle_sync(model, &prompt).await
    }
}

async fn handle_sync(model: &str, prompt: &str) -> Response<Body> {
    match run_cli_sync(model, prompt).await {
        Ok(result) => {
            let id = format!(
                "chatcmpl-cli-{}",
                uuid::Uuid::new_v4().as_simple()
            );
            let response = CliChatResponse {
                id,
                object: "chat.completion",
                created: crate::common::utils::now_secs(),
                model: model.to_string(),
                choices: vec![CliChoice {
                    index: 0,
                    message: CliResponseMessage {
                        role: "assistant",
                        content: result.content,
                        reasoning_content: result.thinking,
                    },
                    finish_reason: "stop",
                }],
                usage: CliUsage {
                    prompt_tokens: result.usage.input_tokens,
                    completion_tokens: result.usage.output_tokens,
                    total_tokens: result.usage.input_tokens + result.usage.output_tokens,
                },
            };

            let body = serde_json::to_string(&response).unwrap();
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap()
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

async fn handle_stream(model: &str, prompt: &str) -> Response<Body> {
    let cfg = config();
    let mut child = match build_agent_command(cfg.agent_bin.as_ref(), model, cfg.workspace.as_ref(), cfg.use_ask_mode)
        .arg("--stream-partial-output")
        .arg("--output-format")
        .arg("stream-json")
        .arg(prompt)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to start agent CLI: {e}"),
            );
        }
    };

    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to capture agent stdout".to_string(),
            );
        }
    };

    let id = format!(
        "chatcmpl-cli-{}",
        uuid::Uuid::new_v4().as_simple()
    );
    let created = crate::common::utils::now_secs();
    let model_owned = model.to_string();

    // Use a channel to bridge the CLI stdout reader to the HTTP response stream
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, core::convert::Infallible>>(32);

    // Spawn blocking thread to read CLI stdout line by line and send SSE chunks via channel
    tokio::task::spawn_blocking(move || {
        let reader = BufReader::new(stdout);
        let mut parser = StreamParser::new();

        // Helper: send a chunk, return false if client disconnected
        let send = |tx: &tokio::sync::mpsc::Sender<Result<Bytes, core::convert::Infallible>>,
                    chunk: &CliStreamChunk| -> bool {
            tx.blocking_send(Ok(Bytes::from(format!(
                "data: {}\n\n",
                __unwrap!(serde_json::to_string(chunk))
            ))))
            .is_ok()
        };

        // Send initial role chunk
        let initial = CliStreamChunk {
            id: id.clone(),
            object: "chat.completion.chunk",
            created,
            model: model_owned.clone(),
            choices: vec![CliStreamChoice {
                index: 0,
                delta: CliStreamDelta {
                    role: Some("assistant"),
                    content: None,
                    reasoning_content: None,
                },
                finish_reason: None,
            }],
        };
        if !send(&tx, &initial) {
            return;
        }

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match parser.parse_line(trimmed) {
                // Thinking delta → reasoning_content field
                Some(StreamParseResult::ThinkingDelta(text)) => {
                    let chunk = CliStreamChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk",
                        created,
                        model: model_owned.clone(),
                        choices: vec![CliStreamChoice {
                            index: 0,
                            delta: CliStreamDelta {
                                role: None,
                                content: None,
                                reasoning_content: Some(text),
                            },
                            finish_reason: None,
                        }],
                    };
                    if !send(&tx, &chunk) {
                        break;
                    }
                }

                // Thinking done → no special chunk needed, just continue
                Some(StreamParseResult::ThinkingDone) => {}

                // Content delta → content field
                Some(StreamParseResult::ContentDelta(text)) => {
                    let chunk = CliStreamChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk",
                        created,
                        model: model_owned.clone(),
                        choices: vec![CliStreamChoice {
                            index: 0,
                            delta: CliStreamDelta {
                                role: None,
                                content: Some(text),
                                reasoning_content: None,
                            },
                            finish_reason: None,
                        }],
                    };
                    if !send(&tx, &chunk) {
                        break;
                    }
                }

                // File content from tool_call → emit as content
                Some(StreamParseResult::FileContent { path, content }) => {
                    let text = format!("\n```\n// File: {path}\n{content}\n```\n");
                    let chunk = CliStreamChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk",
                        created,
                        model: model_owned.clone(),
                        choices: vec![CliStreamChoice {
                            index: 0,
                            delta: CliStreamDelta {
                                role: None,
                                content: Some(text),
                                reasoning_content: None,
                            },
                            finish_reason: None,
                        }],
                    };
                    if !send(&tx, &chunk) {
                        break;
                    }
                }

                // Done → send finish chunk
                Some(StreamParseResult::Done(_usage)) => {
                    break;
                }

                None => {}
            }
        }

        // Send finish chunk
        let finish = CliStreamChunk {
            id: id.clone(),
            object: "chat.completion.chunk",
            created,
            model: model_owned,
            choices: vec![CliStreamChoice {
                index: 0,
                delta: CliStreamDelta {
                    role: None,
                    content: None,
                    reasoning_content: None,
                },
                finish_reason: Some("stop"),
            }],
        };
        let _ = tx.blocking_send(Ok(Bytes::from(format!(
            "data: {}\n\n",
            __unwrap!(serde_json::to_string(&finish))
        ))));
        let _ = tx.blocking_send(Ok(Bytes::from("data: [DONE]\n\n")));

        // Wait for child to finish
        let _ = child.wait();
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, EVENT_STREAM)
        .header(CACHE_CONTROL, NO_CACHE_REVALIDATE)
        .header(CONNECTION, KEEP_ALIVE)
        .header(TRANSFER_ENCODING, CHUNKED)
        .body(Body::from_stream(stream))
        .unwrap()
}

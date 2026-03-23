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
}

impl CliConfig {
    pub fn from_env() -> Self {
        use crate::common::utils::parse_from_env;
        Self {
            enabled: parse_from_env("CLI_BACKEND_ENABLED", false),
            agent_bin: parse_from_env("CLI_AGENT_BIN", "agent"),
            timeout_ms: parse_from_env("CLI_TIMEOUT_MS", 300_000u64),
            workspace: parse_from_env("CLI_WORKSPACE", "/tmp"),
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

async fn run_cli_sync(model: &str, prompt: &str) -> Result<String, String> {
    let cfg = config();
    let agent_bin = cfg.agent_bin.to_string();
    let workspace = cfg.workspace.to_string();
    let timeout_ms = cfg.timeout_ms;
    let model = model.to_string();
    let prompt = prompt.to_string();

    let result = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        tokio::task::spawn_blocking(move || {
            let output = Command::new(&agent_bin)
                .arg("--print")
                .arg("--mode")
                .arg("ask")
                .arg("--model")
                .arg(&model)
                .arg("--workspace")
                .arg(&workspace)
                .arg("--trust")
                .arg("--output-format")
                .arg("text")
                .arg(&prompt)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| format!("Failed to execute agent CLI: {e}"))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "agent CLI exited with code {}: {stderr}",
                    output.status.code().unwrap_or(-1)
                ));
            }

            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }),
    )
    .await
    .map_err(|_| "CLI request timed out".to_string())?
    .map_err(|e| format!("Task join error: {e}"))?;

    result
}

/// CLI 流式输出解析器（去重逻辑）
struct StreamParser {
    accumulated: String,
}

impl StreamParser {
    fn new() -> Self {
        Self {
            accumulated: String::new(),
        }
    }

    /// 解析一行 CLI 输出，返回增量文本（如果有）
    fn parse_line(&mut self, line: &str) -> Option<StreamParseResult> {
        #[derive(Deserialize)]
        struct CliStreamLine {
            r#type: Option<String>,
            subtype: Option<String>,
            message: Option<CliStreamMessage>,
        }
        #[derive(Deserialize)]
        struct CliStreamMessage {
            content: Option<Vec<CliStreamContent>>,
        }
        #[derive(Deserialize)]
        struct CliStreamContent {
            r#type: Option<String>,
            text: Option<String>,
        }

        let obj: CliStreamLine = serde_json::from_str(line).ok()?;

        // Check for done signal
        if obj.r#type.as_deref() == Some("result") && obj.subtype.as_deref() == Some("success") {
            return Some(StreamParseResult::Done);
        }

        if obj.r#type.as_deref() != Some("assistant") {
            return None;
        }

        let content = obj.message?.content?;
        let text: String = content
            .iter()
            .filter(|p| p.r#type.as_deref() == Some("text"))
            .filter_map(|p| p.text.as_deref())
            .collect::<Vec<_>>()
            .join("");

        if text.is_empty() {
            return None;
        }

        // Deduplication: CLI sends accumulated text, we need only the delta
        if text == self.accumulated {
            return None;
        }

        if text.starts_with(&self.accumulated) && !self.accumulated.is_empty() {
            let delta = text[self.accumulated.len()..].to_string();
            self.accumulated = text;
            if delta.is_empty() {
                return None;
            }
            return Some(StreamParseResult::Delta(delta));
        }

        // New text that doesn't extend accumulated — just emit it
        self.accumulated.push_str(&text);
        Some(StreamParseResult::Delta(text))
    }
}

enum StreamParseResult {
    Delta(String),
    Done,
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
        Ok(content) => {
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
                        content,
                    },
                    finish_reason: "stop",
                }],
                usage: CliUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
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
    let mut child = match Command::new(cfg.agent_bin.as_ref())
        .arg("--print")
        .arg("--mode")
        .arg("ask")
        .arg("--model")
        .arg(model)
        .arg("--workspace")
        .arg(cfg.workspace.as_ref())
        .arg("--trust")
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
                },
                finish_reason: None,
            }],
        };
        let _ = tx.blocking_send(Ok(Bytes::from(format!(
            "data: {}\n\n",
            __unwrap!(serde_json::to_string(&initial))
        ))));

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
                Some(StreamParseResult::Delta(text)) => {
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
                            },
                            finish_reason: None,
                        }],
                    };
                    if tx
                        .blocking_send(Ok(Bytes::from(format!(
                            "data: {}\n\n",
                            __unwrap!(serde_json::to_string(&chunk))
                        ))))
                        .is_err()
                    {
                        break; // Client disconnected
                    }
                }
                Some(StreamParseResult::Done) => {
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

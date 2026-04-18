use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChatMessage,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingUsage {
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
    pub usage: Option<EmbeddingUsage>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

pub async fn chat_completion_with_base(base_url: &str, api_key: &str, request: &ChatRequest) -> Result<ChatResponse, String> {
    let client = Client::new();
    let url = format!("{base_url}/chat/completions");
    let mut builder = client.post(&url).json(request);
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }
    let resp = builder.send().await.map_err(|e| format!("Network error: {e}"))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("Read error: {e}"))?;

    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            return Err(format!("API error ({}): {}", status, err.error.message));
        }
        return Err(format!("API error ({}): {}", status, body));
    }

    serde_json::from_str(&body).map_err(|e| format!("Parse error: {e}"))
}

// ─── Streaming Chat Completion ──────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct StreamingRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    // Parsed from the SSE payload but unused today — kept so future code
    // can distinguish natural stops from length cutoffs in streaming.
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
    /// Emitted by reasoning/chain-of-thought models (e.g. via LM Studio's
    /// reasoning-model support). Normally we never surface this to the UI
    /// — but if a whole response produces ONLY reasoning and no content
    /// tokens (e.g. the model ran out of tokens while thinking), we fall
    /// back to it so the caller isn't stuck with an empty string.
    #[serde(default)]
    reasoning_content: Option<String>,
}

/// Stream a chat completion, emitting each token chunk as a Tauri event.
/// Returns the full assembled response text.
pub async fn chat_completion_stream(
    base_url: &str,
    api_key: &str,
    request: &StreamingRequest,
    app_handle: &tauri::AppHandle,
    event_name: &str,
) -> Result<String, String> {
    use futures_util::StreamExt;

    let client = Client::new();
    let url = format!("{base_url}/chat/completions");
    let mut builder = client.post(&url).json(request);
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }

    let resp = builder.send().await.map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            return Err(format!("API error ({}): {}", status, err.error.message));
        }
        return Err(format!("API error ({}): {}", status, body));
    }

    let mut full_text = String::new();
    let mut reasoning_text = String::new();
    let mut stream = resp.bytes_stream();

    let mut buffer = String::new();
    let mut raw_body = String::new();
    let mut sse_events_seen = 0usize;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream error: {e}"))?;
        let chunk_str = String::from_utf8_lossy(&chunk);
        raw_body.push_str(&chunk_str);
        buffer.push_str(&chunk_str);

        // Process complete SSE lines
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }
            if let Some(json_str) = line.strip_prefix("data: ") {
                sse_events_seen += 1;
                // LM Studio (and some other OpenAI-compatible servers) emit
                // error events mid-stream as `data: {"error":{"message":"..."}}`.
                // Detect those first and bail with the server's own message.
                if let Ok(err) = serde_json::from_str::<ApiError>(json_str) {
                    return Err(format!("Model error: {}", err.error.message));
                }
                if let Ok(parsed) = serde_json::from_str::<StreamChunk>(json_str) {
                    for choice in &parsed.choices {
                        if let Some(content) = &choice.delta.content {
                            full_text.push_str(content);
                            let _ = tauri::Emitter::emit(app_handle, event_name, content.clone());
                        }
                        if let Some(reasoning) = &choice.delta.reasoning_content {
                            // Accumulate silently; don't emit to UI.
                            reasoning_text.push_str(reasoning);
                        }
                    }
                }
            }
        }
    }

    // Reasoning-model fallback (chain-of-thought only, no content).
    if full_text.is_empty() && !reasoning_text.is_empty() {
        let _ = tauri::Emitter::emit(app_handle, event_name, reasoning_text.clone());
        return Ok(reasoning_text);
    }
    // Non-SSE fallback: some servers (or certain request configurations) return
    // a normal JSON chat-completion payload even when stream=true. Try parsing
    // the full accumulated body as a ChatResponse and pull message.content.
    if full_text.is_empty() {
        if let Ok(parsed) = serde_json::from_str::<ChatResponse>(raw_body.trim()) {
            if let Some(content) = parsed.choices.first().map(|c| c.message.content.clone()) {
                if !content.is_empty() {
                    let _ = tauri::Emitter::emit(app_handle, event_name, content.clone());
                    return Ok(content);
                }
            }
        }
    }
    // Fully empty response — surface a diagnostic error instead of silently
    // returning "". Include a short snippet of the raw body so the cause is
    // at least inspectable from the UI.
    if full_text.is_empty() {
        let snippet: String = raw_body.chars().take(400).collect();
        return Err(format!(
            "Empty response from model (parsed {sse_events_seen} SSE events, {} bytes received). First bytes: {}",
            raw_body.len(),
            if snippet.is_empty() { "(none)".to_string() } else { snippet },
        ));
    }
    Ok(full_text)
}

/// Streaming chat completion that does NOT emit Tauri events — the returned
/// String is the full assembled response. Use when you need streaming so
/// that cancelling the future closes the HTTP connection (halting the local
/// model's generation), but don't want any UI tokens to fire. Intended for
/// background work that shouldn't leak into the foreground chat UI.
pub async fn chat_completion_stream_silent(
    base_url: &str,
    api_key: &str,
    request: &StreamingRequest,
) -> Result<String, String> {
    use futures_util::StreamExt;

    let client = Client::new();
    let url = format!("{base_url}/chat/completions");
    let mut builder = client.post(&url).json(request);
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }
    let resp = builder.send().await.map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            return Err(format!("API error ({}): {}", status, err.error.message));
        }
        return Err(format!("API error ({}): {}", status, body));
    }

    let mut full_text = String::new();
    let mut reasoning_text = String::new();
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();
    let mut raw_body = String::new();
    let mut sse_events_seen = 0usize;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream error: {e}"))?;
        let chunk_str = String::from_utf8_lossy(&chunk);
        raw_body.push_str(&chunk_str);
        buffer.push_str(&chunk_str);
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();
            if line.is_empty() || line == "data: [DONE]" { continue; }
            if let Some(json_str) = line.strip_prefix("data: ") {
                sse_events_seen += 1;
                if let Ok(err) = serde_json::from_str::<ApiError>(json_str) {
                    return Err(format!("Model error: {}", err.error.message));
                }
                if let Ok(parsed) = serde_json::from_str::<StreamChunk>(json_str) {
                    for choice in &parsed.choices {
                        if let Some(content) = &choice.delta.content {
                            full_text.push_str(content);
                        }
                        if let Some(reasoning) = &choice.delta.reasoning_content {
                            reasoning_text.push_str(reasoning);
                        }
                    }
                }
            }
        }
    }
    if full_text.is_empty() && !reasoning_text.is_empty() {
        return Ok(reasoning_text);
    }
    if full_text.is_empty() {
        if let Ok(parsed) = serde_json::from_str::<ChatResponse>(raw_body.trim()) {
            if let Some(content) = parsed.choices.first().map(|c| c.message.content.clone()) {
                if !content.is_empty() { return Ok(content); }
            }
        }
    }
    if full_text.is_empty() {
        let snippet: String = raw_body.chars().take(400).collect();
        return Err(format!(
            "Empty response from model (parsed {sse_events_seen} SSE events, {} bytes received). First bytes: {}",
            raw_body.len(),
            if snippet.is_empty() { "(none)".to_string() } else { snippet },
        ));
    }
    Ok(full_text)
}

// ─── Image Generation ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ImageRequest {
    pub model: String,
    pub prompt: String,
    pub n: u32,
    pub size: String,
    pub quality: String,
    /// dall-e uses "response_format", gpt-image uses "output_format"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageResponse {
    pub data: Vec<ImageData>,
}

#[derive(Debug, Deserialize)]
pub struct ImageData {
    pub b64_json: Option<String>,
    /// gpt-image-1 returns the field as "b64" instead of "b64_json"
    pub b64: Option<String>,
}

impl ImageData {
    pub fn image_b64(&self) -> Option<&String> {
        self.b64_json.as_ref().or(self.b64.as_ref())
    }
}

pub async fn generate_image_with_base(base_url: &str, api_key: &str, request: &ImageRequest) -> Result<ImageResponse, String> {
    let debug_path = std::path::PathBuf::from("/tmp/world-chat-image-debug.log");

    let log_debug = |msg: &str| {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&debug_path) {
            let _ = writeln!(f, "[{}] {}", chrono::Utc::now().to_rfc3339(), msg);
        }
    };

    log_debug(&format!("REQUEST url={base_url}/images/generations model={} size={} quality={}", request.model, request.size, request.quality));
    log_debug(&format!("REQUEST body (prompt truncated): model={} n={} size={} quality={} prompt={:.200}", request.model, request.n, request.size, request.quality, request.prompt));

    let client = Client::new();
    let url = format!("{base_url}/images/generations");
    let mut builder = client.post(&url).json(request);
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }
    let resp = match builder.send().await {
        Ok(r) => r,
        Err(e) => {
            log_debug(&format!("NETWORK ERROR: {e}"));
            return Err(format!("Network error: {e}"));
        }
    };

    let status = resp.status();
    let body = resp.text().await.map_err(|e| {
        log_debug(&format!("READ ERROR: {e}"));
        format!("Read error: {e}")
    })?;

    log_debug(&format!("RESPONSE status={} body_len={}", status, body.len()));

    if !status.is_success() {
        // Log the full error body (it won't contain image data so it's small)
        log_debug(&format!("ERROR BODY: {body}"));
        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            return Err(format!("API error ({}): {}", status, err.error.message));
        }
        return Err(format!("API error ({}): {}", status, body));
    }

    // Log a snippet of the response structure (not the full b64 data)
    let preview = if body.len() > 500 { &body[..500] } else { &body };
    log_debug(&format!("RESPONSE PREVIEW: {preview}"));

    match serde_json::from_str::<ImageResponse>(&body) {
        Ok(parsed) => {
            let has_b64_json = parsed.data.first().and_then(|d| d.b64_json.as_ref()).is_some();
            let has_b64 = parsed.data.first().and_then(|d| d.b64.as_ref()).is_some();
            let has_any = parsed.data.first().and_then(|d| d.image_b64()).is_some();
            log_debug(&format!("PARSED OK: data_len={} has_b64_json={} has_b64={} has_any={}", parsed.data.len(), has_b64_json, has_b64, has_any));
            Ok(parsed)
        }
        Err(e) => {
            log_debug(&format!("PARSE ERROR: {e}"));
            log_debug(&format!("FULL RESPONSE KEYS: {}", &body[..body.len().min(2000)]));
            Err(format!("Parse error: {e}"))
        }
    }
}

/// Generate an image with reference image inputs (gpt-image edit endpoint).
/// Multiple reference images are sent as multipart file parts.
pub async fn generate_image_edit_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    reference_images: &[Vec<u8>],
    size: &str,
    quality: &str,
    output_format: Option<&str>,
) -> Result<ImageResponse, String> {
    let debug_path = std::path::PathBuf::from("/tmp/world-chat-image-debug.log");

    let log_debug = |msg: &str| {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&debug_path) {
            let _ = writeln!(f, "[{}] {}", chrono::Utc::now().to_rfc3339(), msg);
        }
    };

    let total_bytes: usize = reference_images.iter().map(|img| img.len()).sum();
    log_debug(&format!("EDIT REQUEST url={base_url}/images/edits model={model} size={size} quality={quality} ref_images={} total_bytes={}", reference_images.len(), total_bytes));
    log_debug(&format!("EDIT PROMPT (truncated): {:.200}", prompt));

    let client = Client::new();
    let url = format!("{base_url}/images/edits");

    let mut form = reqwest::multipart::Form::new()
        .text("model", model.to_string())
        .text("prompt", prompt.to_string())
        .text("n", "1")
        .text("size", size.to_string())
        .text("quality", quality.to_string());

    for (i, img_bytes) in reference_images.iter().enumerate() {
        let part = reqwest::multipart::Part::bytes(img_bytes.clone())
            .file_name(format!("reference_{i}.png"))
            .mime_str("image/png")
            .map_err(|e| format!("Failed to create image part: {e}"))?;
        form = form.part("image[]", part);
    }

    if let Some(fmt) = output_format {
        form = form.text("output_format", fmt.to_string());
    }

    let mut builder = client.post(&url).multipart(form);
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }

    let resp = match builder.send().await {
        Ok(r) => r,
        Err(e) => {
            log_debug(&format!("EDIT NETWORK ERROR: {e}"));
            return Err(format!("Network error: {e}"));
        }
    };

    let status = resp.status();
    let body = resp.text().await.map_err(|e| {
        log_debug(&format!("EDIT READ ERROR: {e}"));
        format!("Read error: {e}")
    })?;

    log_debug(&format!("EDIT RESPONSE status={} body_len={}", status, body.len()));

    if !status.is_success() {
        log_debug(&format!("EDIT ERROR BODY: {body}"));
        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            return Err(format!("API error ({}): {}", status, err.error.message));
        }
        return Err(format!("API error ({}): {}", status, body));
    }

    let preview = if body.len() > 500 { &body[..500] } else { &body };
    log_debug(&format!("EDIT RESPONSE PREVIEW: {preview}"));

    match serde_json::from_str::<ImageResponse>(&body) {
        Ok(parsed) => {
            let has_any = parsed.data.first().and_then(|d| d.image_b64()).is_some();
            log_debug(&format!("EDIT PARSED OK: data_len={} has_image={}", parsed.data.len(), has_any));
            Ok(parsed)
        }
        Err(e) => {
            log_debug(&format!("EDIT PARSE ERROR: {e}"));
            Err(format!("Parse error: {e}"))
        }
    }
}

// ─── Embeddings ─────────────────────────────────────────────────────────────

pub async fn create_embeddings_with_base(base_url: &str, api_key: &str, model: &str, texts: Vec<String>) -> Result<(Vec<Vec<f32>>, u32), String> {
    let client = Client::new();
    let request = EmbeddingRequest {
        model: model.to_string(),
        input: texts,
    };
    let url = format!("{base_url}/embeddings");
    let mut builder = client.post(&url).json(&request);
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }
    let resp = builder.send().await.map_err(|e| format!("Network error: {e}"))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("Read error: {e}"))?;

    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            return Err(format!("API error ({}): {}", status, err.error.message));
        }
        return Err(format!("API error ({}): {}", status, body));
    }

    let parsed: EmbeddingResponse = serde_json::from_str(&body).map_err(|e| format!("Parse error: {e}"))?;
    let tokens = parsed.usage.map(|u| u.total_tokens).unwrap_or(0);
    Ok((parsed.data.into_iter().map(|d| d.embedding).collect(), tokens))
}

// ─── Text-to-Speech ────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TtsRequest {
    pub model: String,
    pub input: String,
    pub voice: String,
}

/// Generate speech audio via OpenAI TTS API. Returns raw MP3 bytes.
/// Routes to /audio/speech for dedicated TTS models, or /chat/completions
/// with audio modality for chat-based audio models (e.g. gpt-audio-1.5).
pub async fn text_to_speech(base_url: &str, api_key: &str, request: &TtsRequest) -> Result<Vec<u8>, String> {
    if is_chat_audio_model(&request.model) {
        text_to_speech_via_chat(base_url, api_key, request).await
    } else {
        text_to_speech_direct(base_url, api_key, request).await
    }
}

/// Returns true for models that use the chat completions endpoint with audio modality.
fn is_chat_audio_model(model: &str) -> bool {
    model.starts_with("gpt-audio")
}

/// Direct TTS via /audio/speech (tts-1, tts-1-hd, gpt-4o-mini-tts).
async fn text_to_speech_direct(base_url: &str, api_key: &str, request: &TtsRequest) -> Result<Vec<u8>, String> {
    let client = Client::new();
    let url = format!("{base_url}/audio/speech");
    let mut builder = client.post(&url).json(request);
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }
    let resp = builder.send().await.map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            return Err(format!("API error ({}): {}", status, err.error.message));
        }
        return Err(format!("API error ({}): {}", status, body));
    }
    resp.bytes().await.map(|b| b.to_vec()).map_err(|e| format!("Read error: {e}"))
}

/// Chat-based TTS via /chat/completions with audio modality (gpt-audio-1.5 etc.).
async fn text_to_speech_via_chat(base_url: &str, api_key: &str, request: &TtsRequest) -> Result<Vec<u8>, String> {
    let client = Client::new();
    let url = format!("{base_url}/chat/completions");

    let body = serde_json::json!({
        "model": request.model,
        "modalities": ["text", "audio"],
        "audio": {
            "voice": request.voice,
            "format": "mp3"
        },
        "messages": [
            {
                "role": "user",
                "content": format!("Read the following text aloud exactly as written. Do not add, change, or omit any words. Do not include any commentary, confirmation, or preamble such as \"Sure\" or \"Here's the text\". Speak only the provided text.\n\n{}", request.input)
            }
        ]
    });

    let mut builder = client.post(&url).json(&body);
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }
    let resp = builder.send().await.map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    let resp_body = resp.text().await.map_err(|e| format!("Read error: {e}"))?;

    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<ApiError>(&resp_body) {
            return Err(format!("API error ({}): {}", status, err.error.message));
        }
        return Err(format!("API error ({}): {}", status, resp_body));
    }

    // Parse the audio data from the chat response
    let parsed: serde_json::Value = serde_json::from_str(&resp_body)
        .map_err(|e| format!("Parse error: {e}"))?;

    let audio_b64 = parsed
        .get("choices").and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("audio"))
        .and_then(|a| a.get("data"))
        .and_then(|d| d.as_str())
        .ok_or_else(|| format!("No audio data in response: {}", &resp_body[..resp_body.len().min(500)]))?;

    b64_decode(audio_b64)
}

fn b64_decode(input: &str) -> Result<Vec<u8>, String> {
    const DECODE: [u8; 128] = {
        let mut table = [255u8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 { table[chars[i] as usize] = i as u8; i += 1; }
        table
    };
    let input = input.as_bytes();
    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;
    for &b in input {
        if b == b'=' || b == b'\n' || b == b'\r' { continue; }
        if b >= 128 { return Err("Invalid base64 character".to_string()); }
        let val = DECODE[b as usize];
        if val == 255 { return Err(format!("Invalid base64 character: {}", b as char)); }
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 { bits -= 8; result.push((buf >> bits) as u8); buf &= (1 << bits) - 1; }
    }
    Ok(result)
}

// ─── List Models ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    #[serde(default)]
    pub owned_by: String,
}

pub async fn list_models(base_url: &str, api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let client = Client::new();
    let url = format!("{base_url}/models");
    let mut builder = client.get(&url);
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }
    let resp = builder.send().await.map_err(|e| format!("Network error: {e}"))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("Read error: {e}"))?;

    if !status.is_success() {
        return Err(format!("Failed to list models ({}): {}", status, body));
    }

    let parsed: ModelsResponse = serde_json::from_str(&body).map_err(|e| format!("Parse error: {e}"))?;
    Ok(parsed.data)
}

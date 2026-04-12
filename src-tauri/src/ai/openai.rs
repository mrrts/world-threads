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
pub async fn text_to_speech(base_url: &str, api_key: &str, request: &TtsRequest) -> Result<Vec<u8>, String> {
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

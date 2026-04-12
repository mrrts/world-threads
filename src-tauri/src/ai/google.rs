use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const VEO_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const POLL_INTERVAL_SECS: u64 = 5;
const POLL_TIMEOUT_SECS: u64 = 300; // 5 minutes max

// ─── Request Types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct VeoRequest {
    instances: Vec<VeoInstance>,
    parameters: VeoParameters,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VeoInstance {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<VeoImageInput>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VeoImageInput {
    bytes_base64_encoded: String,
    mime_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VeoParameters {
    sample_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    person_generation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generate_audio: Option<bool>,
}

// ─── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct VeoOperationResponse {
    name: Option<String>,
    done: Option<bool>,
    response: Option<VeoResultResponse>,
    error: Option<VeoErrorResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VeoResultResponse {
    generate_video_response: Option<VeoGenerateVideoResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VeoGenerateVideoResponse {
    generated_samples: Option<Vec<VeoSample>>,
}

#[derive(Debug, Deserialize)]
struct VeoSample {
    video: Option<VeoVideo>,
}

#[derive(Debug, Deserialize)]
struct VeoVideo {
    uri: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct VeoErrorResponse {
    message: Option<String>,
    code: Option<i32>,
}

// ─── Debug Logging ──────────────────────────────────────────────────────────

fn log_debug(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/world-chat-video-debug.log")
    {
        let _ = writeln!(f, "[{}] {}", chrono::Utc::now().to_rfc3339(), msg);
    }
}

// ─── API Functions ──────────────────────────────────────────────────────────

/// Start a Veo video generation job. Returns the operation name for polling.
pub async fn start_veo_generation(
    api_key: &str,
    model: &str,
    image_b64: Option<&str>,
    prompt: &str,
    duration_seconds: Option<u32>,
    aspect_ratio: Option<&str>,
    generate_audio: Option<bool>,
) -> Result<String, String> {
    let url = format!("{VEO_BASE_URL}/models/{model}:predictLongRunning?key={api_key}");

    let is_lite = model.contains("lite");

    let instance = VeoInstance {
        prompt: prompt.to_string(),
        image: image_b64.map(|b64| VeoImageInput {
            bytes_base64_encoded: b64.to_string(),
            mime_type: "image/png".to_string(),
        }),
    };

    let request = VeoRequest {
        instances: vec![instance],
        parameters: VeoParameters {
            sample_count: 1,
            duration_seconds,
            aspect_ratio: aspect_ratio.map(|s| s.to_string()),
            person_generation: None,
            negative_prompt: if is_lite { None } else {
                Some("text, words, letters, watermark, UI, blurry, distorted faces, cartoon, anime, painterly, flat colors, cel shading".to_string())
            },
            generate_audio,
        },
    };

    log_debug(&format!(
        "VEO START model={model} prompt_len={} has_image={} duration={:?} aspect={:?}",
        prompt.len(),
        image_b64.is_some(),
        duration_seconds,
        aspect_ratio,
    ));

    let client = Client::new();
    let resp = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            log_debug(&format!("VEO NETWORK ERROR: {e}"));
            format!("Network error: {e}")
        })?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("Read error: {e}"))?;

    log_debug(&format!("VEO START RESPONSE status={status} body_len={}", body.len()));

    if !status.is_success() {
        log_debug(&format!("VEO START ERROR: {body}"));
        if status.as_u16() == 429 {
            return Err("RATE_LIMITED".to_string());
        }
        // Try to extract a readable error message
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(msg) = parsed.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
                return Err(format!("Veo error: {msg}"));
            }
        }
        return Err(format!("Veo API error ({}): {}", status, &body[..body.len().min(500)]));
    }

    let op: VeoOperationResponse =
        serde_json::from_str(&body).map_err(|e| {
            log_debug(&format!("VEO PARSE ERROR: {e}\nBody: {}", &body[..body.len().min(500)]));
            format!("Failed to parse Veo response: {e}")
        })?;

    // If the operation completed immediately
    if op.done.unwrap_or(false) {
        if let Some(err) = &op.error {
            let msg = err.message.as_deref().unwrap_or("Unknown error");
            log_debug(&format!("VEO IMMEDIATE ERROR: {msg}"));
            return Err(format!("Veo error: {msg}"));
        }
    }

    let op_name = op
        .name
        .ok_or_else(|| "Veo response missing operation name".to_string())?;

    log_debug(&format!("VEO OPERATION: {op_name}"));
    Ok(op_name)
}

/// Poll a Veo operation until it completes. Returns the video download URL.
pub async fn poll_veo_until_done(api_key: &str, operation_name: &str) -> Result<String, String> {
    let url = format!("{VEO_BASE_URL}/{operation_name}?key={api_key}");
    let client = Client::new();
    let start = std::time::Instant::now();

    loop {
        if start.elapsed().as_secs() > POLL_TIMEOUT_SECS {
            log_debug("VEO TIMEOUT");
            return Err("Video generation timed out after 5 minutes".to_string());
        }

        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;

        let resp = client.get(&url).send().await.map_err(|e| {
            log_debug(&format!("VEO POLL NETWORK ERROR: {e}"));
            format!("Poll network error: {e}")
        })?;

        let status = resp.status();
        let body = resp.text().await.map_err(|e| format!("Poll read error: {e}"))?;

        if !status.is_success() {
            log_debug(&format!("VEO POLL ERROR status={status}: {}", &body[..body.len().min(500)]));
            return Err(format!("Veo poll error ({}): {}", status, &body[..body.len().min(500)]));
        }

        let op: VeoOperationResponse = serde_json::from_str(&body).map_err(|e| {
            log_debug(&format!("VEO POLL PARSE ERROR: {e}"));
            format!("Failed to parse poll response: {e}")
        })?;

        let elapsed = start.elapsed().as_secs();
        log_debug(&format!("VEO POLL {elapsed}s done={:?}", op.done));

        if let Some(err) = &op.error {
            let msg = err.message.as_deref().unwrap_or("Unknown error");
            log_debug(&format!("VEO ERROR: {msg}"));
            return Err(format!("Veo error: {msg}"));
        }

        if op.done.unwrap_or(false) {
            let video_uri = op
                .response
                .and_then(|r| r.generate_video_response)
                .and_then(|g| g.generated_samples)
                .and_then(|mut s| s.pop())
                .and_then(|s| s.video)
                .and_then(|v| v.uri)
                .ok_or_else(|| "Veo completed but no video URI found".to_string())?;

            log_debug(&format!("VEO DONE uri_len={}", video_uri.len()));
            return Ok(video_uri);
        }
    }
}

/// Download a video from a signed URL. Returns the raw video bytes.
pub async fn download_video(url: &str, api_key: &str) -> Result<Vec<u8>, String> {
    // The video URI may need the API key appended
    let download_url = if url.contains('?') {
        format!("{url}&key={api_key}")
    } else {
        format!("{url}?key={api_key}")
    };

    log_debug(&format!("VEO DOWNLOAD url_len={}", download_url.len()));

    let client = Client::new();
    let resp = client
        .get(&download_url)
        .timeout(Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| {
            log_debug(&format!("VEO DOWNLOAD ERROR: {e}"));
            format!("Download error: {e}")
        })?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        log_debug(&format!("VEO DOWNLOAD FAIL status={status}: {}", &body[..body.len().min(500)]));
        return Err(format!("Download failed ({}): {}", status, &body[..body.len().min(200)]));
    }

    let bytes = resp.bytes().await.map_err(|e| format!("Download read error: {e}"))?;
    log_debug(&format!("VEO DOWNLOAD OK {} bytes", bytes.len()));
    Ok(bytes.to_vec())
}

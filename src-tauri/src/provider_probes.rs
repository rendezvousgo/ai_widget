use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;

use crate::api_keys::get_key;

#[derive(Serialize, Clone, Default)]
pub struct ProbeResult {
    pub provider: String,
    pub valid: bool,
    pub status: String, // "connected" | "missing-key" | "invalid" | "error"
    pub detail: String,
    pub items: Vec<ProbeItem>,
    pub fetched_at: i64,
    pub cached: bool,
}

#[derive(Serialize, Clone, Default)]
pub struct ProbeItem {
    pub label: String,
    pub used_pct: f64,
    pub raw: String,
    pub reset_label: String,
}

static PROBE_CACHE: Mutex<Option<Vec<(Instant, ProbeResult)>>> = Mutex::new(None);
const CACHE_TTL_SECS: u64 = 60;

fn cache_get(provider: &str) -> Option<ProbeResult> {
    let cache = PROBE_CACHE.lock().ok()?;
    let entries = cache.as_ref()?;
    for (t, r) in entries.iter() {
        if r.provider == provider && t.elapsed().as_secs() < CACHE_TTL_SECS {
            let mut hit = r.clone();
            hit.cached = true;
            return Some(hit);
        }
    }
    None
}

fn cache_put(provider: &str, r: ProbeResult) {
    let mut cache = PROBE_CACHE.lock().unwrap();
    let now = Instant::now();
    let entries = cache.get_or_insert_with(Vec::new);
    entries.retain(|(_, e)| e.provider != provider);
    entries.push((now, r));
}

fn no_key(provider: &str) -> ProbeResult {
    ProbeResult {
        provider: provider.into(),
        valid: false,
        status: "missing-key".into(),
        detail: "API key not set".into(),
        items: vec![],
        fetched_at: chrono::Utc::now().timestamp(),
        cached: false,
    }
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("ai-quota-widget/0.1")
        .build()
        .map_err(|e| e.to_string())
}

async fn probe_openai(key: &str) -> ProbeResult {
    let mut r = ProbeResult { provider: "openai".into(), ..Default::default() };
    r.fetched_at = chrono::Utc::now().timestamp();
    let client = match http_client() { Ok(c) => c, Err(e) => { r.status = "error".into(); r.detail = e; return r; } };
    // probe a lightweight chat completion to get rate-limit headers (since /v1/models doesn't always include them)
    // we'll use /v1/models first for validation
    let resp = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(key)
        .send()
        .await;
    let resp = match resp { Ok(r) => r, Err(e) => { r.status = "error".into(); r.detail = format!("network: {}", e); return r; } };
    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        r.status = "invalid".into(); r.detail = "API key invalid or missing — check Settings".into(); return r;
    }
    if !status.is_success() {
        r.status = "error".into(); r.detail = format!("HTTP {}", status.as_u16()); return r;
    }
    // /v1/models doesn't return rate-limit headers. Do a minimal chat completion request to capture headers.
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [{"role":"user","content":"."}],
        "max_tokens": 1,
    });
    let resp2 = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(key)
        .json(&body)
        .send()
        .await;
    let resp2 = match resp2 { Ok(r) => r, Err(e) => { r.status = "error".into(); r.detail = format!("probe: {}", e); return r; } };
    let headers = resp2.headers().clone();
    let _ = resp2.text().await;

    let h = |k: &str| -> Option<String> {
        headers.get(k).and_then(|v| v.to_str().ok().map(|s| s.to_string()))
    };
    let rpm_lim = h("x-ratelimit-limit-requests").and_then(|s| s.parse::<f64>().ok());
    let rpm_rem = h("x-ratelimit-remaining-requests").and_then(|s| s.parse::<f64>().ok());
    let tpm_lim = h("x-ratelimit-limit-tokens").and_then(|s| s.parse::<f64>().ok());
    let tpm_rem = h("x-ratelimit-remaining-tokens").and_then(|s| s.parse::<f64>().ok());
    let rpm_reset = h("x-ratelimit-reset-requests").unwrap_or_default();
    let tpm_reset = h("x-ratelimit-reset-tokens").unwrap_or_default();

    if let (Some(lim), Some(rem)) = (rpm_lim, rpm_rem) {
        let used = ((lim - rem) / lim * 100.0).max(0.0).min(100.0);
        r.items.push(ProbeItem {
            label: "RPM".into(),
            used_pct: used,
            raw: format!("{} / {}", lim - rem, lim),
            reset_label: rpm_reset,
        });
    }
    if let (Some(lim), Some(rem)) = (tpm_lim, tpm_rem) {
        let used = ((lim - rem) / lim * 100.0).max(0.0).min(100.0);
        r.items.push(ProbeItem {
            label: "TPM".into(),
            used_pct: used,
            raw: format!("{} / {}", lim - rem, lim),
            reset_label: tpm_reset,
        });
    }
    r.valid = true;
    r.status = "connected".into();
    r.detail = if r.items.is_empty() { "connected (no rate-limit headers returned)".into() } else { "live".into() };
    r
}

async fn probe_gemini(key: &str) -> ProbeResult {
    let mut r = ProbeResult { provider: "gemini".into(), ..Default::default() };
    r.fetched_at = chrono::Utc::now().timestamp();
    let client = match http_client() { Ok(c) => c, Err(e) => { r.status = "error".into(); r.detail = e; return r; } };
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", key);
    let resp = client.get(&url).send().await;
    match resp {
        Ok(rsp) => {
            let status = rsp.status();
            if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
                r.status = "invalid".into(); r.detail = "auth failed".into();
            } else if !status.is_success() {
                r.status = "error".into(); r.detail = format!("HTTP {}", status.as_u16());
            } else {
                r.valid = true;
                r.status = "connected".into();
                if let Ok(j) = rsp.json::<serde_json::Value>().await {
                    let n = j.get("models").and_then(|m| m.as_array()).map(|a| a.len()).unwrap_or(0);
                    r.detail = format!("{} models accessible (no usage API)", n);
                } else {
                    r.detail = "connected".into();
                }
            }
        }
        Err(e) => { r.status = "error".into(); r.detail = format!("network: {}", e); }
    }
    r
}

async fn probe_simple(provider: &str, key: &str, url: &str) -> ProbeResult {
    let mut r = ProbeResult { provider: provider.into(), ..Default::default() };
    r.fetched_at = chrono::Utc::now().timestamp();
    let client = match http_client() { Ok(c) => c, Err(e) => { r.status = "error".into(); r.detail = e; return r; } };
    let resp = client.get(url).bearer_auth(key).send().await;
    match resp {
        Ok(rsp) => {
            let status = rsp.status();
            if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
                r.status = "invalid".into(); r.detail = "auth failed".into();
            } else if !status.is_success() {
                r.status = "error".into(); r.detail = format!("HTTP {}", status.as_u16());
            } else {
                r.valid = true;
                r.status = "connected".into();
                if let Ok(j) = rsp.json::<serde_json::Value>().await {
                    let n = j.get("data").and_then(|m| m.as_array()).map(|a| a.len())
                        .or_else(|| j.get("models").and_then(|m| m.as_array()).map(|a| a.len()))
                        .unwrap_or(0);
                    r.detail = if n > 0 { format!("{} models", n) } else { "connected".into() };
                } else {
                    r.detail = "connected".into();
                }
            }
        }
        Err(e) => { r.status = "error".into(); r.detail = format!("network: {}", e); }
    }
    r
}

#[tauri::command]
pub async fn probe_provider(provider: String, force: Option<bool>) -> Result<ProbeResult, String> {
    if !force.unwrap_or(false) {
        if let Some(c) = cache_get(&provider) { return Ok(c); }
    }
    let key = match get_key(&provider) {
        Some(k) => k,
        None => return Ok(no_key(&provider)),
    };
    let result = match provider.as_str() {
        "openai" => probe_openai(&key).await,
        "gemini" => probe_gemini(&key).await,
        "perplexity" => probe_simple(&provider, &key, "https://api.perplexity.ai/models").await,
        "xai" => probe_simple(&provider, &key, "https://api.x.ai/v1/models").await,
        "groq" => probe_simple(&provider, &key, "https://api.groq.com/openai/v1/models").await,
        "mistral" => probe_simple(&provider, &key, "https://api.mistral.ai/v1/models").await,
        "deepseek" => probe_simple(&provider, &key, "https://api.deepseek.com/v1/models").await,
        _ => {
            let mut r = no_key(&provider);
            r.status = "error".into();
            r.detail = format!("unknown provider: {}", provider);
            r
        }
    };
    cache_put(&provider, result.clone());
    Ok(result)
}

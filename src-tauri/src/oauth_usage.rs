use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct Bucket {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsageResponse {
    five_hour: Option<Bucket>,
    seven_day: Option<Bucket>,
    seven_day_sonnet: Option<Bucket>,
    seven_day_opus: Option<Bucket>,
    seven_day_omelette: Option<Bucket>,
    seven_day_oauth_apps: Option<Bucket>,
    seven_day_cowork: Option<Bucket>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageItem {
    pub label: String,
    pub utilization: f64,
    pub resets_at: String,
    pub reset_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthUsage {
    pub items: Vec<UsageItem>,
    pub fetched_at: i64,
    pub cached: bool,
    pub plan: String,
    pub plan_raw: String,
}

static CACHE: Mutex<Option<(Instant, OAuthUsage)>> = Mutex::new(None);
const CACHE_TTL_SECS: u64 = 45;

fn read_access_token() -> Option<String> {
    let path = crate::paths::claude_credentials()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("claudeAiOauth")?
        .get("accessToken")?
        .as_str()
        .map(|s| s.to_string())
}

fn fmt_reset(iso: &str) -> String {
    let Ok(t) = chrono::DateTime::parse_from_rfc3339(iso) else {
        return iso.to_string();
    };
    let now = chrono::Utc::now();
    let secs = (t.timestamp() - now.timestamp()).max(0);
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let d = h / 24;
    if d > 0 {
        format!("in {}d {}h", d, h % 24)
    } else if h > 0 {
        format!("in {}h {}m", h, m)
    } else {
        format!("in {}m", m)
    }
}

fn push_item(items: &mut Vec<UsageItem>, label: &str, b: Option<Bucket>) {
    // null 버킷 = 사용자 플랜에 적용 안 되는 메트릭이므로 스킵.
    // bucket은 존재하나 utilization=0&resets_at=null인 경우 (사용량 없음) → 0%로 표시.
    if let Some(b) = b {
        if let Some(u) = b.utilization {
            let r = b.resets_at.unwrap_or_default();
            let reset_label = if r.is_empty() { "—".to_string() } else { fmt_reset(&r) };
            items.push(UsageItem {
                label: label.into(),
                utilization: u,
                resets_at: r,
                reset_label,
            });
        }
    }
}

#[tauri::command]
pub async fn get_oauth_usage(force: Option<bool>) -> Result<OAuthUsage, String> {
    let force = force.unwrap_or(false);
    if !force {
        if let Some((t, u)) = CACHE.lock().unwrap().as_ref() {
            if t.elapsed().as_secs() < CACHE_TTL_SECS {
                let mut cached = u.clone();
                cached.cached = true;
                return Ok(cached);
            }
        }
    }

    let token = read_access_token().ok_or_else(|| {
        "no access_token in ~/.claude/.credentials.json".to_string()
    })?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", format!("Bearer {}", token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", "ai-quota-widget/0.1")
        .send()
        .await
        .map_err(|e| format!("network: {}", e))?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("Claude session expired — run `claude` once to refresh.".into());
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        // serve stale cache if any
        if let Some((_, u)) = CACHE.lock().unwrap().as_ref() {
            let mut stale = u.clone();
            stale.cached = true;
            return Ok(stale);
        }
        return Err("rate-limited (429); try again in a few minutes".into());
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), body));
    }

    let data: UsageResponse = resp.json().await.map_err(|e| format!("parse: {}", e))?;

    // Codename → user-facing label mapping.
    //   omelette  → "Design" (Claude's new Design feature)
    //   oauth_apps → "OAuth Apps" (3rd-party clients via OAuth)
    //   cowork    → "Cowork" (Claude background/agent work)
    let mut items: Vec<UsageItem> = Vec::new();
    push_item(&mut items, "5h window", data.five_hour);
    push_item(&mut items, "Weekly", data.seven_day);
    push_item(&mut items, "Weekly · Sonnet", data.seven_day_sonnet);
    push_item(&mut items, "Weekly · Opus", data.seven_day_opus);
    push_item(&mut items, "Weekly · Design", data.seven_day_omelette);
    push_item(&mut items, "Weekly · OAuth Apps", data.seven_day_oauth_apps);
    push_item(&mut items, "Weekly · Cowork", data.seven_day_cowork);

    let (plan_raw, _) = crate::claude_usage::read_subscription_pub();
    let plan = crate::claude_usage::plan_name_for(&plan_raw);

    let result = OAuthUsage {
        items,
        fetched_at: chrono::Utc::now().timestamp(),
        cached: false,
        plan,
        plan_raw,
    };

    *CACHE.lock().unwrap() = Some((Instant::now(), result.clone()));
    Ok(result)
}

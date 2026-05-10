use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
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
const CACHE_TTL_SECS: u64 = 90;
// On any transient API failure (network, 5xx, 429), serve cached data up to
// this many seconds old instead of leaving the user with a blank screen.
const STALE_FALLBACK_SECS: u64 = 3600;

/// Return last cached usage marked as stale, if it's still within the
/// fallback window. Used when fetching fresh data fails for a recoverable
/// reason and we don't want to flash an error at the user.
fn stale_fallback() -> Option<OAuthUsage> {
    let cache = CACHE.lock().ok()?;
    let (t, u) = cache.as_ref()?;
    if t.elapsed().as_secs() <= STALE_FALLBACK_SECS {
        let mut stale = u.clone();
        stale.cached = true;
        Some(stale)
    } else {
        None
    }
}

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

    // Network-level failure → if we have a recent cache, serve it stale
    // instead of forcing the user to stare at an error.
    let resp = match client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", format!("Bearer {}", token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", "ai-quota-widget/0.1")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            if let Some(stale) = stale_fallback() { return Ok(stale); }
            return Err(format!("network: {}", e));
        }
    };

    let status = resp.status();
    // 401 = token expired. User has to run `claude` to refresh, so we
    // surface this loudly even if a cache exists.
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("Claude session expired — run `claude` once to refresh.".into());
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        if let Some(stale) = stale_fallback() { return Ok(stale); }
        return Err("rate-limited (429); try again in a few minutes".into());
    }
    if !status.is_success() {
        // 5xx, 502/503/504 etc. — same idea, prefer stale data over a blank error.
        if let Some(stale) = stale_fallback() { return Ok(stale); }
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), body));
    }

    let data: UsageResponse = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            if let Some(stale) = stale_fallback() { return Ok(stale); }
            return Err(format!("parse: {}", e));
        }
    };

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
    append_snapshot(&result);
    Ok(result)
}

/// Append a usage snapshot to ~/.config/ai-quota-widget/usage_history.jsonl
/// (deduped — only keep one snapshot per ~5 minutes).
fn append_snapshot(usage: &OAuthUsage) {
    let Some(dir) = crate::paths::app_config_dir() else { return };
    let path = dir.join("usage_history.jsonl");

    // Skip if last entry is < 5 min ago.
    if let Ok(file) = std::fs::File::open(&path) {
        let last_ts = BufReader::new(file)
            .lines()
            .filter_map(|l| l.ok())
            .last()
            .and_then(|l| serde_json::from_str::<serde_json::Value>(&l).ok())
            .and_then(|v| v.get("ts").and_then(|t| t.as_i64()));
        if let Some(t) = last_ts {
            if usage.fetched_at - t < 300 {
                return;
            }
        }
    }

    // Pull two main metrics; ignore others to keep the file small.
    let five_h = usage.items.iter().find(|i| i.label == "5h window").map(|i| i.utilization);
    let seven_d = usage.items.iter().find(|i| i.label == "Weekly").map(|i| i.utilization);

    let entry = serde_json::json!({
        "ts": usage.fetched_at,
        "five_hour": five_h,
        "seven_day": seven_d,
    });
    let line = format!("{}\n", entry);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

#[derive(Debug, Serialize)]
pub struct HistoryPoint {
    pub ts: i64,
    pub five_hour: Option<f64>,
    pub seven_day: Option<f64>,
}

#[tauri::command]
pub fn get_usage_history(days: Option<u32>) -> Vec<HistoryPoint> {
    let days = days.unwrap_or(30).clamp(1, 90);
    let cutoff = chrono::Utc::now().timestamp() - (days as i64) * 24 * 3600;
    let Some(dir) = crate::paths::app_config_dir() else { return vec![] };
    let path = dir.join("usage_history.jsonl");
    let Ok(file) = std::fs::File::open(&path) else { return vec![] };
    let mut out: Vec<HistoryPoint> = Vec::new();
    for line in BufReader::new(file).lines().flatten() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
        let ts = match v.get("ts").and_then(|t| t.as_i64()) {
            Some(t) if t >= cutoff => t,
            _ => continue,
        };
        out.push(HistoryPoint {
            ts,
            five_hour: v.get("five_hour").and_then(|x| x.as_f64()),
            seven_day: v.get("seven_day").and_then(|x| x.as_f64()),
        });
    }
    out
}

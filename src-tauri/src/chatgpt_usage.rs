use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct RateWindow {
    used_percent: Option<f64>,
    limit_window_seconds: Option<i64>,
    reset_after_seconds: Option<i64>,
    reset_at: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct RateLimit {
    primary_window: Option<RateWindow>,
    secondary_window: Option<RateWindow>,
}

#[derive(Debug, Deserialize)]
struct WhamResponse {
    plan_type: Option<String>,
    rate_limit: Option<RateLimit>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatGptUsageItem {
    pub label: String,
    pub utilization: f64,
    pub resets_at: String,
    pub reset_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatGptUsage {
    pub items: Vec<ChatGptUsageItem>,
    pub fetched_at: i64,
    pub cached: bool,
    pub plan: String,
    pub renews_at_utc: Option<String>,
    pub renews_at_kst: Option<String>,
    pub renews_days: Option<i64>,
}

static CACHE: Mutex<Option<(Instant, ChatGptUsage)>> = Mutex::new(None);
const CACHE_TTL_SECS: u64 = 90;
const STALE_FALLBACK_SECS: u64 = 3600;

fn stale_fallback() -> Option<ChatGptUsage> {
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

fn read_codex_token() -> Option<String> {
    let home = crate::paths::home()?;
    let paths = [
        home.join(".codex").join("auth.json"),
        home.join(".chatgpt-local").join("auth.json"),
    ];
    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(tok) = v.get("tokens")
                    .and_then(|t| t.get("access_token"))
                    .and_then(|t| t.as_str())
                {
                    return Some(tok.to_string());
                }
                if let Some(tok) = v.get("access_token").and_then(|t| t.as_str()) {
                    return Some(tok.to_string());
                }
            }
        }
    }
    None
}

fn fmt_reset_secs(secs: i64) -> String {
    if secs <= 0 {
        return "now".to_string();
    }
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

fn plan_label(raw: &str) -> String {
    match raw {
        "plus" => "Plus".into(),
        "pro" => "Pro".into(),
        "free" => "Free".into(),
        "team" => "Team".into(),
        "business" => "Business".into(),
        "enterprise" => "Enterprise".into(),
        other => {
            let mut c = other.chars();
            match c.next() {
                Some(first) => first.to_uppercase().to_string() + c.as_str(),
                None => other.to_string(),
            }
        }
    }
}

fn push_window(items: &mut Vec<ChatGptUsageItem>, label: &str, w: Option<RateWindow>) {
    if let Some(w) = w {
        if let Some(pct) = w.used_percent {
            let reset = w.reset_after_seconds.unwrap_or(0);
            items.push(ChatGptUsageItem {
                label: label.into(),
                utilization: pct,
                resets_at: w.reset_at.map(|t| t.to_string()).unwrap_or_default(),
                reset_label: if reset > 0 { fmt_reset_secs(reset) } else { "—".into() },
            });
        }
    }
}

#[tauri::command]
pub async fn get_chatgpt_usage(force: Option<bool>) -> Result<ChatGptUsage, String> {
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

    let token = read_codex_token().ok_or_else(|| {
        "no access_token in ~/.codex/auth.json — run `codex login` first".to_string()
    })?;

    let mut cmd = std::process::Command::new("curl");
    cmd.args([
        "-s", "--max-time", "10",
        "-H", &format!("Authorization: Bearer {}", token),
        "-H", "User-Agent: ai-quota-widget/0.1",
        "https://chatgpt.com/backend-api/wham/usage",
    ]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd.output()
        .map_err(|e| {
            if let Some(stale) = stale_fallback() { return "stale".to_string(); }
            format!("curl: {}", e)
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        if let Some(stale) = stale_fallback() { return Ok(stale); }
        return Err("empty response from chatgpt".to_string());
    }

    let data: WhamResponse = serde_json::from_str(stdout.trim()).map_err(|e| {
        if let Some(stale) = stale_fallback() { return "stale".to_string(); }
        format!("parse: {}", e)
    })?;

    let mut items: Vec<ChatGptUsageItem> = Vec::new();
    if let Some(rl) = data.rate_limit {
        push_window(&mut items, "5h window", rl.primary_window);
        push_window(&mut items, "Weekly", rl.secondary_window);
    }

    let plan_raw = data.plan_type.as_deref().unwrap_or("unknown");
    let plan = plan_label(plan_raw);

    let (renews_at_utc, renews_at_kst, renews_days) = fetch_chatgpt_renewal();

    let result = ChatGptUsage {
        items,
        fetched_at: chrono::Utc::now().timestamp(),
        cached: false,
        plan,
        renews_at_utc,
        renews_at_kst,
        renews_days,
    };

    *CACHE.lock().unwrap() = Some((Instant::now(), result.clone()));
    append_snapshot(&result);
    Ok(result)
}

fn append_snapshot(usage: &ChatGptUsage) {
    let Some(dir) = crate::paths::app_config_dir() else { return };
    let path = dir.join("chatgpt_usage_history.jsonl");

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
pub struct ChatGptHistoryPoint {
    pub ts: i64,
    pub five_hour: Option<f64>,
    pub seven_day: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct CodexDailyEntry {
    pub date: String,
    pub tokens: i64,
    pub messages: i64,
}

#[tauri::command]
pub fn get_codex_daily_tokens(days: Option<u32>) -> Vec<CodexDailyEntry> {
    let days = days.unwrap_or(30).clamp(1, 90) as i64;
    let Some(home) = crate::paths::home() else { return vec![] };
    let db_path = home.join(".codex").join("state_5.sqlite");
    let Ok(db) = rusqlite::Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) else { return vec![] };
    let cutoff = chrono::Utc::now().timestamp() - days * 24 * 3600;
    let mut stmt = match db.prepare(
        "SELECT date(created_at, 'unixepoch') as day, SUM(tokens_used) as total_tokens, COUNT(*) as msgs FROM threads WHERE tokens_used > 0 AND created_at >= ?1 GROUP BY day ORDER BY day"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let rows = stmt.query_map([cutoff], |row| {
        Ok(CodexDailyEntry {
            date: row.get(0)?,
            tokens: row.get(1)?,
            messages: row.get(2)?,
        })
    });
    match rows {
        Ok(r) => r.filter_map(|r| r.ok()).collect(),
        Err(_) => vec![],
    }
}

#[tauri::command]
pub fn get_chatgpt_usage_history(days: Option<u32>) -> Vec<ChatGptHistoryPoint> {
    let days = days.unwrap_or(30).clamp(1, 90);
    let cutoff = chrono::Utc::now().timestamp() - (days as i64) * 24 * 3600;
    let Some(dir) = crate::paths::app_config_dir() else { return vec![] };
    let path = dir.join("chatgpt_usage_history.jsonl");
    let Ok(file) = std::fs::File::open(&path) else { return vec![] };
    let mut out: Vec<ChatGptHistoryPoint> = Vec::new();
    for line in BufReader::new(file).lines().flatten() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
        let ts = match v.get("ts").and_then(|t| t.as_i64()) {
            Some(t) if t >= cutoff => t,
            _ => continue,
        };
        out.push(ChatGptHistoryPoint {
            ts,
            five_hour: v.get("five_hour").and_then(|x| x.as_f64()),
            seven_day: v.get("seven_day").and_then(|x| x.as_f64()),
        });
    }
    out
}

fn fetch_chatgpt_renewal() -> (Option<String>, Option<String>, Option<i64>) {
    fn inner() -> Option<(String, String, i64)> {
        // 1차: 캐시 파일에서 읽기
        if let Some(dir) = crate::paths::app_config_dir() {
            let cache_path = dir.join("chatgpt_renewal.json");
            if let Ok(content) = std::fs::read_to_string(&cache_path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(renews_str) = v.get("renews_at").and_then(|r| r.as_str()) {
                        if let Ok(renews_dt) = chrono::DateTime::parse_from_rfc3339(renews_str) {
                            let now = chrono::Utc::now();
                            let kst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
                            let days = (renews_dt.with_timezone(&chrono::Utc) - now).num_days();
                            if days >= 0 {
                                let utc_str = renews_dt.with_timezone(&chrono::Utc).format("%Y-%m-%d").to_string();
                                let kst_str = renews_dt.with_timezone(&kst).format("%Y-%m-%d").to_string();
                                return Some((utc_str, kst_str, days));
                            }
                        }
                    }
                }
            }
        }

        // 2차: API에서 가져오기 (curl_cffi 필요)
        let home = crate::paths::home()?;
        let auth_path = home.join(".codex").join("auth.json");
        let auth_content = std::fs::read_to_string(&auth_path).ok()?;
        let auth_json: serde_json::Value = serde_json::from_str(&auth_content).ok()?;
        let token = auth_json.get("tokens")?.get("access_token")?.as_str()?;

        let script = format!(
            r#"
import json, uuid
try:
    from curl_cffi import requests as r
    resp = r.get(
        "https://chatgpt.com/backend-api/accounts/check/v4-2023-04-27",
        headers={{"Authorization":"Bearer {}","Accept":"application/json","OAI-Device-Id":str(uuid.uuid4()),"OAI-Language":"en-US"}},
        impersonate="chrome",
    )
    if resp.status_code == 200:
        d = resp.json()
        for acc in d.get("accounts",{{}}).values():
            ent = acc.get("account",{{}}).get("entitlement",{{}})
            renews = ent.get("renews_at") or ent.get("expires_at")
            period = ent.get("billing_period","monthly")
            if renews:
                print(json.dumps({{"renews_at":renews,"billing_period":period}}))
                break
except Exception as e:
    pass
"#,
            token
        );

        let mut cmd = std::process::Command::new("python3");
        cmd.arg("-c").arg(&script);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let output = cmd.output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).ok()?;
        let renews_str = parsed.get("renews_at")?.as_str()?;
        let renews_dt = chrono::DateTime::parse_from_rfc3339(renews_str).ok()?;
        let now = chrono::Utc::now();
        let days = (renews_dt.with_timezone(&chrono::Utc) - now).num_days();
        let kst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let utc_str = renews_dt.with_timezone(&chrono::Utc).format("%Y-%m-%d").to_string();
        let kst_str = renews_dt.with_timezone(&kst).format("%Y-%m-%d").to_string();
        // 성공 시 캐시 저장
        if let Some(dir) = crate::paths::app_config_dir() {
            let cache = serde_json::json!({"renews_at": renews_str});
            let _ = std::fs::write(dir.join("chatgpt_renewal.json"), cache.to_string());
        }
        Some((utc_str, kst_str, days))
    }
    match inner() {
        Some((utc, kst, days)) => (Some(utc), Some(kst), Some(days)),
        None => (None, None, None),
    }
}

pub fn codex_cli_installed() -> bool {
    if let Some(home) = crate::paths::home() {
        let candidates = [
            home.join(".codex").join("auth.json"),
            home.join(".chatgpt-local").join("auth.json"),
        ];
        for c in &candidates {
            if c.exists() {
                return true;
            }
        }
    }

    let names: &[&str] = if cfg!(target_os = "windows") {
        &["codex.exe", "codex.cmd", "codex"]
    } else {
        &["codex"]
    };
    if let Ok(path_var) = std::env::var("PATH") {
        let sep = if cfg!(target_os = "windows") { ';' } else { ':' };
        for dir in path_var.split(sep) {
            if dir.is_empty() { continue; }
            for name in names {
                if std::path::PathBuf::from(dir).join(name).exists() {
                    return true;
                }
            }
        }
    }
    false
}

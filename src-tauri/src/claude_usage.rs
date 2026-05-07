use chrono::{DateTime, Datelike};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Serialize)]
pub struct UsageWindow {
    pub label: String,
    pub used_tokens: u64,
    pub limit_tokens: u64,
    pub used_messages: u64,
    pub limit_messages: u64,
    pub reset_label: String,
}

#[derive(Serialize)]
pub struct ClaudeUsage {
    pub plan: String,
    pub plan_raw: String,
    pub windows: Vec<UsageWindow>,
    pub total_messages: u64,
    pub fetched_at: i64,
}

#[derive(Serialize)]
pub struct DailyEntry {
    pub date: String,
    pub tokens: u64,
    pub messages: u32,
}

#[derive(Debug, Serialize)]
pub struct TrendDiag {
    pub path: String,
    pub path_exists: bool,
    pub jsonl_count: u32,
    pub total_bytes: u64,
}

#[tauri::command]
pub fn claude_trend_diag() -> TrendDiag {
    let projects_dir = crate::paths::claude_projects_dir();
    let path_str = projects_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let exists = projects_dir.as_ref().map(|p| p.exists()).unwrap_or(false);
    let mut count = 0u32;
    let mut bytes = 0u64;
    if let Some(p) = &projects_dir {
        if p.exists() {
            for entry in WalkDir::new(p).max_depth(3).into_iter().filter_map(|e| e.ok()) {
                if entry.path().is_file()
                    && entry.path().extension().and_then(|e| e.to_str()) == Some("jsonl")
                {
                    count += 1;
                    if let Ok(meta) = entry.metadata() {
                        bytes += meta.len();
                    }
                }
            }
        }
    }
    TrendDiag { path: path_str, path_exists: exists, jsonl_count: count, total_bytes: bytes }
}

#[tauri::command]
pub fn get_claude_daily_tokens(days: Option<u32>) -> Result<Vec<DailyEntry>, String> {
    let days = days.unwrap_or(30).clamp(1, 90);
    let projects_dir = crate::paths::claude_projects_dir()
        .ok_or_else(|| "no home directory".to_string())?;

    let now = chrono::Utc::now();
    let start_ts = now.timestamp() - (days as i64) * 24 * 3600;

    let mut buckets: std::collections::BTreeMap<String, (u64, u32)> = std::collections::BTreeMap::new();
    // pre-fill with zeros so empty days show in the chart
    for i in 0..days as i64 {
        let day = (now - chrono::Duration::days(days as i64 - 1 - i))
            .with_timezone(&chrono::Local);
        let key = format!("{:04}-{:02}-{:02}", day.year(), day.month(), day.day());
        buckets.insert(key, (0, 0));
    }

    if projects_dir.exists() {
        for entry in WalkDir::new(&projects_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file()
                || path.extension().and_then(|e| e.to_str()) != Some("jsonl")
            {
                continue;
            }
            if let Ok(meta) = path.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed.as_secs() > (days as u64 + 1) * 24 * 3600 {
                            continue;
                        }
                    }
                }
            }
            let Ok(content) = fs::read_to_string(path) else { continue };
            for line in content.lines() {
                let Some((ts, total)) = parse_line(line) else { continue };
                if ts < start_ts {
                    continue;
                }
                let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
                    .unwrap_or_else(chrono::Utc::now)
                    .with_timezone(&chrono::Local);
                let key = format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day());
                let entry = buckets.entry(key).or_insert((0, 0));
                entry.0 += total;
                entry.1 += 1;
            }
        }
    }

    Ok(buckets
        .into_iter()
        .map(|(date, (tokens, messages))| DailyEntry {
            date,
            tokens,
            messages,
        })
        .collect())
}

struct PlanLimits {
    name: &'static str,
    msg_5h: u64,
    msg_week: u64,
    tok_5h: u64,
    tok_week: u64,
}

pub fn plan_name_for(sub: &str) -> String {
    plan_for(sub).name.to_string()
}

pub fn read_subscription_pub() -> (String, String) {
    read_subscription()
}

fn plan_for(sub: &str) -> PlanLimits {
    let s = sub.to_ascii_lowercase();
    let s = s.as_str();
    // claude.ai actual values: default_claude_pro, default_claude_max_5x,
    // default_claude_max_20x, default_claude_free, etc. We match flexibly.
    if s.contains("max_20x") || s.contains("max20x") {
        PlanLimits {
            name: "Max 20×",
            msg_5h: 900,
            msg_week: 5_600,
            tok_5h: 1_000_000_000,
            tok_week: 6_000_000_000,
        }
    } else if s.contains("max_5x") || s.contains("max5x") || s == "max" {
        PlanLimits {
            name: "Max 5×",
            msg_5h: 225,
            msg_week: 1_400,
            tok_5h: 250_000_000,
            tok_week: 1_500_000_000,
        }
    } else if s.contains("pro") {
        PlanLimits {
            name: "Pro",
            msg_5h: 45,
            msg_week: 280,
            tok_5h: 50_000_000,
            tok_week: 300_000_000,
        }
    } else if s.contains("free") {
        PlanLimits {
            name: "Free",
            msg_5h: 10,
            msg_week: 60,
            tok_5h: 5_000_000,
            tok_week: 30_000_000,
        }
    } else {
        PlanLimits {
            name: "Unknown",
            msg_5h: 100,
            msg_week: 500,
            tok_5h: 50_000_000,
            tok_week: 300_000_000,
        }
    }
}

/// Read plan tier from ~/.claude.json (authoritative).
/// Prefers userRateLimitTier (per-user override) then organizationRateLimitTier.
/// Returns (tier_string, source_label).
fn read_subscription() -> (String, String) {
    let claude_json = match crate::paths::claude_json() {
        Some(p) => p,
        None => return ("".to_string(), "unavailable".to_string()),
    };
    if let Ok(content) = fs::read_to_string(&claude_json) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(oa) = v.get("oauthAccount") {
                let user_tier = oa
                    .get("userRateLimitTier")
                    .and_then(|s| s.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                let org_tier = oa
                    .get("organizationRateLimitTier")
                    .and_then(|s| s.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                if let Some(t) = user_tier {
                    return (t, ".claude.json::oauthAccount.userRateLimitTier".into());
                }
                if let Some(t) = org_tier {
                    return (t, ".claude.json::oauthAccount.organizationRateLimitTier".into());
                }
            }
        }
    }
    // last-resort fallback
    let cred = match crate::paths::claude_credentials() {
        Some(p) => p,
        None => return ("".to_string(), "unavailable".to_string()),
    };
    if let Ok(content) = fs::read_to_string(&cred) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(o) = v.get("claudeAiOauth") {
                if let Some(s) = o.get("rateLimitTier").and_then(|x| x.as_str()) {
                    if !s.is_empty() && s != "default_claude_ai" {
                        return (s.to_string(), ".credentials.json::rateLimitTier".into());
                    }
                }
            }
        }
    }
    ("".to_string(), "not found".to_string())
}

fn parse_line(line: &str) -> Option<(i64, u64)> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let ts_str = v.get("timestamp")?.as_str()?;
    let ts = DateTime::parse_from_rfc3339(ts_str).ok()?.timestamp();
    let usage = v.get("message")?.get("usage")?;
    let input = usage.get("input_tokens").and_then(|n| n.as_u64()).unwrap_or(0);
    let output = usage.get("output_tokens").and_then(|n| n.as_u64()).unwrap_or(0);
    let cache_create = usage
        .get("cache_creation_input_tokens")
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    let cache_read = usage
        .get("cache_read_input_tokens")
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    let total = input + output + cache_create + cache_read;
    if total == 0 {
        return None;
    }
    Some((ts, total))
}

fn fmt_remaining(secs: i64) -> String {
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

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(default)
}

#[tauri::command]
pub fn get_claude_usage() -> Result<ClaudeUsage, String> {
    let projects_dir = crate::paths::claude_projects_dir()
        .ok_or_else(|| "no home directory".to_string())?;

    let (tier_raw, _source) = read_subscription();
    let plan = plan_for(&tier_raw);

    let now = chrono::Utc::now().timestamp();
    let h5_cutoff = now - 5 * 3600;
    let w_cutoff = now - 7 * 24 * 3600;

    let mut h5_tokens: u64 = 0;
    let mut w_tokens: u64 = 0;
    let mut h5_msgs: u64 = 0;
    let mut w_msgs: u64 = 0;
    let mut earliest_5h: Option<i64> = None;
    let mut earliest_w: Option<i64> = None;

    if projects_dir.exists() {
        for entry in WalkDir::new(&projects_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file()
                || path.extension().and_then(|e| e.to_str()) != Some("jsonl")
            {
                continue;
            }
            if let Ok(meta) = path.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed.as_secs() > 8 * 24 * 3600 {
                            continue;
                        }
                    }
                }
            }
            let Ok(content) = fs::read_to_string(path) else { continue };
            for line in content.lines() {
                let Some((ts, total)) = parse_line(line) else { continue };
                if ts >= w_cutoff {
                    w_tokens += total;
                    w_msgs += 1;
                    if earliest_w.is_none() || Some(ts) < earliest_w {
                        earliest_w = Some(ts);
                    }
                    if ts >= h5_cutoff {
                        h5_tokens += total;
                        h5_msgs += 1;
                        if earliest_5h.is_none() || Some(ts) < earliest_5h {
                            earliest_5h = Some(ts);
                        }
                    }
                }
            }
        }
    }

    let h5_reset = match earliest_5h {
        Some(ts) => fmt_remaining(ts + 5 * 3600 - now),
        None => "—".to_string(),
    };
    let w_reset = match earliest_w {
        Some(ts) => fmt_remaining(ts + 7 * 24 * 3600 - now),
        None => "—".to_string(),
    };

    let msg_5h = env_u64("CLAUDE_5H_MSG_LIMIT", plan.msg_5h);
    let msg_w = env_u64("CLAUDE_WEEK_MSG_LIMIT", plan.msg_week);
    let tok_5h = env_u64("CLAUDE_5H_TOK_LIMIT", plan.tok_5h);
    let tok_w = env_u64("CLAUDE_WEEK_TOK_LIMIT", plan.tok_week);

    Ok(ClaudeUsage {
        plan: plan.name.to_string(),
        plan_raw: tier_raw,
        windows: vec![
            UsageWindow {
                label: "5h window".to_string(),
                used_tokens: h5_tokens,
                limit_tokens: tok_5h,
                used_messages: h5_msgs,
                limit_messages: msg_5h,
                reset_label: h5_reset,
            },
            UsageWindow {
                label: "Weekly".to_string(),
                used_tokens: w_tokens,
                limit_tokens: tok_w,
                used_messages: w_msgs,
                limit_messages: msg_w,
                reset_label: w_reset,
            },
        ],
        total_messages: w_msgs,
        fetched_at: now,
    })
}

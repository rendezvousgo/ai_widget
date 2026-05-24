use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct StatusPageComponent {
    name: String,
    status: String,
    #[serde(default)]
    only_show_if_degraded: bool,
}

#[derive(Debug, Deserialize)]
struct StatusPageIncident {
    name: String,
    status: String,
    created_at: String,
    #[serde(default)]
    components: Vec<StatusPageComponent>,
}

#[derive(Debug, Deserialize)]
struct StatusPageStatus {
    indicator: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct StatusPageSummary {
    status: StatusPageStatus,
    #[serde(default)]
    components: Vec<StatusPageComponent>,
    #[serde(default)]
    incidents: Vec<StatusPageIncident>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentStatus {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IncidentInfo {
    pub name: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub indicator: String,
    pub description: String,
    pub components: Vec<ComponentStatus>,
    pub incidents: Vec<IncidentInfo>,
    pub fetched_at: i64,
    pub cached: bool,
}

static CLAUDE_CACHE: Mutex<Option<(Instant, ServiceStatus)>> = Mutex::new(None);
static OPENAI_CACHE: Mutex<Option<(Instant, ServiceStatus)>> = Mutex::new(None);
const CACHE_TTL_SECS: u64 = 120;

fn get_cache(provider: &str) -> &'static Mutex<Option<(Instant, ServiceStatus)>> {
    match provider {
        "openai" => &OPENAI_CACHE,
        _ => &CLAUDE_CACHE,
    }
}

fn api_url(provider: &str) -> &'static str {
    match provider {
        "openai" => "https://status.openai.com/api/v2/summary.json",
        _ => "https://status.claude.com/api/v2/summary.json",
    }
}

#[tauri::command]
pub async fn get_service_status(provider: String) -> Result<ServiceStatus, String> {
    let cache = get_cache(&provider);

    if let Some((t, s)) = cache.lock().unwrap().as_ref() {
        if t.elapsed().as_secs() < CACHE_TTL_SECS {
            let mut cached = s.clone();
            cached.cached = true;
            return Ok(cached);
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(api_url(&provider))
        .header("User-Agent", "ai-quota-widget/0.1")
        .send()
        .await
        .map_err(|e| format!("network: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status().as_u16()));
    }

    let data: StatusPageSummary = resp.json().await.map_err(|e| format!("parse: {}", e))?;

    let components: Vec<ComponentStatus> = data.components
        .into_iter()
        .filter(|c| !c.only_show_if_degraded || c.status != "operational")
        .map(|c| ComponentStatus { name: c.name, status: c.status })
        .collect();

    let incidents: Vec<IncidentInfo> = data.incidents
        .into_iter()
        .map(|i| IncidentInfo {
            name: i.name,
            status: i.status,
            created_at: i.created_at,
        })
        .collect();

    let result = ServiceStatus {
        indicator: data.status.indicator,
        description: data.status.description,
        components,
        incidents,
        fetched_at: chrono::Utc::now().timestamp(),
        cached: false,
    };

    *cache.lock().unwrap() = Some((Instant::now(), result.clone()));
    Ok(result)
}

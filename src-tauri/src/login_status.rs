use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LoginStatus {
    pub logged_in: bool,
    pub cli_installed: bool,
    pub credentials_path: String,
}

fn has_valid_credentials() -> bool {
    let Some(path) = crate::paths::claude_credentials() else { return false };
    let Ok(content) = std::fs::read_to_string(&path) else { return false };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else { return false };
    v.get("claudeAiOauth")
        .and_then(|c| c.get("accessToken"))
        .and_then(|t| t.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

#[tauri::command]
pub fn claude_login_status() -> LoginStatus {
    LoginStatus {
        logged_in: has_valid_credentials(),
        cli_installed: crate::paths::claude_cli_installed(),
        credentials_path: crate::paths::claude_credentials()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
    }
}

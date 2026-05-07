use std::path::PathBuf;

/// Cross-platform home directory.
/// Linux/macOS: $HOME, Windows: %USERPROFILE%
pub fn home() -> Option<PathBuf> {
    dirs::home_dir()
}

/// Claude Code credentials path. Same layout on Linux/macOS/Windows
/// (Claude CLI writes ~/.claude/.credentials.json regardless of platform).
pub fn claude_credentials() -> Option<PathBuf> {
    Some(home()?.join(".claude").join(".credentials.json"))
}

/// Per-user Claude config (oauth profile, plan tier).
pub fn claude_json() -> Option<PathBuf> {
    Some(home()?.join(".claude.json"))
}

/// Claude project history directory (JSONL session files for daily token tally).
pub fn claude_projects_dir() -> Option<PathBuf> {
    Some(home()?.join(".claude").join("projects"))
}

/// Our own keystore directory.
/// Linux:   ~/.config/ai-quota-widget/
/// macOS:   ~/Library/Application Support/ai-quota-widget/
/// Windows: %APPDATA%\ai-quota-widget\
pub fn app_config_dir() -> Option<PathBuf> {
    let base = dirs::config_dir()?;
    let dir = base.join("ai-quota-widget");
    let _ = std::fs::create_dir_all(&dir);
    Some(dir)
}

pub fn keystore_path() -> Option<PathBuf> {
    Some(app_config_dir()?.join("keys.json"))
}

/// Detect whether the `claude` CLI is on PATH or in known install locations.
/// On Windows we deliberately avoid spawning a child process so no console
/// window flashes during the login-screen polling.
pub fn claude_cli_installed() -> bool {
    if let Some(home) = home() {
        let mut candidates: Vec<PathBuf> = vec![
            home.join(".local/bin/claude"),
            home.join(".npm-global/bin/claude"),
        ];
        #[cfg(target_os = "windows")]
        {
            candidates.push(home.join("AppData/Local/AnthropicClaude/claude.exe"));
            candidates.push(home.join("AppData/Local/Programs/Claude/claude.exe"));
            candidates.push(home.join("AppData/Roaming/npm/claude.cmd"));
            candidates.push(home.join("AppData/Roaming/npm/claude.ps1"));
            candidates.push(home.join("AppData/Roaming/npm/claude"));
            candidates.push(home.join("scoop/shims/claude.exe"));
            candidates.push(home.join("scoop/shims/claude"));
        }
        for candidate in &candidates {
            if candidate.exists() {
                return true;
            }
        }
    }

    // Walk PATH ourselves on every OS — no child process spawn, no console flash.
    if let Ok(path_var) = std::env::var("PATH") {
        let sep = if cfg!(target_os = "windows") { ';' } else { ':' };
        let names: &[&str] = if cfg!(target_os = "windows") {
            &["claude.exe", "claude.cmd", "claude.bat", "claude"]
        } else {
            &["claude"]
        };
        for dir in path_var.split(sep) {
            if dir.is_empty() {
                continue;
            }
            for name in names {
                if PathBuf::from(dir).join(name).exists() {
                    return true;
                }
            }
        }
    }
    false
}

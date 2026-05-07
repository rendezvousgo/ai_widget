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
pub fn claude_cli_installed() -> bool {
    if let Some(home) = home() {
        // Common install spots
        for candidate in [
            home.join(".local/bin/claude"),
            home.join(".npm-global/bin/claude"),
            #[cfg(target_os = "windows")]
            home.join("AppData/Local/AnthropicClaude/claude.exe"),
            #[cfg(target_os = "windows")]
            home.join("AppData/Roaming/npm/claude.cmd"),
        ] {
            if candidate.exists() {
                return true;
            }
        }
    }
    // Fall back to PATH lookup
    #[cfg(target_os = "windows")]
    let cmd = ("cmd", &["/C", "where claude"][..]);
    #[cfg(not(target_os = "windows"))]
    let cmd = ("sh", &["-c", "command -v claude >/dev/null 2>&1"][..]);
    std::process::Command::new(cmd.0)
        .args(cmd.1)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
